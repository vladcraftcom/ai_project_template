# API Документация

Справочник по API модулей AI Project Template.

## 📑 Содержание

1. [Модуль `presets`](#модуль-presets)
2. [Модуль `command`](#модуль-command)
3. [Структуры данных](#структуры-данных)

## 📦 Модуль `presets`

Модуль для работы с пресетами и их конфигурациями.

### Константы

#### `PRESETS_ZIP_URL`

```rust
pub const PRESETS_ZIP_URL: &str = "https://github.com/vladcraftcom/ai_prompt_presets/archive/refs/heads/main.zip";
```

URL для загрузки архива пресетов из GitHub.

#### `PRESETS_PATH_ENV_VAR`

```rust
pub const PRESETS_PATH_ENV_VAR: &str = "AI_PROJECT_TEMPLATE_PRESETS_PATH";
```

Имя переменной окружения для хранения пути к директории пресетов.

### Структуры

#### `PresetConfig`

Конфигурация пресета проекта.

```rust
pub struct PresetConfig {
    pub id: String,                    // preset_id из JSON
    pub name: String,                  // preset_name из JSON
    pub description: String,
    pub directories: Vec<String>,
    pub templates: Vec<TemplateConfig>,
    pub empty_files: Vec<String>,
    pub readme_template: String,
    pub fields: Vec<FieldConfig>,
    pub options: Vec<OptionConfig>,
}
```

#### `TemplateConfig`

Конфигурация шаблона файла.

```rust
pub struct TemplateConfig {
    pub source: String,      // Имя файла-источника в директории пресета
    pub destination: String, // Имя файла-назначения в проекте
}
```

#### `FieldConfig`

Конфигурация динамического поля пресета.

```rust
pub struct FieldConfig {
    pub id: String,
    pub label: String,
    pub required: bool,
    pub field_type: String,  // "text" или "select"
    pub options: Option<Vec<String>>, // Для типа "select"
    pub description: Option<String>,
}
```

#### `OptionConfig`

Конфигурация опции пресета.

```rust
pub struct OptionConfig {
    pub id: String,
    pub label: String,
    pub default: bool,
    pub description: Option<String>,
}
```

### Функции

#### `get_default_presets_path()`

```rust
pub fn get_default_presets_path() -> PathBuf
```

Возвращает путь по умолчанию для директории пресетов: `{HOME}/Documents/ai_prompt_presets`.

**Returns**: `PathBuf` - путь к директории пресетов по умолчанию

**Platform-specific behavior:**
- На Unix системах использует переменную `HOME`
- На Windows использует `USERPROFILE` как fallback

#### `save_presets_path_to_global_namespace()`

```rust
pub fn save_presets_path_to_global_namespace(path: &Path) -> Result<(), String>
```

Сохраняет путь к пресетам в глобальное пространство имен ОС.

**Arguments:**
- `path` - путь к директории пресетов для сохранения

**Returns:**
- `Ok(())` если путь успешно сохранен
- `Err(String)` с описанием ошибки

**Platform-specific implementation:**
- **Windows**: Использует команду `setx` для установки переменной окружения. Если `setx` недоступен, сохраняет в конфиг-файл.
- **Linux/macOS**: Сохраняет путь в файл `~/.config/ai_project_template/presets_path.txt`

#### `load_presets_path_from_global_namespace()`

```rust
pub fn load_presets_path_from_global_namespace() -> Option<PathBuf>
```

Загружает путь к пресетам из глобального пространства имен ОС.

**Returns:**
- `Some(PathBuf)` если путь найден
- `None` если путь не сохранен

**Порядок проверки:**
1. Переменная окружения `AI_PROJECT_TEMPLATE_PRESETS_PATH`
2. Конфигурационный файл `~/.config/ai_project_template/presets_path.txt`

#### `load_preset_config()`

```rust
pub fn load_preset_config(
    presets_dir: &Path,
    preset_id: &str
) -> Result<PresetConfig, String>
```

Загружает конфигурацию пресета из файла `files_config.json`.

**Arguments:**
- `presets_dir` - корневая директория со всеми пресетами
- `preset_id` - идентификатор пресета (имя директории)

**Returns:**
- `Ok(PresetConfig)` если конфигурация успешно загружена
- `Err(String)` с описанием ошибки

**Errors:**
- Файл `files_config.json` не существует
- Файл не может быть прочитан
- JSON не валиден или не соответствует структуре `PresetConfig`

**Example:**

```rust
use std::path::Path;
use ai_project_template::presets::load_preset_config;

let presets_dir = Path::new("/path/to/presets");
match load_preset_config(&presets_dir, "software") {
    Ok(config) => println!("Loaded preset: {}", config.name),
    Err(e) => eprintln!("Error: {}", e),
}
```

#### `discover_presets()`

```rust
pub fn discover_presets(presets_dir: &Path) -> Result<Vec<String>, String>
```

Обнаруживает все доступные пресеты в директории.

**Arguments:**
- `presets_dir` - корневая директория со всеми пресетами

**Returns:**
- `Ok(Vec<String>)` со списком идентификаторов найденных пресетов
- `Err(String)` с описанием ошибки

**Как работает:**
1. Сканирует директорию пресетов
2. Ищет поддиректории, содержащие файл `files_config.json`
3. Имя поддиректории используется как идентификатор пресета

**Example:**

```rust
use std::path::Path;
use ai_project_template::presets::discover_presets;

let presets_dir = Path::new("/path/to/presets");
match discover_presets(&presets_dir) {
    Ok(presets) => {
        println!("Found {} presets:", presets.len());
        for preset in presets {
            println!("  - {}", preset);
        }
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

#### `download_and_extract_presets()`

```rust
pub async fn download_and_extract_presets(
    target_dir: &Path,
    zip_url: &str
) -> Result<(), String>
```

Скачивает и распаковывает пресеты из GitHub.

**Arguments:**
- `target_dir` - директория, в которую будут распакованы пресеты
- `zip_url` - URL для скачивания ZIP архива пресетов

**Returns:**
- `Ok(())` если операция завершена успешно
- `Err(String)` с описанием ошибки

**Как работает:**
1. Создает целевую директорию если не существует
2. Скачивает ZIP архив из указанного URL
3. Распаковывает архив в целевую директорию
4. Перезаписывает только файлы из архива (сохраняет кастомные пресеты)
5. Удаляет временный ZIP файл

**Important**: Эта функция **не удаляет** существующие пресеты. Она только обновляет/добавляет те пресеты, которые есть в архиве.

**Errors:**
- Не удается скачать архив (сетевые ошибки, HTTP ошибки)
- Архив поврежден или не является валидным ZIP
- Нет прав на запись в целевую директорию
- Недостаточно места на диске

**Platform-specific behavior:**
- На Unix системах сохраняет права доступа файлов из архива
- На всех платформах удаляет префикс `ai_prompt_presets-main/` из путей в архиве

**Example:**

```rust
use std::path::Path;
use ai_project_template::presets::{download_and_extract_presets, PRESETS_ZIP_URL};

#[tokio::main]
async fn main() {
    let target_dir = Path::new("/path/to/presets");
    match download_and_extract_presets(&target_dir, PRESETS_ZIP_URL).await {
        Ok(()) => println!("Presets downloaded successfully!"),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

## 🛠️ Модуль `command`

Модуль для создания проектов на основе конфигурации пресета.

### Функции

#### `create_project()`

```rust
pub fn create_project(
    project_path: &Path,
    presets_dir: &Path,
    preset_config: &PresetConfig,
    project_name: &str,
    dynamic_fields: &HashMap<String, String>,
    options: &HashMap<String, bool>
) -> Result<Vec<String>, String>
```

Создает проект на основе конфигурации пресета.

**Arguments:**
- `project_path` - путь к создаваемой директории проекта
- `presets_dir` - корневая директория со всеми пресетами
- `preset_config` - конфигурация выбранного пресета
- `project_name` - имя проекта (используется в README и уведомлениях)
- `dynamic_fields` - значения динамических полей пресета для подстановки в шаблоны
- `options` - опции создания проекта (например, "refresh", "force")

**Returns:**
- `Ok(Vec<String>)` со списком строк лога операций при успехе
- `Err(String)` с описанием ошибки при неудаче

**Как работает:**
1. Проверяет существование директории проекта
2. Создает директорию проекта
3. Создает поддиректории согласно конфигурации
4. Копирует шаблоны файлов из пресета
5. Создает пустые файлы
6. Генерирует README.md с подстановкой значений

**Errors:**
- Директория проекта уже существует и не пуста (без опции "force")
- Нет прав на создание директорий или файлов
- Шаблон-источник не найден
- Недостаточно места на диске

**Example:**

```rust
use std::path::Path;
use std::collections::HashMap;
use ai_project_template::presets::PresetConfig;
use ai_project_template::command::create_project;

let project_path = Path::new("./my_project");
let presets_dir = Path::new("./presets");
let preset_config = /* ... */;
let project_name = "my_project";
let dynamic_fields = HashMap::new();
let options = HashMap::new();

match create_project(
    project_path,
    presets_dir,
    &preset_config,
    project_name,
    &dynamic_fields,
    &options,
) {
    Ok(log_lines) => {
        for line in log_lines {
            println!("{}", line);
        }
    }
    Err(e) => eprintln!("Ошибка: {}", e),
}
```

## 📊 Структуры данных

### HashMap для динамических полей

```rust
HashMap<String, String>  // field_id -> value
```

Пример:
```rust
let mut fields = HashMap::new();
fields.insert("genre".to_string(), "Science Fiction".to_string());
```

### HashMap для опций

```rust
HashMap<String, bool>  // option_id -> enabled
```

Пример:
```rust
let mut options = HashMap::new();
options.insert("refresh".to_string(), true);
options.insert("force".to_string(), false);
```

## 🔗 Связи между модулями

```
main.rs
  ├── использует presets::*
  │   ├── discover_presets()
  │   ├── load_preset_config()
  │   ├── download_and_extract_presets()
  │   └── save/load_presets_path_to_global_namespace()
  │
  └── использует command::*
      └── create_project()
          └── использует presets::PresetConfig
```

## 📝 Примечания

### Обработка ошибок

Все функции модуля `presets` возвращают `Result<T, String>`, где `String` содержит человекочитаемое описание ошибки.

### Асинхронность

- `download_and_extract_presets()` - асинхронная функция, требует `tokio` runtime
- Все остальные функции синхронные

### Потокобезопасность

- Все функции работают с файловой системой и не требуют дополнительной синхронизации
- Для параллельного использования создавайте отдельные экземпляры структур или используйте мьютексы

---

*Следующие разделы: [Руководство разработчика](DEVELOPMENT.md) | [Глоссарий](GLOSSARY.md)*

