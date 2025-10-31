
//! # AI Project Template - GUI Application
//!
//! Графическое приложение для создания проектов на основе настраиваемых пресетов.
//! Позволяет генерировать структуру проектов с шаблонами, промптами и конфигурациями
//! для различных типов проектов (разработка ПО, написание книг и т.д.).
//!
//! ## Основные возможности
//!
//! - Динамическая загрузка пресетов из внешнего репозитория GitHub
//! - Настраиваемая структура проектов через JSON конфигурации
//! - Кроссплатформенные системные уведомления (Windows, macOS, Linux)
//! - Поддержка динамических полей и опций для каждого пресета
//!
//! ## Архитектура
//!
//! Приложение использует фреймворк [Iced](https://github.com/iced-rs/iced) для GUI и
//! следует архитектуре Model-View-Update (MVU).
//!
//! - `AppState` - состояние приложения
//! - `Msg` - сообщения для обновления состояния
//! - `presets` - модуль для работы с конфигурациями пресетов
//! - `command` - модуль для создания проектов

mod presets;
mod command;

use iced::theme::{self, Theme};
use iced::widget::{button, checkbox, column, container, pick_list, progress_bar, row, scrollable, text, text_input};
use iced::{Application, Command, Element, Length, Settings, Subscription};
use std::time::Instant;
use std::path::PathBuf;
use std::collections::HashMap;
use presets::*;
use command::*;
use notify_rust::Notification;

/// Сообщения для обновления состояния приложения (MVU паттерн)
#[derive(Clone, Debug)]
enum Msg {
    /// Изменено имя проекта
    NameChanged(String),
    /// Выбран пресет из списка доступных
    PresetSelected(Option<String>),
    /// Изменено значение динамического поля пресета
    FieldChanged(String, String), // field_id, value
    /// Переключена опция пресета
    OptionToggled(String, bool), // option_id, enabled
    /// Запрошено создание проекта
    Create,
    /// Завершено выполнение операции создания проекта
    ProcessFinished { 
        /// Строки лога выполнения операции
        lines: Vec<String>, 
        /// Успешно ли завершена операция
        success: bool 
    },
    /// Обновить прогресс диалога (для анимации)
    Tick,
    /// Выбрана директория для установки пресетов
    PresetsPathSelected(Option<PathBuf>),
    /// Завершена загрузка пресетов из GitHub
    PresetsDownloaded(Result<PathBuf, String>),
    /// Загружен список доступных пресетов
    PresetsLoaded(Result<Vec<String>, String>),
    /// Загружена конфигурация выбранного пресета
    PresetConfigLoaded(Result<PresetConfig, String>),
    /// Обновить список доступных пресетов (загрузить заново из GitHub)
    RefreshPresets,
}

/// Основное состояние приложения
///
/// Хранит все данные, необходимые для работы GUI, включая:
/// - состояние пресетов и их конфигурации
/// - введенные пользователем данные
/// - состояние UI (диалоги, прогресс, логи)
#[derive(Debug)]
struct AppState {
    // Пресеты
    presets_dir: Option<PathBuf>,
    available_presets: Vec<String>, // preset_id
    preset_names: HashMap<String, String>, // preset_id -> preset_name (для отображения)
    preset_display_names: Vec<String>, // Список имен для отображения (синхронизирован с available_presets)
    selected_preset: Option<String>, // preset_id
    selected_preset_display_name: Option<String>, // Имя выбранного пресета для отображения в UI
    preset_config: Option<PresetConfig>,
    dynamic_fields: HashMap<String, String>, // field_id -> value
    dynamic_options: HashMap<String, bool>, // option_id -> enabled
    
    // Проект
    project_name: String,
    
    // UI состояние
    project_name_error: String,
    is_busy: bool,
    log_lines: Vec<String>,
    min_busy_ms: u64,
    show_dialog: bool,
    dialog_progress: f32,
    dialog_start: Option<Instant>,
    
    // Инициализация
    presets_initialized: bool,
}

impl AppState {
    /// Проверить, можно ли создать проект в текущий момент
    ///
    /// Проект можно создать если:
    /// - приложение не занято выполнением другой операции
    /// - введено корректное имя проекта
    /// - выбран и загружен пресет
    /// - задана директория с пресетами
    ///
    /// # Returns
    ///
    /// `true` если все условия выполнены, иначе `false`
    fn can_create(&self) -> bool {
        !self.is_busy
            && !self.project_name.trim().is_empty()
            && is_valid_project_name(&self.project_name)
            && self.preset_config.is_some()
            && self.presets_dir.is_some()
    }
}

impl Application for AppState {
    type Executor = iced::executor::Default;
    type Message = Msg;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        let mut state = Self {
            // Пресеты
            presets_dir: None,
            available_presets: Vec::new(),
            preset_names: HashMap::new(),
            preset_display_names: Vec::new(),
            selected_preset: None,
            selected_preset_display_name: None,
            preset_config: None,
            dynamic_fields: HashMap::new(),
            dynamic_options: HashMap::new(),
            
            // Проект
                project_name: String::new(),
            
            // UI состояние
                project_name_error: String::new(),
                is_busy: false,
                log_lines: Vec::new(),
                min_busy_ms: 2000,
                show_dialog: false,
                dialog_progress: 0.0,
                dialog_start: None,
            
            // Инициализация
            presets_initialized: false,
        };
        
        // Попытаться загрузить путь к пресетам
        let presets_dir = load_presets_path_from_global_namespace();
        
        if let Some(dir) = presets_dir {
            // Путь найден - загрузить пресеты
            state.presets_dir = Some(dir.clone());
            (
                state,
                Command::perform(async move {
                    discover_presets(&dir).map_err(|e| e.to_string())
                }, |result| Msg::PresetsLoaded(result))
            )
        } else {
            // Путь не найден - запросить выбор папки
            let default_path = get_default_presets_path();
            (
                state,
                Command::perform(async move {
                    // Открыть диалог выбора папки
                    rfd::AsyncFileDialog::new()
                        .set_directory(&default_path)
                        .pick_folder()
                        .await
                        .map(|folder| folder.path().to_path_buf())
                }, |path| Msg::PresetsPathSelected(path))
            )
        }
    }

    /// Заголовок окна приложения
    fn title(&self) -> String { 
        "Project Creator".into() 
    }
    
    /// Тема оформления приложения
    fn theme(&self) -> Theme { 
        theme::Theme::Dark 
    }

    /// Подписка на периодические события
    ///
    /// Используется для обновления прогресс-бара диалога во время выполнения операций.
    /// Обновление происходит каждые 50 мс пока активен диалог.
    fn subscription(&self) -> Subscription<Self::Message> {
        if self.show_dialog {
            iced::time::every(std::time::Duration::from_millis(50)).map(|_| Msg::Tick)
        } else {
            Subscription::none()
        }
    }

    /// Обработать сообщение и обновить состояние приложения
    ///
    /// Это центральная функция паттерна MVU. Она обрабатывает все события пользователя
    /// и асинхронные операции, возвращая команды для выполнения дополнительных действий.
    ///
    /// # Arguments
    ///
    /// * `message` - сообщение для обработки
    ///
    /// # Returns
    ///
    /// Команда для выполнения асинхронных операций или `Command::none()` если синхронной обработки достаточно
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Msg::NameChanged(s) => {
                self.project_name = s;
                self.project_name_error = if is_valid_project_name(&self.project_name) { String::new() } else { "Invalid name".into() };
            }
            Msg::PresetSelected(preset_id) => {
                self.selected_preset = preset_id.clone();
                // Обновить отображаемое имя выбранного пресета
                self.selected_preset_display_name = preset_id.as_ref()
                    .and_then(|id| self.preset_names.get(id).cloned());
                
                if let Some(id) = preset_id {
                    if let Some(dir) = &self.presets_dir {
                        let dir = dir.clone();
                        self.log_lines.push(format!("Loading preset config: {} from {:?}", id, dir));
                        return Command::perform(async move {
                            load_preset_config(&dir, &id).map_err(|e| e.to_string())
                        }, |result| Msg::PresetConfigLoaded(result));
                    }
                } else {
                    self.preset_config = None;
                    self.dynamic_fields.clear();
                    self.dynamic_options.clear();
                }
            }
            Msg::FieldChanged(field_id, value) => {
                self.dynamic_fields.insert(field_id, value);
            }
            Msg::OptionToggled(option_id, enabled) => {
                self.dynamic_options.insert(option_id, enabled);
            }
            Msg::PresetsPathSelected(path) => {
                if let Some(target_dir) = path {
                    // Скачать и распаковать пресеты
                    return Command::perform(async move {
                        download_and_extract_presets(&target_dir, PRESETS_ZIP_URL).await
                            .map(|_| target_dir)
                            .map_err(|e| e.to_string())
                    }, |result| Msg::PresetsDownloaded(result));
                }
            }
            Msg::PresetsDownloaded(result) => {
                match result {
                    Ok(path) => {
                        // Сохранить путь в глобальное пространство имен
                        if let Err(e) = save_presets_path_to_global_namespace(&path) {
                            self.log_lines.push(format!("Warning: Failed to save presets path: {}", e));
                        }
                        self.presets_dir = Some(path.clone());
                        self.log_lines.push("Presets downloaded successfully. Scanning for available presets...".to_string());
                        // Загрузить список пресетов
                        return Command::perform(async move {
                            discover_presets(&path).map_err(|e| e.to_string())
                        }, |result| Msg::PresetsLoaded(result));
                    }
                    Err(e) => {
                        self.is_busy = false;
                        self.show_dialog = false;
                        self.log_lines.push(format!("Error downloading presets: {}", e));
                    }
                }
            }
            Msg::PresetsLoaded(result) => {
                match result {
                    Ok(presets) => {
                        self.available_presets = presets;
                        // Загрузить имена пресетов для отображения
                        self.preset_names.clear();
                        self.preset_display_names.clear();
                        if let Some(ref presets_dir) = self.presets_dir {
                            for preset_id in &self.available_presets {
                                let display_name = presets::get_preset_display_name(presets_dir, preset_id);
                                self.preset_names.insert(preset_id.clone(), display_name.clone());
                                self.preset_display_names.push(display_name);
                            }
                        }
                        self.presets_initialized = true;
                        self.is_busy = false;
                        self.show_dialog = false;
                        self.log_lines.push(format!("Found {} preset(s)", self.available_presets.len()));
                        // Выбрать первый пресет по умолчанию (или "software" если есть)
                        if let Some(software_idx) = self.available_presets.iter().position(|p| p == "software") {
                            let preset_id = self.available_presets[software_idx].clone();
                            return self.update(Msg::PresetSelected(Some(preset_id)));
                        } else if !self.available_presets.is_empty() {
                            let preset_id = self.available_presets[0].clone();
                            return self.update(Msg::PresetSelected(Some(preset_id)));
                        }
                    }
                    Err(e) => {
                        self.is_busy = false;
                        self.show_dialog = false;
                        self.log_lines.push(format!("Error loading presets: {}", e));
                    }
                }
            }
            Msg::PresetConfigLoaded(result) => {
                match result {
                    Ok(config) => {
                        self.preset_config = Some(config.clone());
                        self.log_lines.push(format!(
                            "Preset loaded: {} (fields: {}, options: {})",
                            config.name,
                            config.fields.len(),
                            config.options.len()
                        ));
                        // Инициализировать опции из конфига
                        for opt in &config.options {
                            self.dynamic_options.insert(
                                opt.id.clone(),
                                opt.default,
                            );
                        }
                    }
                    Err(e) => {
                        self.log_lines.push(format!("Error loading preset config: {}", e));
                    }
                }
            }
            Msg::RefreshPresets => {
                if let Some(ref dir) = self.presets_dir {
                    let dir = dir.clone();
                    self.is_busy = true;
                    self.log_lines.push("Downloading and updating presets from GitHub...".to_string());
                    self.show_dialog = true;
                    self.dialog_progress = 0.0;
                    self.dialog_start = Some(Instant::now());
                    return Command::perform(async move {
                        download_and_extract_presets(&dir, PRESETS_ZIP_URL).await
                            .map(|_| dir)
                            .map_err(|e| e.to_string())
                    }, |result| {
                        match result {
                            Ok(dir) => Msg::PresetsDownloaded(Ok(dir)),
                            Err(e) => Msg::PresetsDownloaded(Err(e)),
                        }
                    });
                } else {
                    self.log_lines.push("No presets directory set".to_string());
                }
            }
            Msg::Create => {
                if !self.can_create() { return Command::none(); }
                
                let preset_config = self.preset_config.clone().unwrap();
                let presets_dir = self.presets_dir.clone().unwrap();
                let project_name = self.project_name.clone();
                let dynamic_fields = self.dynamic_fields.clone();
                let dynamic_options = self.dynamic_options.clone();
                
                // Определить путь к проекту (текущая директория)
                let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                let project_path = current_dir.join(&project_name);
                
                self.is_busy = true;
                self.log_lines.clear();
                self.show_dialog = true;
                self.dialog_progress = 0.0;
                self.dialog_start = Some(Instant::now());
                
                return Command::perform(async move {
                    match create_project(
                        &project_path,
                        &presets_dir,
                        &preset_config,
                        &project_name,
                        &dynamic_fields,
                        &dynamic_options,
                    ) {
                        Ok(lines) => (lines, true),
                        Err(e) => (vec![format!("Error: {}", e)], false),
                    }
                }, |(lines, success)| Msg::ProcessFinished { lines, success });
            }
            Msg::ProcessFinished { lines, success } => {
                for l in lines { self.log_lines.push(l); }
                if success {
                    self.log_lines.push("Project created successfully!".to_string());
                    // Отправить системное уведомление
                    let project_name = self.project_name.clone();
                    send_notification(&project_name, success);
                } else {
                    self.log_lines.push("Project creation failed!".to_string());
                    // Отправить уведомление об ошибке
                    let project_name = self.project_name.clone();
                    send_notification(&project_name, success);
                }
                self.is_busy = false;
            }
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

    /// Построить UI представление текущего состояния
    ///
    /// Создает иерархию виджетов Iced на основе текущего состояния приложения.
    /// UI динамически адаптируется в зависимости от выбранного пресета.
    ///
    /// # Returns
    ///
    /// Корневой элемент UI дерева
    fn view(&self) -> Element<Self::Message> {
        // Выбор пресета - показываем человекочитаемые имена
        let preset_selector: Element<Msg> = if !self.available_presets.is_empty() {
            // Создать копию данных для использования в замыкании
            let presets_ids = self.available_presets.clone();
            let preset_display_names = self.preset_display_names.clone();
            
            pick_list(
                preset_display_names.clone(),
                self.selected_preset_display_name.as_ref(),
                move |display_name: String| {
                    // Найти ID по индексу отображаемого имени
                    let idx = preset_display_names.iter()
                        .position(|name| name == &display_name)
                        .unwrap_or(0);
                    
                    // Получим ID по индексу
                    let preset_id = if idx < presets_ids.len() {
                        presets_ids[idx].clone()
                    } else {
                        display_name.clone() // fallback
                    };
                    
                    Msg::PresetSelected(Some(preset_id))
                },
            )
            .width(Length::Fixed(150.0))
            .into()
        } else {
            text("No presets available").size(12).into()
        };
        
        // Кнопка обновления списка пресетов
        let refresh_presets_btn = button("Refresh Presets")
            .on_press(Msg::RefreshPresets)
            .width(Length::Fixed(120.0));
        
        let name = text_input("Project name", &self.project_name)
            .on_input(Msg::NameChanged)
            .width(Length::Fixed(200.0));
        let name_err: Element<Msg> = if !self.project_name_error.is_empty() {
            text(&self.project_name_error).size(11).into()
        } else {
            container(text("")).height(Length::Fixed(0.0)).width(Length::Shrink).into()
        };

        // Динамические поля из конфига пресета
        let mut dynamic_fields_vec: Vec<Element<Msg>> = Vec::new();
        if let Some(ref config) = self.preset_config {
            for field in &config.fields {
                let field_value = self.dynamic_fields.get(&field.id).cloned().unwrap_or_default();
                let field_widget: Element<Msg> = match field.field_type.as_str() {
                    "select" => {
                        if let Some(ref options) = field.options {
                            let field_id_clone = field.id.clone();
                            let field_value_clone = field_value.clone();
                            pick_list(
                                &options[..],
                                if field_value_clone.is_empty() { None } else { Some(field_value_clone.clone()) },
                                move |val| Msg::FieldChanged(field_id_clone.clone(), val.clone()),
                            )
                            .width(Length::Fixed(180.0))
                            .into()
                        } else {
                            text_input(&field.label, &field_value)
                                .on_input(move |val| Msg::FieldChanged(field.id.clone(), val))
                                .width(Length::Fixed(180.0))
                                .into()
                        }
                    }
                    _ => {
                        text_input(&field.label, &field_value)
                            .on_input(move |val| Msg::FieldChanged(field.id.clone(), val))
                            .width(Length::Fixed(180.0))
                            .into()
                    }
                };
                dynamic_fields_vec.push(field_widget);
            }
        }
        let dynamic_fields_empty = dynamic_fields_vec.is_empty();
        let dynamic_fields = if !dynamic_fields_empty {
            let mut col = column![];
            for widget in dynamic_fields_vec {
                col = col.push(widget);
            }
            col.spacing(4)
        } else {
            column![]
        };

        // Динамические опции из конфига пресета
        let mut dynamic_opts_vec: Vec<Element<Msg>> = Vec::new();
        if let Some(ref config) = self.preset_config {
            for opt in &config.options {
                let opt_enabled = self.dynamic_options.get(&opt.id).copied().unwrap_or(opt.default);
                let opt_msg = opt.id.clone();
                dynamic_opts_vec.push(
                    checkbox(&opt.label, opt_enabled)
                        .on_toggle(move |v| Msg::OptionToggled(opt_msg.clone(), v))
                        .into()
                );
            }
        }
        let dynamic_opts_empty = dynamic_opts_vec.is_empty();
        let dynamic_opts = if !dynamic_opts_empty {
            let mut col = column![];
            for widget in dynamic_opts_vec {
                col = col.push(widget);
            }
            col.spacing(3)
        } else {
            column![]
        };

        let create_btn = if self.can_create() {
            button("Create project").on_press(Msg::Create)
                .width(Length::Fixed(130.0))
        } else {
            button("Create project").width(Length::Fixed(130.0))
        };

        let log = scrollable(text(self.log_lines.join("\n")).size(11))
            .height(Length::Fixed(80.0));

        let dialog: Element<Msg> = if self.show_dialog {
            container(
                column![
                    text("Processing...").size(14),
                    progress_bar(0.0..=1.0, self.dialog_progress),
                    text(format!("{:.0}%", self.dialog_progress * 100.0)).size(11)
                ]
                .spacing(4)
            )
            .padding(8)
            .into()
        } else { container(column![]).into() };

        container(column![
            text("Project Creator").size(16),
            row![ 
                text("Preset:").width(Length::Fixed(80.0)).size(12), 
                preset_selector,
                refresh_presets_btn,
            ].spacing(6),
            row![ 
                text("Project name:").width(Length::Fixed(80.0)).size(12), 
                column![name, name_err].spacing(2).width(Length::Shrink),
                create_btn,
            ].spacing(6),
            if !dynamic_fields_empty {
                column![
                    text("Fields:").size(12),
                    dynamic_fields,
                ].spacing(3)
            } else {
                column![]
            },
            if !dynamic_opts_empty {
                column![
                    text("Options:").size(12),
                    dynamic_opts,
                ].spacing(3)
            } else {
                column![]
            },
            dialog,
            text("Log").size(12),
            log,
        ].spacing(6).padding(10))
        .into()
    }
}

/// Точка входа в приложение
///
/// Инициализирует и запускает главный цикл приложения Iced.
/// Использует Tokio runtime для асинхронных операций (загрузка пресетов, создание проектов).
#[tokio::main]
async fn main() -> iced::Result {
    AppState::run(Settings::default())
}

/// Проверить валидность имени проекта
///
/// Имя проекта должно соответствовать следующим правилам:
/// - Начинаться с буквы или цифры
/// - Содержать только буквы, цифры, точки, подчеркивания и дефисы
/// - Длина от 1 до 64 символов
/// - Не заканчиваться точкой или пробелом
/// - Не быть зарезервированным именем Windows (CON, PRN, AUX, NUL, COM1-9, LPT1-9)
///
/// # Arguments
///
/// * `name` - строка с именем проекта для проверки
///
/// # Returns
///
/// `true` если имя валидно, иначе `false`
///
/// # Examples
///
/// ```
/// assert!(is_valid_project_name("my_project"));
/// assert!(is_valid_project_name("test-123"));
/// assert!(!is_valid_project_name("CON")); // зарезервированное имя Windows
/// assert!(!is_valid_project_name("")); // пустое имя
/// ```
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

/// Отправить системное уведомление о результате создания проекта
///
/// Использует кроссплатформенную библиотеку `notify-rust` для показа
/// системных уведомлений с автоматической поддержкой звуков.
///
/// # Платформенные особенности
///
/// - **Windows**: Toast уведомление в правом нижнем углу с системным звуком
/// - **macOS**: Уведомление в Центре уведомлений (Notification Center) со звуком
/// - **Linux**: Desktop Notification через DBus со звуком (требует сервер уведомлений)
///
/// # Arguments
///
/// * `project_name` - имя созданного проекта для отображения в уведомлении
/// * `success` - `true` если проект создан успешно, `false` при ошибке
///
/// # Note
///
/// Ошибки показа уведомлений логируются в stderr, но не прерывают работу приложения.
/// На macOS может потребоваться разрешение на уведомления в системных настройках.
fn send_notification(project_name: &str, success: bool) {
    let notification = if success {
        Notification::new()
            .summary("Project Created")
            .body(&format!("Project '{}' has been created successfully!", project_name))
            .appname("AI Project Template")
            .finalize()
    } else {
        Notification::new()
            .summary("Project Creation Failed")
            .body(&format!("Failed to create project '{}'", project_name))
            .appname("AI Project Template")
            .finalize()
    };
    
    // Попытка показать уведомление
    // На Windows: покажет всплывающее уведомление с системным звуком
    // На macOS: покажет уведомление в Центре уведомлений со звуком
    // На Linux: покажет уведомление через DBus со звуком
    // Игнорируем ошибки если система не поддерживает уведомления
    if let Err(e) = notification.show() {
        eprintln!("Failed to show notification: {}", e);
        // На macOS может потребоваться разрешение на уведомления в системных настройках
        // На Linux должен быть установлен сервер уведомлений (например, notify-osd)
    }
}
