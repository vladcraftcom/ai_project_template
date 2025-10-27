use iced::theme::{self, Theme};
use iced::widget::{button, checkbox, column, container, progress_bar, row, scrollable, text, text_input};
use iced::{Application, Command, Element, Length, Settings, Subscription};
use std::time::Instant;

#[derive(Clone, Debug)]
enum Msg {
    NameChanged(String),
    UseVenv(bool),
    Install(bool),
    Refresh(bool),
    Force(bool),
    RefreshEnv,
    Create,
    ProcessFinished { lines: Vec<String>, code: i32 },
    SetEnv(String, String, String),
    SetBusy(bool),
    Tick, // для обновления прогресса диалога
}

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
    min_busy_ms: u64,
    show_dialog: bool,
    dialog_progress: f32,
    dialog_start: Option<Instant>,
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

impl Application for AppState {
    type Executor = iced::executor::Default;
    type Message = Msg;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (
            Self {
                project_name: String::new(),
                use_venv: false,
                install_packages: false,
                refresh_templates: false,
                force: false,
                python_status: "Checking...".into(),
                pip_status: "Checking...".into(),
                venv_status: "Checking...".into(),
                project_name_error: String::new(),
                is_busy: false,
                log_lines: Vec::new(),
                min_busy_ms: 2000,
                show_dialog: false,
                dialog_progress: 0.0,
                dialog_start: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String { "Project Creator".into() }
    fn theme(&self) -> Theme { theme::Theme::Dark }

    fn subscription(&self) -> Subscription<Self::Message> {
        if self.show_dialog {
            iced::time::every(std::time::Duration::from_millis(50)).map(|_| Msg::Tick)
        } else {
            Subscription::none()
        }
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Msg::NameChanged(s) => {
                self.project_name = s;
                self.project_name_error = if is_valid_project_name(&self.project_name) { String::new() } else { "Invalid name".into() };
            }
            Msg::UseVenv(v) => self.use_venv = v,
            Msg::Install(v) => self.install_packages = v,
            Msg::Refresh(v) => self.refresh_templates = v,
            Msg::Force(v) => self.force = v,
            Msg::RefreshEnv => {
                return Command::perform(check_environment_async(), |(py, pip, venv)| Msg::SetEnv(py, pip, venv));
            }
            Msg::Create => {
                if !self.can_create() { return Command::none(); }
                self.is_busy = true;
                self.log_lines.clear();
                // Запустить диалог на 2 секунды с прогресс-баром
                self.show_dialog = true;
                self.dialog_progress = 0.0;
                self.dialog_start = Some(Instant::now());
                let name = self.project_name.clone();
                let use_venv = self.use_venv;
                let install = self.install_packages;
                let refresh = self.refresh_templates;
                let force = self.force;
                let min_ms = self.min_busy_ms;
                return Command::perform(run_create_collect(name, force, use_venv, install, refresh, min_ms), |(lines, code)| Msg::ProcessFinished { lines, code });
            }
            Msg::ProcessFinished { lines, code } => {
                for l in lines { self.log_lines.push(l); }
                self.log_lines.push("Done".to_string());
                self.log_lines.push(format!("ExitCode: {}", code));
                self.is_busy = false;
            }
            Msg::SetEnv(py, pip, venv) => {
                self.python_status = py;
                self.pip_status = pip;
                self.venv_status = venv;
            }
            Msg::SetBusy(b) => self.is_busy = b,
            Msg::Tick => {
                if let Some(start) = self.dialog_start {
                    let elapsed = start.elapsed().as_millis() as f32;
                    let total = self.min_busy_ms as f32;
                    self.dialog_progress = (elapsed / total).clamp(0.0, 1.0);
                    if self.dialog_progress >= 1.0 {
                        self.show_dialog = false;
                        self.dialog_start = None;
                        self.dialog_progress = 0.0;
                    }
                }
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let name = text_input("Project name", &self.project_name).on_input(Msg::NameChanged).width(Length::Fixed(320.0));
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
            if self.can_create() { button("Create project").on_press(Msg::Create) } else { button("Create project") }
        ]
        .spacing(10);

        let log = scrollable(text(self.log_lines.join("\n")).size(13)).height(Length::Fixed(120.0));

        let dialog: Element<Msg> = if self.show_dialog {
            container(
                column![
                    text("Processing...").size(16),
                    progress_bar(0.0..=1.0, self.dialog_progress),
                    text(format!("{:.0}%", self.dialog_progress * 100.0)).size(12)
                ]
                .spacing(6)
            )
            .padding(12)
            .into()
        } else { container(column![]).into() };

        container(column![
            text("Project Creator").size(18),
            row![ text("Project name:").width(Length::Fixed(120.0)), column![name, name_err] ].spacing(8),
            text("Options:"),
            opts,
            container(env).padding(8),
            actions,
            dialog,
            text("Log"),
            log,
        ].spacing(10).padding(16))
        .into()
    }
}

#[tokio::main]
async fn main() -> iced::Result {
    AppState::run(Settings::default())
}

fn is_valid_project_name(name: &str) -> bool {
    use regex::Regex;
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

async fn check_cmd(cmd: &str, args: &[&str]) -> bool {
    use tokio::process::Command;
    Command::new(cmd)
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false)
}

async fn check_environment_async() -> (String, String, String) {
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

async fn run_create_collect(name: String, force: bool, venv: bool, install: bool, refresh: bool, min_busy_ms: u64) -> (Vec<String>, i32) {
    use std::path::PathBuf;
    use tokio::{io::{AsyncBufReadExt, BufReader}, process::Command};
    use tokio::time::{sleep, Duration, Instant};
    let mut lines: Vec<String> = Vec::new();

    // Сформировать аргументы для скрипта
    let mut args: Vec<String> = vec!["create_project.py".into(), name.clone()];
    if force { args.push("--force".into()); }
    if venv { args.push("--venv".into()); }
    if install { args.push("--install".into()); }
    if refresh { args.push("--refresh-templates".into()); }

    // Определить рабочую директорию: каталог, где лежит исполняемый файл
    let exe_dir: PathBuf = match std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf())) {
        Some(d) => d,
        None => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    };

    // Выбрать интерпретатор: сначала python, если нет — python3
    let py = if check_cmd("python", &["--version"]).await { "python" } else { "python3" };
    lines.push(format!("> {} {}", py, args.join(" ")));

    let start = Instant::now();
    // Запустить процесс с указанной рабочей директорией, чтобы относительный путь к create_project.py работал
    let mut child = match Command::new(py)
        .current_dir(&exe_dir)
        .args(&args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn() {
        Ok(c) => c,
        Err(e) => return (vec![format!("Ошибка запуска: {}", e)], -1),
    };

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let mut out = BufReader::new(stdout).lines();
    let mut err = BufReader::new(stderr).lines();

    while let Ok(Some(line)) = out.next_line().await { lines.push(line); }
    while let Ok(Some(line)) = err.next_line().await { lines.push(line); }

    let status = child.wait().await.ok().and_then(|s| s.code()).unwrap_or(-1);
    // Гарантировать минимум ~2 секунды видимого busy (как в C#)
    let elapsed = start.elapsed();
    if elapsed < Duration::from_millis(min_busy_ms) {
        sleep(Duration::from_millis(min_busy_ms) - elapsed).await;
    }
    (lines, status)
}
