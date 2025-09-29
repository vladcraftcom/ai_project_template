# excel_to_csv

Создано: 2025-09-29 20:27

## Что дальше
  1) cursor_prompt.md → запонить brief и доп. требования
  2) cursor_prompt.md → скорми агенту этот файл
  3) plan.md → сохранить план от ИИ если он сам не сохранил
  4) code/, run_log.txt, chat_transcript.md → вести по мере работы
  5) audit_report.md / changelog.md / limits.md → после аудита

## Установка и запуск

1. Активируйте venv:
   - Linux/macOS:
     - `source /home/codefather/Documents/ai_project_template/ai_project_template/venv/bin/activate`

2. Установите зависимости для Excel/диаграмм (если не установлены):
   - `pip install openpyxl plotly`

3. Запустите обработку:
   - `python /home/codefather/Documents/ai_project_template/ai_project_template/excel_to_csv/code/process_shopify_orders.py \
      --input /home/codefather/Documents/ai_project_template/ai_project_template/excel_to_csv/data_in/shopify_orders_20250929_202605.xlsx \
      --out_csv /home/codefather/Documents/ai_project_template/ai_project_template/excel_to_csv/data_out/shopify_orders_20250929_202605.csv \
      --out_html /home/codefather/Documents/ai_project_template/ai_project_template/excel_to_csv/data_out/diagram.html`

4. Результат:
   - CSV: `excel_to_csv/data_out/shopify_orders_20250929_202605.csv`
   - Диаграмма (HTML): `excel_to_csv/data_out/diagram.html`
   - Логи: `excel_to_csv/logs/run.log`
