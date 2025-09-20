# Excel → CSV Converter

Быстрый конвертер Excel (XLSX) → CSV. Поддерживает режимы: pandas (по умолчанию) и потоковый (openpyxl) для больших файлов.

## Установка
- Python 3.10+
- Опционально: `pip install pandas openpyxl`

## Примеры

Конкретный файл:
```bash
python3 excel_to_csv/code/convert_xlsx_to_csv.py \
  --input excel_to_csv/data_in/shopify_orders_20250918_202022.xlsx \
  --out   excel_to_csv/data_out/shopify_orders_20250918_202022.csv \
  --force
```

Общий случай (pandas):
```bash
python3 excel_to_csv/code/convert_xlsx_to_csv.py \
  --input path/to/file.xlsx \
  --sheet Sheet1 \
  --delimiter , \
  --encoding utf-8
```

Большие файлы (streaming):
```bash
python3 excel_to_csv/code/convert_xlsx_to_csv.py \
  --input path/to/big.xlsx \
  --streaming \
  --out path/to/big.csv \
  --force
```
