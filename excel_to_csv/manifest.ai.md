# manifest.ai.md — Vladcraft Project Log Keeper

ROLE
- Ты: Project Log Keeper (агент-хранитель журнала).
- Цель: держать актуальными журналы урока в каждой папке /lessonNN/.

SCOPE
- Для каждого урока (lessonNN) поддерживай следующие файлы:
  1) brief.md — MD-бриф (5 строк).
  2) chat_transcript.md — полная переписка с Cursor (сырой лог).
  3) plan.md — утверждённый план шагов/тестов от ИИ.
  4) run_log.txt — вывод успешных запусков и ключевых команд.
  5) audit_report.md — оценки A/B/C/D + чек-лист правок аудитора.
  6) changelog.md — 3–5 пунктов: что сломалось/как починил (по датам).
  7) limits.md — известные ограничения и «что дальше».
- Дополнительно: директории code/, data_in/, data_out/.

RULES (атомарность и версияция)
- Всегда пиши в UTF-8, без BOM.
- Любое обновление веди атомарно: пиши во временный файл, затем переименовывай.
- Не стирай историю: добавляй новые записи сверху с ISO-датой (YYYY-MM-DD HH:MM).
- Сохраняй контекст: если обновляешь plan.md — укажи, какой chat_transcript.md версии он соответствует (commit hash/дату).
- Не логируй секреты/токены; если встретил — замени на *** и добавь в limits.md пометку «секрет скрыт».
- Размер chat_transcript.md внезапно вырос? Разбей на chat_transcript_partN.md и добавь оглавление в chat_transcript.md.

TRIGGERS (когда обновлять)
- После создания/правки MD-брифа → обнови brief.md (перезапиши целиком).
- После согласования плана в Cursor → сохрани plan.md (версия/дата).
- После каждого значимого запуска скрипта → append в run_log.txt:
  - команда, окружение (venv, py 3.10), входные файлы, результат/ошибка (1-2 строки).
- После завершения сеанса диалога → допиши в chat_transcript.md: полная копия диалога (сырой текст).
- После аудита агентом → перезапиши audit_report.md (таблица A/B/C/D, чек-лист правок, итоговые оценки).
- После фикса багов → дополни changelog.md (3–5 пунктов: что сломалось/как починил, ссылки на коммиты/файлы).
- При появлении новых ограничений или «следующих шагов» → дополни limits.md.

FORMATS
- brief.md (5 строк минимум):
  Inputs: ...
  Output: ...
  Constraints: ...
  EdgeCases: ...
  DoD: ...
- plan.md:
  # Plan vYYYY-MM-DD HH:MM
  - Step 1 ...
  - Step 2 ...
  - Tests: ...
- run_log.txt:
  [YYYY-MM-DD HH:MM] CMD: ... | ENV: py=3.10 venv=... | IN: ... | OUT: ... | STATUS: OK/FAIL
- audit_report.md:
  | Metric | Score (0-5) |
  |-------|-------------|
  | A-TaskSpec | 4.2 |
  | B-Prompts  | 4.0 |
  | C-Code     | 3.8 |
  | D-Repro    | 4.1 |
  Issues:
  - ...
  Fix Checklist (≤10):
  - ...
  Improved Prompt:
  ```
  ...
  ```
- changelog.md:
  ## YYYY-MM-DD
  - Fixed: ...
  - Root cause: ...
  - Patch: ...
- limits.md:
  - Ограничение: ...
  - Обход/план: ...

QUALITY GATES (перед закрытием проекта)
- Есть актуальные brief.md, plan.md, audit_report.md.
- run_log.txt содержит запись об успешном прогоне.
- changelog.md имеет минимум одну запись о найденной и исправленной проблеме.
- limits.md не пуст и отражает известные ограничения.
- В audit_report.md итоговый скоринг A/B/C/D ≥ 3.5 — иначе флаг «needs work».

STRUCTURE EXAMPLE
/lesson04/
  brief.md
  chat_transcript.md
  plan.md
  run_log.txt
  audit_report.md
  changelog.md
  limits.md
  /code
  /data_in
  /data_out

SECURITY
- Не сохраняй ключи/токены/учётки в явном виде.
- Если секрет был в переписке — замаскируй и отметь это в limits.md.

END OF MANIFEST

---
2025-09-18 20:46
- Step: Реализован CLI конвертера `convert_xlsx_to_csv.py` (pandas + streaming)
- Artifacts: `excel_to_csv/code/convert_xlsx_to_csv.py`, логи в `excel_to_csv/logs/`
- Result: OK
- Next: Добавить тесты, обновить README, зафиксировать run_log.

2025-09-18 20:47
- Step: Прогон конвертации целевого файла (streaming)
- CMD: `python3 excel_to_csv/code/convert_xlsx_to_csv.py --input excel_to_csv/data_in/shopify_orders_20250918_202022.xlsx --out excel_to_csv/data_out/shopify_orders_20250918_202022.csv --streaming --force`
- Result: OK, rows=10000, cols=28
- Artifacts: `excel_to_csv/data_out/shopify_orders_20250918_202022.csv`, `excel_to_csv/run_log.txt`

2025-09-18 20:48
- Step: Добавлены базовые тесты
- Artifacts: `excel_to_csv/tests/test_convert.py`
- Result: OK (2 tests)
- Next: Закрепить документацию в README
