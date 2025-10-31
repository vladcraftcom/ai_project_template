//! # Модуль управления пресетами
//!
//! Этот модуль предоставляет функциональность для работы с пресетами проектов:
//! - Загрузка конфигураций пресетов из JSON файлов
//! - Обнаружение доступных пресетов в директории
//! - Загрузка пресетов из GitHub репозитория
//! - Сохранение и загрузка пути к пресетам в глобальное пространство имен ОС
//!
//! ## Структура пресета
//!
//! Каждый пресет должен находиться в отдельной директории и содержать файл `files_config.json`
//! с конфигурацией структуры проекта, шаблонов и полей.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::env;
use std::fs;
use std::io::{self, Write};

/// URL для загрузки архива пресетов из GitHub
pub const PRESETS_ZIP_URL: &str = "https://github.com/vladcraftcom/ai_prompt_presets/archive/refs/heads/main.zip";

/// Имя переменной окружения для хранения пути к директории пресетов
pub const PRESETS_PATH_ENV_VAR: &str = "AI_PROJECT_TEMPLATE_PRESETS_PATH";

/// Конфигурация пресета проекта
///
/// Описывает структуру проекта, который будет создан на основе этого пресета.
/// Загружается из файла `files_config.json` в директории пресета.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PresetConfig {
    #[serde(rename = "preset_id")]
    pub id: String,
    #[serde(rename = "preset_name")]
    pub name: String,
    pub description: String,
    pub directories: Vec<String>,
    pub templates: Vec<TemplateConfig>,
    #[serde(rename = "empty_files")]
    pub empty_files: Vec<String>,
    #[serde(rename = "readme_template")]
    pub readme_template: String,
    pub fields: Vec<FieldConfig>,
    pub options: Vec<OptionConfig>,
}

/// Конфигурация шаблона файла
///
/// Описывает файл-шаблон, который будет скопирован из директории пресета
/// в создаваемый проект.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TemplateConfig {
    /// Имя файла-источника в директории пресета
    pub source: String,
    /// Имя файла-назначения в создаваемом проекте
    pub destination: String,
}

/// Конфигурация динамического поля пресета
///
/// Описывает поле ввода в UI, которое будет отображено при выборе пресета.
/// Значение поля подставляется в шаблон README при создании проекта.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FieldConfig {
    /// Уникальный идентификатор поля
    pub id: String,
    /// Отображаемая метка поля
    pub label: String,
    /// Обязательно ли заполнение поля
    pub required: bool,
    /// Тип поля: "text" или "select"
    #[serde(rename = "type")]
    pub field_type: String,
    /// Опции для выпадающего списка (только для типа "select")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<String>>,
    /// Описание поля (опционально)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Конфигурация опции пресета
///
/// Описывает флаг/чекбокс, который будет отображен в UI при выборе пресета.
/// Опции используются для настройки поведения при создании проекта.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OptionConfig {
    /// Уникальный идентификатор опции
    pub id: String,
    /// Отображаемая метка опции
    pub label: String,
    /// Значение по умолчанию
    pub default: bool,
    /// Описание опции (опционально)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Получить путь по умолчанию для директории пресетов
///
/// Возвращает путь `{HOME}/Documents/ai_prompt_presets` на всех платформах.
///
/// # Returns
///
/// Путь к директории пресетов по умолчанию
///
/// # Platform-specific behavior
///
/// - На Unix системах использует переменную `HOME`
/// - На Windows использует `USERPROFILE` как fallback, если `HOME` не задана
pub fn get_default_presets_path() -> PathBuf {
    let home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE")) // Windows fallback
        .unwrap_or_else(|_| ".".to_string());
    
    PathBuf::from(home).join("Documents").join("ai_prompt_presets")
}

/// Сохранить путь к пресетам в глобальное пространство имен ОС
///
/// Сохраняет путь к директории пресетов так, чтобы он был доступен при следующем запуске приложения.
///
/// # Platform-specific implementation
///
/// - **Windows**: Использует команду `setx` для установки переменной окружения пользователя.
///   Если `setx` недоступен, сохраняет в конфиг-файл как fallback.
/// - **Linux/macOS**: Сохраняет путь в файл `~/.config/ai_project_template/presets_path.txt`
///
/// # Arguments
///
/// * `path` - путь к директории пресетов для сохранения
///
/// # Returns
///
/// `Ok(())` если путь успешно сохранен, иначе `Err` с описанием ошибки
pub fn save_presets_path_to_global_namespace(path: &Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        // Для Windows используем переменную окружения пользователя
        // Это работает без необходимости работы с реестром
        use std::process::Command;
        let path_str = path.to_string_lossy().to_string();
        
        // Устанавливаем переменную окружения через setx (только для текущего пользователя)
        // Это сохраняет её перманентно, но доступна только в новых процессах
        // Альтернатива: использовать winreg crate для реестра
        let output = Command::new("setx")
            .args(&["AI_PROJECT_TEMPLATE_PRESETS_PATH", &path_str])
            .output()
            .map_err(|e| format!("Failed to run setx: {}. Note: setx may not be in PATH.", e))?;
        
        if !output.status.success() {
            // Fallback: сохранить в конфиг файл
            return save_to_config_file(path);
        }
        
        Ok(())
    }
    
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        save_to_config_file(path)
    }
}

/// Сохранить путь в конфигурационный файл (Linux/macOS)
///
/// Создает файл `~/.config/ai_project_template/presets_path.txt` с путем к пресетам.
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn save_to_config_file(path: &Path) -> Result<(), String> {
    let home = env::var("HOME")
        .map_err(|_| "HOME environment variable not set")?;
    
    let config_path = PathBuf::from(home)
        .join(".config")
        .join("ai_project_template");
    
    fs::create_dir_all(&config_path)
        .map_err(|e| format!("Failed to create config dir: {}", e))?;
    
    let config_file = config_path.join("presets_path.txt");
    fs::write(&config_file, path.to_string_lossy().as_ref())
        .map_err(|e| format!("Failed to write config file: {}", e))?;
    
    Ok(())
}

/// Сохранить путь в конфигурационный файл (Windows fallback)
///
/// Создает файл `%USERPROFILE%\.config\ai_project_template\presets_path.txt` с путем к пресетам.
/// Используется как fallback если команда `setx` недоступна.
#[cfg(target_os = "windows")]
fn save_to_config_file(path: &Path) -> Result<(), String> {
    // Для Windows также сохраняем в конфиг файл как fallback
    let home = env::var("USERPROFILE")
        .or_else(|_| env::var("HOME"))
        .map_err(|_| "Could not determine home directory")?;
    
    let config_path = PathBuf::from(home)
        .join(".config")
        .join("ai_project_template");
    
    fs::create_dir_all(&config_path)
        .map_err(|e| format!("Failed to create config dir: {}", e))?;
    
    let config_file = config_path.join("presets_path.txt");
    fs::write(&config_file, path.to_string_lossy().as_ref())
        .map_err(|e| format!("Failed to write config file: {}", e))?;
    
    Ok(())
}

/// Загрузить путь к пресетам из глобального пространства имен ОС
///
/// Пытается загрузить путь к директории пресетов, сохраненный ранее.
/// Проверяет сначала переменную окружения (для текущей сессии),
/// затем конфигурационный файл (для постоянного хранения).
///
/// # Returns
///
/// `Some(PathBuf)` если путь найден, иначе `None`
pub fn load_presets_path_from_global_namespace() -> Option<PathBuf> {
    // Сначала проверяем переменную окружения (актуальная для текущей сессии)
    if let Ok(path) = env::var(PRESETS_PATH_ENV_VAR) {
        return Some(PathBuf::from(path));
    }
    
    // Затем проверяем конфиг файл (работает на всех платформах)
    if let Ok(home) = env::var("HOME").or_else(|_| env::var("USERPROFILE")) {
        let config_file = PathBuf::from(home)
            .join(".config")
            .join("ai_project_template")
            .join("presets_path.txt");
        
        if let Ok(content) = fs::read_to_string(&config_file) {
            let trimmed = content.trim();
            if !trimmed.is_empty() {
                return Some(PathBuf::from(trimmed));
            }
        }
    }
    
    None
}

/// Загрузить конфигурацию пресета из файла
///
/// Читает и парсит JSON файл `files_config.json` из директории пресета.
///
/// # Arguments
///
/// * `presets_dir` - корневая директория со всеми пресетами
/// * `preset_id` - идентификатор пресета (имя директории)
///
/// # Returns
///
/// `Ok(PresetConfig)` если конфигурация успешно загружена и распарсена,
/// иначе `Err` с описанием ошибки
///
/// # Errors
///
/// Возвращает ошибку если:
/// - файл `files_config.json` не существует
/// - файл не может быть прочитан
/// - JSON не валиден или не соответствует структуре `PresetConfig`
pub fn load_preset_config(presets_dir: &Path, preset_id: &str) -> Result<PresetConfig, String> {
    let config_path = presets_dir.join(preset_id).join("files_config.json");
    
    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read preset config from {:?}: {}", config_path, e))?;
    
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse preset config: {}", e))
}

/// Обнаружить все доступные пресеты в директории
///
/// Сканирует директорию пресетов и находит все поддиректории, содержащие файл `files_config.json`.
/// Имя поддиректории используется как идентификатор пресета.
///
/// # Arguments
///
/// * `presets_dir` - корневая директория со всеми пресетами
///
/// # Returns
///
/// `Ok(Vec<String>)` со списком идентификаторов найденных пресетов,
/// иначе `Err` с описанием ошибки
///
/// # Example
///
/// Если структура директории следующая:
/// ```text
/// presets/
///   ├── software/
///   │   └── files_config.json
///   └── book/
///       └── files_config.json
/// ```
///
/// Функция вернет `vec!["software", "book"]`
pub fn discover_presets(presets_dir: &Path) -> Result<Vec<String>, String> {
    let dir = fs::read_dir(presets_dir)
        .map_err(|e| format!("Failed to read presets directory {:?}: {}", presets_dir, e))?;
    
    let mut presets = Vec::new();
    
    for entry in dir {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        
        if path.is_dir() {
            let config_path = path.join("files_config.json");
            if config_path.exists() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    presets.push(name.to_string());
                }
            }
        }
    }
    
    Ok(presets)
}

/// Получить имя пресета для отображения
///
/// Загружает конфигурацию пресета и возвращает человекочитаемое имя (`preset_name`).
/// Если загрузка не удалась, возвращает идентификатор пресета.
///
/// # Arguments
///
/// * `presets_dir` - корневая директория со всеми пресетами
/// * `preset_id` - идентификатор пресета
///
/// # Returns
///
/// Имя пресета для отображения (preset_name из конфига или preset_id как fallback)
pub fn get_preset_display_name(presets_dir: &Path, preset_id: &str) -> String {
    match load_preset_config(presets_dir, preset_id) {
        Ok(config) => config.name,
        Err(_) => preset_id.to_string(),
    }
}

/// Скачать и распаковать пресеты из GitHub
///
/// Обновляет пресеты из GitHub, не удаляя кастомные пресеты пользователя:
/// 1. Скачивает ZIP архив из указанного URL
/// 2. Распаковывает архив в целевую директорию (перезаписывая только файлы из архива)
/// 3. Удаляет временный ZIP файл
///
/// **Важно**: Эта функция не удаляет существующие пресеты. Она только обновляет/добавляет
/// те пресеты, которые есть в архиве. Кастомные пресеты пользователя останутся нетронутыми.
///
/// # Arguments
///
/// * `target_dir` - директория, в которую будут распакованы пресеты
/// * `zip_url` - URL для скачивания ZIP архива пресетов
///
/// # Returns
///
/// `Ok(())` если операция завершена успешно, иначе `Err` с описанием ошибки
///
/// # Platform-specific behavior
///
/// - На Unix системах сохраняет права доступа файлов из архива
/// - На всех платформах удаляет префикс `ai_prompt_presets-main/` из путей в архиве
///
/// # Errors
///
/// Может вернуть ошибку если:
/// - не удается скачать архив (сетевые ошибки, HTTP ошибки)
/// - архив поврежден или не является валидным ZIP
/// - нет прав на запись в целевую директорию
/// - недостаточно места на диске
pub async fn download_and_extract_presets(
    target_dir: &Path,
    zip_url: &str,
) -> Result<(), String> {
    // 2. Скачать ZIP архив
    let response = reqwest::get(zip_url)
        .await
        .map_err(|e| format!("Failed to download from {}: {}", zip_url, e))?;
    
    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }
    
    // 3. Сохранить во временный файл в целевой директории
    let temp_zip = target_dir.parent()
        .unwrap_or(target_dir)
        .join("presets_temp.zip");
    
    let bytes = response.bytes()
        .await
        .map_err(|e| format!("Failed to read response bytes: {}", e))?;
    
    let mut file = fs::File::create(&temp_zip)
        .map_err(|e| format!("Failed to create temp file {:?}: {}", temp_zip, e))?;
    
    file.write_all(&bytes)
        .map_err(|e| format!("Failed to write temp file: {}", e))?;
    file.sync_all()
        .map_err(|e| format!("Failed to sync temp file: {}", e))?;
    drop(file); // Закрыть файл перед распаковкой
    
    // 4. Распаковать ZIP
    let zip_file = fs::File::open(&temp_zip)
        .map_err(|e| format!("Failed to open zip file {:?}: {}", temp_zip, e))?;
    
    let mut archive = zip::ZipArchive::new(zip_file)
        .map_err(|e| format!("Failed to open zip archive: {}", e))?;
    
    // Распаковать все файлы
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("Failed to get file {} from archive: {}", i, e))?;
        
        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };
        
        // Убрать префикс ai_prompt_presets-main/ если есть
        let outpath = if outpath.starts_with("ai_prompt_presets-main/") {
            PathBuf::from(outpath.strip_prefix("ai_prompt_presets-main/").unwrap())
        } else {
            PathBuf::from(outpath)
        };
        
        let full_path = target_dir.join(&outpath);
        
        if file.name().ends_with('/') {
            // Создать директорию
            fs::create_dir_all(&full_path)
                .map_err(|e| format!("Failed to create dir {:?}: {}", full_path, e))?;
        } else {
            // Создать родительские директории если нужно
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create parent dir {:?}: {}", parent, e))?;
            }
            
            // Извлечь файл
            let mut outfile = fs::File::create(&full_path)
                .map_err(|e| format!("Failed to create file {:?}: {}", full_path, e))?;
            
            io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("Failed to extract file {:?}: {}", full_path, e))?;
        }
        
        // Установить права доступа (для Unix)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&full_path, fs::Permissions::from_mode(mode))
                    .ok(); // Игнорируем ошибки прав доступа
            }
        }
    }
    
    // 5. Удалить временный ZIP файл
    fs::remove_file(&temp_zip)
        .ok(); // Игнорируем ошибки удаления
    
    Ok(())
}

