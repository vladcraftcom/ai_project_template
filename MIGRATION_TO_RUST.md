# План миграции C# (Avalonia) → Rust (Iced)

Важно: переписываем ТОЛЬКО C#-часть GUI/оболочки. Python-скрипты (например, `create_project.py`) остаются без изменений и вызываются новым приложением.

---

## 1) Инвентаризация текущего проекта (C#)
Источник: `./ai_project_template/ProjectCreator`
- UI: `MainWindow.axaml`
  - TextBox: `ProjectName` + TextBlock ошибки `ProjectNameError`.
  - CheckBox: `UseVenv`, `InstallPackages`, `RefreshTemplates`, `Force`.
  - Блок "Проверка окружения Python":
    - `PythonStatus`, `PipStatus`, `VenvStatus` + кнопка `RefreshEnvCommand`.
  - Кнопки: `CreateCommand` (Создать проект), `BrowseCommand` (Обзор).
  - Логи: readonly TextBox `Log`.
- ViewModel: `MainWindowViewModel.cs`
  - Свойства + вычислимое `CanCreate`.
  - Валидация имени (regex, запреты Windows, окончание на пробел/точку).
  - Проверка окружения: `python/python3`, `pip`, `virtualenv`/встроенный `venv`.
  - Создание проекта: запуск `create_project.py` с флагами (`--venv`, `--install`, `--refresh-templates`, `--force`), потоковый stdout/stderr в лог.
  - Busy UI: модалка `BusyWindow` на время выполнения.
- Python: `create_project.py` — остаётся как есть.

Ожидаемое поведение: начальная проверка окружения, ручное обновление, доступность Create при валидных условиях, потоковый лог, блокировка UI на время выполнения.

---

## 2) Целевой стек и структура (Rust)
- GUI: Iced 0.12 (Dark)
- Async/процессы: tokio + tokio::process
- Диалоги (при необходимости): rfd
- Regex: regex; Ошибки: anyhow

Предлагаемая структура:
```
ai_project_template_rs/
  Cargo.toml
  src/
    main.rs        # состояние, сообщения (Msg), update/view
    envcheck.rs    # проверки python/pip/venv
    validate.rs    # валидация имени проекта
    process.rs     # запуск create_project.py, стрим логов
create_project.py   # использовать как есть (лежит рядом, не трогаем)
```

Cargo.toml (зависимости):
```toml
[dependencies]
iced = { version = "0.12", features = ["tokio"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros", "process", "io-util"] }
regex = "1"
anyhow = "1"
rfd = "0.14"
```

---

## 3) Маппинг UI (Avalonia → Iced)
- Заголовок: "Project Creator".
- Поле "Project name" + текст ошибки.
- Блок "Options": 4 чекбокса (= флаги CLI к скрипту).
- Блок "Python environment check": 3 строки статусов + кнопка Refresh.
- Низ: Create project (enabled = `can_create()`), Browse (резерв).
- Логи: скроллируемая readonly-панель.
- Busy: дизейбл интерактива + статус (оверлей опционально).

Пример разметки (фрагмент):
```rust
use iced::widget::{column, row, text, text_input, checkbox, button, scrollable, container};
use iced::{Length, Element};

fn view(&self) -> Element<'_, Msg> {
    let name = text_input("Project name", &self.project_name)
        .on_input(Msg::NameChanged)
        .width(Length::Fixed(320.0));
    let name_err = text(&self.project_name_error);

    let opts = column![
        checkbox("Create venv (--venv)", self.use_venv).on_toggle(Msg::UseVenv),
        checkbox("Install packages (--install)", self.install_packages).on_toggle(Msg::Install),
        checkbox("Refresh templates (--refresh-templates)", self.refresh_templates).on_toggle(Msg::Refresh),
        checkbox("Force into existing folder (--force)", self.force).on_toggle(Msg::Force),
    ];

    let env = column![
        text("Python environment check").size(16),
        text(&self.python_status),
        text(&self.pip_status),
        text(&self.venv_status),
        button("Refresh").on_press(Msg::RefreshEnv),
    ];

    let actions = row![
        if self.can_create() { button("Create project").on_press(Msg::Create) } else { button("Create project") },
        button("Browse").on_press(Msg::Browse),
    ].spacing(10);

    let log = scrollable(text(self.log_lines.join("\n")).size(13)).height(Length::Fixed(120.0));

    container(column![
        text("Project Creator").size(18),
        row![ text("Project name:").width(Length::Fixed(120.0)), column![name, name_err] ].spacing(8),
        text("Options:"),
        opts,
        container(env).padding(8),
        actions,
        text("Log"),
        log,
    ].spacing(10).padding(16)).into()
}
```

---

## 4) Состояние и поведение
Состояние:
```rust
struct AppState {
    project_name: String,
    use_venv: bool,
    install_packages: bool,
    refresh_templates: bool,
    force: bool,
    python_status: String,
    pip_status: String,
    venv_status: String,
    project_name_error: String,
    is_busy: bool,
    log_lines: Vec<String>,
}

impl AppState {
    fn can_create(&self) -> bool {
        !self.is_busy
            && !self.project_name.trim().is_empty()
            && is_valid_project_name(&self.project_name)
            && self.python_status.contains("OK")
            && self.pip_status.contains("OK")
            && self.venv_status.contains("OK")
    }
}
```

Валидация имени (эквивалент C#):
```rust
use regex::Regex;

fn is_valid_project_name(name: &str) -> bool {
    let ok = Regex::new(r"^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$").unwrap().is_match(name);
    if !ok { return false; }
    if name.ends_with('.') || name.ends_with(' ') { return false; }
    const RESERVED: &[&str] = &[
        "CON","PRN","AUX","NUL","COM1","COM2","COM3","COM4","COM5","COM6","COM7","COM8","COM9",
        "LPT1","LPT2","LPT3","LPT4","LPT5","LPT6","LPT7","LPT8","LPT9"
    ];
    let upper = name.to_ascii_uppercase();
    !RESERVED.iter().any(|&r| r == upper)
}
```

Проверка окружения (эквивалент `CheckEnvironmentAsync`):
```rust
use tokio::process::Command;

async fn check_cmd(cmd: &str, args: &[&str]) -> bool {
    Command::new(cmd)
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false)
}

pub async fn check_environment() -> (String, String, String) {
    let py_ok = check_cmd("python", &["--version"]).await || check_cmd("python3", &["--version"]).await;
    let python_status = if py_ok { "Python: OK" } else { "Python: NOT FOUND (add to PATH)" }.to_string();

    let pip_ok = check_cmd("pip", &["--version"]).await
        || check_cmd("python", &["-m", "pip", "--version"]).await
        || check_cmd("python3", &["-m", "pip", "--version"]).await;
    let pip_status = if pip_ok { "pip: OK" } else { "pip: NOT FOUND (install pip)" }.to_string();

    let venv_ok = check_cmd("virtualenv", &["--version"]).await
        || check_cmd("python", &["-m", "virtualenv", "--version"]).await
        || check_cmd("python3", &["-m", "virtualenv", "--version"]).await
        || check_cmd("python", &["-c", "import venv; print('ok')"]).await
        || check_cmd("python3", &["-c", "import venv; print('ok')"]).await;
    let venv_status = if venv_ok { "venv/virtualenv: OK" } else { "venv/virtualenv: NOT FOUND" }.to_string();

    (python_status, pip_status, venv_status)
}
```

Запуск `create_project.py` (эквивалент `CreateAsync`):
```rust
use tokio::{io::{AsyncBufReadExt, BufReader}, process::Command};

fn build_args(name: &str, force: bool, venv: bool, install: bool, refresh: bool) -> Vec<String> {
    let mut v = vec!["create_project.py".to_string(), name.to_string()];
    if force { v.push("--force".into()); }
    if venv { v.push("--venv".into()); }
    if install { v.push("--install".into()); }
    if refresh { v.push("--refresh-templates".into()); }
    v
}

pub async fn run_create(name: &str, force: bool, venv: bool, install: bool, refresh: bool,
                        mut on_line: impl FnMut(String) + Send + 'static) -> anyhow::Result<i32> {
    let args = build_args(name, force, venv, install, refresh);
    on_line(format!("> python {} {}", args[0], args[1..].join(" ")));

    let mut child = Command::new("python")
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let mut out = BufReader::new(stdout).lines();
    let mut err = BufReader::new(stderr).lines();

    let out_task = tokio::spawn(async move {
        while let Ok(Some(line)) = out.next_line().await { on_line(line); }
    });
    let err_task = tokio::spawn(async move {
        while let Ok(Some(line)) = err.next_line().await { on_line(line); }
    });

    let status = child.wait().await?;
    let _ = out_task.await; let _ = err_task.await;
    Ok(status.code().unwrap_or(-1))
}
```

Busy/блокировка: `is_busy = true` на время запуска; кнопки без `on_press`/disabled; статус "Running..." (оверлей — по желанию).

---

## 5) Пошаговая миграция
1. Создать Iced-проект `ai_project_template_rs`, не трогая Python.
2. Реализовать состояние/Msg/update/view с эквивалентной разметкой.
3. Перенести валидацию имени.
4. Реализовать `check_environment()` и вызывать при старте и по кнопке.
5. Реализовать `run_create()` и стрим логов в UI.
6. Включить `can_create()`-логику доступности Create.
7. Добавить блокировку `is_busy` и статусы.
8. Протестировать на macOS/Windows/Linux.
9. Настроить CI (релизы GitHub, матрица OS/arch), без включения бинарников в репозиторий.

---

## 6) Чек-лист соответствия
- [ ] Поле имени + ошибка
- [ ] Опции → корректные флаги
- [ ] Проверки python/pip/venv + тексты
- [ ] Refresh делает повторную проверку
- [ ] Create запускает скрипт с нужными флагами
- [ ] Лог потоковый (stdout/stderr)
- [ ] Блокировка на время выполнения
- [ ] Browse зарезервирована (по желанию позже)
- [ ] Python-скрипты не меняем

---

## 7) Риски и примечания
- Отсутствие python/pip/venv в PATH — выводить подсказки, как в оригинале.
- Права на запись/запуск — логировать ошибки.
- Linux ARM64 в CI — потребуются cross-toolchains.

---

## 8) Итог
Перенос C# Avalonia → Rust Iced без изменения Python-части. Сохраняем UI/поведение: валидация, проверки, запуск `create_project.py`, потоковый лог, блокировка. Поставляем через Releases, без бинарников в репозитории.
