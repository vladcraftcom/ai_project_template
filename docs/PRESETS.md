# Работа с пресетами

Подробное руководство по созданию, настройке и использованию пресетов.

## 📑 Содержание

1. [Что такое пресет](#что-такое-пресет)
2. [Структура пресета](#структура-пресета)
3. [Файл конфигурации](#файл-конфигурации)
4. [Создание кастомного пресета](#создание-кастомного-пресета)
5. [Добавление пресета в GitHub](#добавление-пресета-в-github)
6. [Примеры пресетов](#примеры-пресетов)

## 🎯 Что такое пресет

**Пресет** — это набор конфигураций, который определяет:
- Структуру создаваемого проекта (директории и файлы)
- Шаблоны файлов, которые будут скопированы
- Пустые файлы, которые будут созданы
- Шаблон README.md
- Динамические поля и опции для UI

Каждый пресет находится в отдельной директории и содержит файл `files_config.json` с конфигурацией.

## 📁 Структура пресета

Базовая структура пресета:

```
preset_name/
├── files_config.json          # Конфигурация пресета (обязательный)
├── template1.md                # Файлы-шаблоны (опционально)
├── template2.md
└── ...
```

### Обязательные файлы

- **`files_config.json`**: JSON файл с полной конфигурацией пресета

### Опциональные файлы

- **Файлы-шаблоны**: Любые файлы, которые должны быть скопированы в создаваемый проект
- **Дополнительные конфигурации**: Любые другие файлы, специфичные для пресета

## 📝 Файл конфигурации

### Структура `files_config.json`

```json
{
  "preset_id": "unique_preset_id",
  "preset_name": "Human Readable Name",
  "description": "Описание пресета",
  "directories": ["dir1", "dir2"],
  "templates": [
    {
      "source": "template_file.md",
      "destination": "destination_file.md"
    }
  ],
  "empty_files": ["file1.txt", "file2.md"],
  "readme_template": "Шаблон README с {placeholders}",
  "fields": [
    {
      "id": "field_id",
      "label": "Field Label",
      "required": false,
      "type": "text",
      "description": "Field description"
    }
  ],
  "options": [
    {
      "id": "option_id",
      "label": "Option Label",
      "default": false,
      "description": "Option description"
    }
  ]
}
```

### Описание полей конфигурации

#### Основные поля

- **`preset_id`** (строка): Уникальный идентификатор пресета. Обычно совпадает с именем директории. Должен быть уникальным среди всех пресетов.
- **`preset_name`** (строка): Отображаемое имя пресета в UI.
- **`description`** (строка): Описание пресета (пока не используется в UI, но полезно для документации).

#### Структура проекта

- **`directories`** (массив строк): Список директорий, которые будут созданы в проекте.
  - Пример: `["code", "data", "docs"]`
  
- **`templates`** (массив объектов): Файлы-шаблоны из директории пресета, которые будут скопированы в проект.
  - `source`: Имя файла-источника в директории пресета
  - `destination`: Имя файла-назначения в создаваемом проекте
  
- **`empty_files`** (массив строк): Список пустых файлов, которые будут созданы в корне проекта.
  - Пример: `["plan.md", "notes.txt"]`

#### README шаблон

- **`readme_template`** (строка): Шаблон для README.md. Поддерживает подстановки:
  - `{project_name}` или `{PROJECT_NAME}` - имя проекта
  - `{datetime}` или `{DATE}` - дата и время создания
  - `{field_id}` - значения динамических полей (регистр не важен)

#### Динамические поля

- **`fields`** (массив объектов): Поля ввода в UI.
  
  **Поля объекта FieldConfig:**
  - `id` (строка): Уникальный идентификатор поля
  - `label` (строка): Метка поля в UI
  - `required` (boolean): Обязательно ли заполнение (пока не реализовано в валидации)
  - `type` (строка): Тип поля - `"text"` или `"select"`
  - `options` (массив строк, опционально): Для типа `"select"` - список опций
  - `description` (строка, опционально): Описание поля

#### Опции

- **`options`** (массив объектов): Чекбоксы в UI.
  
  **Поля объекта OptionConfig:**
  - `id` (строка): Уникальный идентификатор опции
  - `label` (строка): Метка опции (текст чекбокса)
  - `default` (boolean): Значение по умолчанию
  - `description` (строка, опционально): Описание опции

## 🛠️ Создание кастомного пресета

### Шаг 1: Создание директории

Создайте директорию с именем пресета в директории пресетов:

```bash
mkdir ~/Documents/ai_prompt_presets/my_custom_preset
```

### Шаг 2: Создание файла конфигурации

Создайте файл `files_config.json` в директории пресета:

```json
{
  "preset_id": "my_custom_preset",
  "preset_name": "My Custom Preset",
  "description": "Описание моего кастомного пресета",
  "directories": ["src", "tests", "docs"],
  "templates": [
    {
      "source": "my_template.md",
      "destination": "template.md"
    }
  ],
  "empty_files": ["notes.md", "todo.txt"],
  "readme_template": "# {project_name}\n\nСоздано: {datetime}\n\n## Описание\n\nЭто кастомный пресет.",
  "fields": [],
  "options": []
}
```

### Шаг 3: Добавление шаблонов

Создайте файлы-шаблоны в директории пресета:

```bash
# Пример
echo "# My Template" > ~/Documents/ai_prompt_presets/my_custom_preset/my_template.md
```

### Шаг 4: Обновление списка пресетов

Нажмите кнопку **"Refresh Presets"** в приложении, чтобы новый пресет появился в списке.

### Пример полного пресета

См. существующие пресеты в директории пресетов:
- `software/files_config.json` - пример пресета для разработки ПО
- `book/files_config.json` - пример пресета для написания книг

## 📤 Добавление пресета в GitHub

Если вы хотите поделиться своим пресетом:

1. **Форкните репозиторий**: `https://github.com/vladcraftcom/ai_prompt_presets`
2. **Добавьте ваш пресет** в форк
3. **Создайте Pull Request** в основной репозиторий

### Структура в GitHub репозитории

```
ai_prompt_presets/
├── software/
│   ├── files_config.json
│   └── ...
├── book/
│   ├── files_config.json
│   └── ...
└── your_preset/
    ├── files_config.json
    └── ...
```

## 📚 Примеры пресетов

### Пример 1: Минимальный пресет

```json
{
  "preset_id": "minimal",
  "preset_name": "Minimal Preset",
  "description": "Минимальный пример пресета",
  "directories": [],
  "templates": [],
  "empty_files": ["README.md"],
  "readme_template": "# {project_name}\n\nСоздано: {datetime}",
  "fields": [],
  "options": []
}
```

### Пример 2: Пресет с полями

```json
{
  "preset_id": "with_fields",
  "preset_name": "Preset with Fields",
  "description": "Пресет с динамическими полями",
  "directories": ["src"],
  "templates": [],
  "empty_files": [],
  "readme_template": "# {project_name}\n\nAuthor: {author}\nLicense: {license}",
  "fields": [
    {
      "id": "author",
      "label": "Author name",
      "required": true,
      "type": "text",
      "description": "Имя автора проекта"
    },
    {
      "id": "license",
      "label": "License",
      "required": false,
      "type": "select",
      "options": ["MIT", "Apache-2.0", "GPL-3.0", "Proprietary"],
      "description": "Лицензия проекта"
    }
  ],
  "options": []
}
```

### Пример 3: Пресет с опциями

```json
{
  "preset_id": "with_options",
  "preset_name": "Preset with Options",
  "description": "Пресет с опциями",
  "directories": ["src"],
  "templates": [],
  "empty_files": ["config.json"],
  "readme_template": "# {project_name}",
  "fields": [],
  "options": [
    {
      "id": "init_git",
      "label": "Initialize Git repository",
      "default": true,
      "description": "Инициализировать Git репозиторий"
    },
    {
      "id": "create_docs",
      "label": "Create documentation folder",
      "default": false,
      "description": "Создать папку для документации"
    }
  ]
}
```

## 🔍 Валидация конфигурации

Приложение автоматически валидирует конфигурацию при загрузке пресета:

- ✅ Проверяется наличие файла `files_config.json`
- ✅ Проверяется валидность JSON
- ✅ Проверяется соответствие структуре `PresetConfig`
- ❌ При ошибке в логах появится сообщение об ошибке

## 💡 Рекомендации

### Именование

- Используйте `snake_case` для `preset_id`
- Делайте `preset_id` коротким и понятным
- Используйте понятные имена для директорий и файлов

### Организация

- Группируйте связанные шаблоны
- Используйте описательные имена для полей и опций
- Добавьте описания к полям и опциям для лучшего UX

### Шаблоны

- Используйте понятные placeholder'ы в README шаблонах
- Поддерживайте консистентность в именовании файлов
- Документируйте назначение каждого шаблона

---

*Следующие разделы: [Архитектура](ARCHITECTURE.md) | [API Документация](API.md)*

