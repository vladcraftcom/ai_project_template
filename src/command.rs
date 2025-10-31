//! # Модуль создания проектов
//!
//! Этот модуль содержит логику создания структуры проекта на основе конфигурации пресета.
//! Все операции создания проекта выполняются синхронно и возвращают детальный лог операций.

use crate::presets::PresetConfig;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;

/// Создать проект на основе конфигурации пресета
///
/// Выполняет полный цикл создания проекта:
/// 1. Проверяет и создает директорию проекта
/// 2. Создает поддиректории согласно конфигурации
/// 3. Копирует шаблоны файлов из пресета
/// 4. Создает пустые файлы
/// 5. Генерирует README.md с подстановкой значений
///
/// # Arguments
///
/// * `project_path` - путь к создаваемой директории проекта
/// * `presets_dir` - корневая директория со всеми пресетами
/// * `preset_config` - конфигурация выбранного пресета
/// * `project_name` - имя проекта (используется в README и уведомлениях)
/// * `dynamic_fields` - значения динамических полей пресета для подстановки в шаблоны
/// * `options` - опции создания проекта (например, "refresh", "force")
///
/// # Returns
///
/// `Ok(Vec<String>)` со списком строк лога операций при успехе,
/// `Err(String)` с описанием ошибки при неудаче
///
/// # Errors
///
/// Функция вернет ошибку если:
/// - директория проекта уже существует и не пуста (без опции "force")
/// - нет прав на создание директорий или файлов
/// - шаблон-источник не найден
/// - недостаточно места на диске
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use std::collections::HashMap;
/// # use ai_project_template::presets::PresetConfig;
/// # use ai_project_template::command::create_project;
///
/// let project_path = Path::new("./my_project");
/// let presets_dir = Path::new("./presets");
/// let preset_config = /* ... */;
/// let project_name = "my_project";
/// let dynamic_fields = HashMap::new();
/// let options = HashMap::new();
///
/// match create_project(
///     project_path,
///     presets_dir,
///     &preset_config,
///     project_name,
///     &dynamic_fields,
///     &options,
/// ) {
///     Ok(log_lines) => {
///         for line in log_lines {
///             println!("{}", line);
///         }
///     }
///     Err(e) => eprintln!("Ошибка: {}", e),
/// }
/// ```
pub fn create_project(
    project_path: &Path,
    presets_dir: &Path,
    preset_config: &PresetConfig,
    project_name: &str,
    dynamic_fields: &HashMap<String, String>,
    options: &HashMap<String, bool>,
) -> Result<Vec<String>, String> {
    let mut log_lines = Vec::new();
    
    // Проверка: существует ли директория и не пуста ли она
    let force = options.get("force").copied().unwrap_or(false);
    if project_path.exists() {
        let is_empty = project_path.read_dir()
            .map_err(|e| format!("Failed to read project directory: {}", e))?
            .next()
            .is_none();
        
        if !is_empty && !force {
            return Err(format!(
                "Project directory {:?} already exists and is not empty. Use --force to override.",
                project_path
            ));
        }
    }
    
    // 1. Создать директорию проекта
    log_lines.push(format!("Creating project directory: {:?}", project_path));
    fs::create_dir_all(project_path)
        .map_err(|e| format!("Failed to create project directory: {}", e))?;
    
    // 2. Создать поддиректории из конфига пресета
    for dir_name in &preset_config.directories {
        let dir_path = project_path.join(dir_name);
        log_lines.push(format!("Creating subdirectory: {:?}", dir_path));
        fs::create_dir_all(&dir_path)
            .map_err(|e| format!("Failed to create directory {:?}: {}", dir_path, e))?;
    }
    
    // 3. Скопировать шаблоны из папки пресета
    let preset_source_dir = presets_dir.join(&preset_config.id);
    let refresh = options.get("refresh").copied().unwrap_or(false);
    
    for template in &preset_config.templates {
        let source_path = preset_source_dir.join(&template.source);
        let dest_path = project_path.join(&template.destination);
        
        // Проверка существования файла назначения (если refresh=false, пропускаем существующие)
        if dest_path.exists() && !refresh {
            log_lines.push(format!("Skipping existing file: {:?}", dest_path));
            continue;
        }
        
        if !source_path.exists() {
            log_lines.push(format!("Warning: Template source not found: {:?}", source_path));
            continue;
        }
        
        log_lines.push(format!("Copying template: {:?} -> {:?}", source_path, dest_path));
        
        // Создать родительские директории если нужно
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create parent directory for {:?}: {}", dest_path, e))?;
        }
        
        fs::copy(&source_path, &dest_path)
            .map_err(|e| format!("Failed to copy template {:?} to {:?}: {}", source_path, dest_path, e))?;
    }
    
    // 4. Создать пустые файлы из конфига
    for file_name in &preset_config.empty_files {
        let file_path = project_path.join(file_name);
        if file_path.exists() && !refresh {
            log_lines.push(format!("Skipping existing empty file: {:?}", file_path));
            continue;
        }
        
        log_lines.push(format!("Creating empty file: {:?}", file_path));
        
        // Создать родительские директории если нужно
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create parent directory for {:?}: {}", file_path, e))?;
        }
        
        fs::File::create(&file_path)
            .map_err(|e| format!("Failed to create empty file {:?}: {}", file_path, e))?;
    }
    
    // 5. Генерировать README на основе шаблона из пресета
    let readme_path = project_path.join("README.md");
    let refresh_readme = refresh || !readme_path.exists();
    
    if refresh_readme {
        log_lines.push(format!("Generating README: {:?}", readme_path));
        
        let datetime = chrono::Local::now()
            .format("%Y-%m-%d %H:%M")
            .to_string();
        
        // Подстановка значений в шаблон README
        let mut readme_content = preset_config.readme_template.clone();
        
        // Подстановка имени проекта
        readme_content = readme_content.replace("{PROJECT_NAME}", project_name);
        readme_content = readme_content.replace("{project_name}", project_name);
        
        // Подстановка даты создания
        readme_content = readme_content.replace("{DATE}", &datetime);
        readme_content = readme_content.replace("{date}", &datetime);
        
        // Подстановка значений динамических полей
        for (field_id, value) in dynamic_fields {
            let placeholder = format!("{{{}}}", field_id.to_uppercase());
            readme_content = readme_content.replace(&placeholder, value);
            
            let placeholder_lower = format!("{{{}}}", field_id.to_lowercase());
            readme_content = readme_content.replace(&placeholder_lower, value);
        }
        
        // Добавить заголовок и дату в начало README
        let full_readme = format!(
            "# {}\n\nСоздано: {}\n\n## Что дальше\n{}",
            project_name,
            datetime,
            readme_content
        );
        
        let mut readme_file = fs::File::create(&readme_path)
            .map_err(|e| format!("Failed to create README {:?}: {}", readme_path, e))?;
        
        readme_file.write_all(full_readme.as_bytes())
            .map_err(|e| format!("Failed to write README: {}", e))?;
    }
    
    log_lines.push("Project created successfully!".to_string());
    Ok(log_lines)
}

