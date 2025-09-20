# План разработки: Excel (XLSX) → CSV, универсальный конвертер

Цель: реализовать утилиту командной строки, которая конвертирует произвольные таблицы из XLSX в CSV. Обязательная проверка на конкретном файле: `excel_to_csv/data_in/shopify_orders_20250918_202022.xlsx` → `excel_to_csv/data_out/shopify_orders_20250918_202022.csv`.

Все шаги сопровождаются логированием прогресса в `manifest.ai.md` и записью технических логов в `excel_to_csv/logs/`.

## 1) Требования и допущения
- Python 3.10, виртуальное окружение `.venv` (если есть).
- Разрешены бесплатные библиотеки: `openpyxl`, `pandas` (по умолчанию), без проприетарных.
- Выходной CSV по умолчанию: разделитель `,`, кодировка `utf-8`, окончания строк `LF`, quoting минимальный.
- Обработка дат: если распознаны как даты — выводить ISO 8601 (UTC). Если встречается Excel-сериал — конвертировать в ISO.
- По умолчанию выгружаем все столбцы и строки «как есть». Фильтры/переименования не применяем, если не указано явно.
- Один лист: используем указанный через `--sheet` (имя или индекс) или первый.
- DoD: один запуск командой, корректное завершение, ошибки и процесс логируются в `logs/`.

Логировать после каждого шага в `manifest.ai.md`: описание выполненного шага, артефакты (пути файлов), статус (успех/ошибка), next step.

## 2) Архитектура и интерфейсы
Файлы:
- `excel_to_csv/code/convert_xlsx_to_csv.py` — основной модуль конвертации и CLI.
- `excel_to_csv/code/inspect_xlsx.py` — инспекция XLSX (готово, будем переиспользовать функции при необходимости).
- `excel_to_csv/tests/` — авто-тесты (юнит + e2e с небольшими фиктивными XLSX).

Интерфейсы (псевдокод):
```python
# excel_to_csv/code/convert_xlsx_to_csv.py
from dataclasses import dataclass
from typing import Optional, Union, List

@dataclass
class ConversionOptions:
    input_path: str
    output_path: Optional[str]
    sheet: Optional[Union[str, int]]  # имя или индекс (1-based)
    delimiter: str  # ',', ';', '\t'
    encoding: str  # 'utf-8', 'utf-8-sig'
    header_row: Optional[int]  # 1-based; None = автоопределение первой непустой строки
    line_ending: str  # 'LF'|'CRLF'
    quoting: str  # 'minimal'|'all'|'nonnumeric'|'none'
    decimal: str  # '.' или ',' (только для форматирования чисел при необходимости)
    date_format: str  # 'iso'|'excel_serial'|'auto'
    force: bool  # перезаписывать файл, если существует
    limit_rows: Optional[int]  # ограничение строк (для отладки)
    use_streaming: bool  # True: потоковое чтение openpyxl, False: pandas

@dataclass
class WorkbookInfo:
    sheets: List[str]
    chosen_sheet: str
    header_row_index: Optional[int]
    headers: List[str]

@dataclass
class ConversionReport:
    rows_written: int
    columns_written: int
    output_path: str
```

Ключевые функции:
```python
def detect_workbook_info(input_path: str, sheet: Optional[Union[str, int]], header_row: Optional[int]) -> WorkbookInfo: ...

def convert_with_pandas(opts: ConversionOptions) -> ConversionReport: ...

def convert_streaming_openpyxl(opts: ConversionOptions) -> ConversionReport: ...

def main_cli() -> None:  # argparse, вызов convert_*, обработка ошибок и логов
    ...
```

Логирование: `logging` в файл `excel_to_csv/logs/run_YYYYMMDD_HHMMSS.log` + короткие маркеры прогресса в `manifest.ai.md`.

## 3) Пошаговый план реализации (с логированием в manifest.ai.md)
0. Подготовка окружения
   - Убедиться, что существуют директории: `excel_to_csv/data_in`, `excel_to_csv/data_out`, `excel_to_csv/logs`.
   - Настроить базовый логгер (уровень INFO) с выводом в файл и консоль.
   - Лог в `manifest.ai.md`: «Подготовка окружения завершена». Артефакты: список директорий.

1. CLI и парсинг аргументов (`argparse`)
   - Флаги: `--input` (обяз.), `--out`, `--sheet`, `--delimiter`, `--encoding`, `--header-row`, `--line-ending`, `--quoting`, `--decimal`, `--date-format`, `--force`, `--limit-rows`, `--streaming`.
   - Значения по умолчанию выставить по разделу «Требования и допущения».
   - Если `--out` опущен: формировать путь в `excel_to_csv/data_out/<имя>.csv`.
   - Лог в `manifest.ai.md`: «CLI параметры разобраны». Артефакты: JSON опций.

2. Инспекция книги (повторно-используемая)
   - Использовать `inspect_xlsx` для получения листов, предполагаемой строки заголовков, заголовков.
   - Если пользователь указал `--sheet`, выбрать его. Иначе — первый.
   - Если `--header-row` задан — использовать его; иначе — автоопределение по первой непустой строке.
   - Лог в `manifest.ai.md`: «Инспекция XLSX». Артефакты: лист, индекс заголовков, первые N заголовков.

3. Ветвление: Pandas vs Streaming
   - По умолчанию — `pandas` (`openpyxl` engine). Для больших файлов/ограничений памяти — `--streaming` (openpyxl read-only + построчная запись CSV).
   - Лог в `manifest.ai.md`: «Выбран режим конвертации: pandas/streaming».

4. Реализация: `convert_with_pandas`
   - `pandas.read_excel(..., sheet_name=..., engine='openpyxl', header=header_row-1 или None)`.
   - Если `header=None`, сгенерировать имена колонок `col_1..N` или из автоопределённых заголовков.
   - Обработка дат: 
     - Пытаться `parse_dates='coerce'` (постобработка: `to_datetime`),
     - Явно конвертировать Excel-сериалы в ISO (через `datetime.fromordinal` с поправкой 25569 и время суток — если столбец числовой и распознан как дата).
   - Нормализация чисел: опционально перевод десятичного разделителя (если требуется).
   - Запись CSV: `DataFrame.to_csv(..., index=False, sep=delimiter, encoding=encoding, lineterminator=\n или \r\n, quoting=...)`.
   - Лог в `manifest.ai.md`: «Pandas-конвертация завершена». Артефакты: путь к CSV, кол-во строк/колонок.

5. Реализация: `convert_streaming_openpyxl`
   - `openpyxl.load_workbook(..., read_only=True, data_only=True)`; выбор листа по имени/индексу.
   - Определение `header_row`; генерация заголовков и запись первой строки в CSV.
   - Построчно итерироваться по `iter_rows(values_only=True)`, начиная со следующей строки.
   - Преобразования:
     - Даты: если тип `datetime` — форматировать ISO; если float и это похоже на Excel-сериал (>= 25569, < 60000) — конвертировать.
     - Булевы/числа/строки — безопасно приводить к str без потери.
   - Записывать потоково через `csv.writer` с заданными `delimiter`, `quotechar`, `quoting`.
   - Лог в `manifest.ai.md`: «Streaming-конвертация завершена».

6. Поведение при существующем выходном файле
   - Если файл существует и нет `--force` — выводить понятную ошибку и завершать с ненулевым кодом.
   - С `--force` — перезаписывать.
   - Лог в `manifest.ai.md`: «Проверка существования файла вывода» (действие).

7. Обработка ошибок
   - Валидировать входной путь, лист, доступность чтения, корректность настроек.
   - Любую ошибку логировать в файл и кратко на STDOUT, возвращать код выхода:
     - 2 — файл не найден
     - 3 — некорректная структура XLSX/лист
     - 4 — ошибка записи CSV
     - 1 — прочие
   - Лог в `manifest.ai.md`: «Ошибка/успех шага», краткое описание.

8. Инструкция запуска (генерировать в README и в разделе «Как запустить»)
   - Команды для macOS/Linux; пример с вашим файлом и с общим случаем.
   - Лог в `manifest.ai.md`: «Документация обновлена».

9. Прогон на конкретном файле
   - Запустить конвертацию: `--input excel_to_csv/data_in/shopify_orders_20250918_202022.xlsx --out excel_to_csv/data_out/shopify_orders_20250918_202022.csv`.
   - Проверить, что строки > 0, столбцов = 28.
   - Лог в `manifest.ai.md`: «Конвертация целевого файла завершена».

## 4) Тесты/Проверки
Юнит-тесты (минимальные, с синтетическими XLSX, создаваемыми через `openpyxl`):
1. Чтение листа по имени и индексу: совпадение заголовков и количества строк.
2. Заголовки не с первой строки: указание `--header-row` корректно выбирает заголовки.
3. Даты: Excel-сериал → ISO 8601; `datetime` → ISO 8601.
4. Числа и локали: корректная запись `int` и `float`, независимость от `decimal` (по умолчанию точка).
5. Кавычки/разделители: корректное экранирование при `,`, `;`, `\t` и кавычках в тексте.
6. Большой файл (стримингово): генерируем 50–100k строк, конвертация в streaming-режиме без OOM.
7. Выходной файл уже существует: без `--force` — ошибка; с `--force` — перезапись.
8. Коды выхода: 2/3/4/1 — покрыть негативными сценариями.
9. CLI: парсинг флагов и авто-формирование пути вывода.

Проверки на целевом файле:
- Заголовки и число столбцов (28) соответствуют.
- `quantity` — целые, суммы — числа с плавающей точкой.
- `order_date` конвертирован в ISO (не числовой сериал) — выборочно проверить первые 5 строк.
- Количество строк CSV > 0 (ожидаемо 10 000+).

## 5) Структура проекта (целевое состояние)
```
excel_to_csv/
  code/
    convert_xlsx_to_csv.py
    inspect_xlsx.py
  data_in/
    shopify_orders_20250918_202022.xlsx
  data_out/
    shopify_orders_20250918_202022.csv  # (результат)
  logs/
    run_YYYYMMDD_HHMMSS.log
  plan.md
  questions.md
  README.md
```

## 6) Как запускать (черновик)
- Конкретный файл:
```bash
python3 excel_to_csv/code/convert_xlsx_to_csv.py \
  --input excel_to_csv/data_in/shopify_orders_20250918_202022.xlsx \
  --out   excel_to_csv/data_out/shopify_orders_20250918_202022.csv
```
- Общий случай:
```bash
python3 excel_to_csv/code/convert_xlsx_to_csv.py \
  --input path/to/file.xlsx \
  --sheet Sheet1 \
  --delimiter , \
  --encoding utf-8 \
  --force
```
- Потоковый режим для больших файлов:
```bash
python3 excel_to_csv/code/convert_xlsx_to_csv.py --input big.xlsx --streaming --out big.csv
```

## 7) Next steps (к исполнению)
1. Реализовать `convert_xlsx_to_csv.py` (CLI + pandas-режим).
2. Добавить streaming-режим (openpyxl read-only). Разделить общую логику преобразований (даты, типы).
3. Написать юнит-тесты в `excel_to_csv/tests/` и запустить их.
4. Прогнать на целевом файле, приложить результаты в логи и `manifest.ai.md`.
5. Обновить `README.md` с актуальными командами и возможностями.

После каждого из шагов фиксировать прогресс и результат в `manifest.ai.md` (по правилам проекта).
