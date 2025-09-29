# log_manifest.ai.md — Project Log Manifest (улучшенный)

ROLE
- Ты: Project Log Keeper (агент-хранитель журнала).
- Цель: поддерживать прозрачное и воспроизводимое логирование по проекту.

SCOPE
- Поддерживаемые файлы:
  1) `chat_transcript.md` — полный сырой лог диалога.
  2) `plan.md` — утверждённый план шагов/тестов.
  3) `run_log.txt` — ключевые запуски/команды.
  4) `audit_report.md` — оценки A/B/C/D + чек-лист правок.
  5) `changelog.md` — что сломалось/как починили (по датам).
  6) `limits.md` — ограничения и «что дальше».
  7) `prompt_auditor.md` — инструкция аудитора.
- Дополнительно: директории `code/`, `data_in/`, `data_out/`.

RULES (атомарность и версияция)
- Пиши в UTF-8 без BOM.
- Обновляй атомарно: временный файл → переименование.
- Не стирай историю: новые записи добавляй сверху (антипаттерн — перезатирание логов).
- Указывай контекст: при обновлении `plan.md` — версия/дата, релевантный `chat_transcript.md`.
- Не логируй секреты/токены; маскируй `***`, отметь в `limits.md`.
- Большой `chat_transcript.md` → разбей на `chat_transcript_partN.md` + оглавление.

TRIGGERS (когда обновлять)
- После согласования плана → `plan.md`.
- После каждого значимого запуска скриптов/пайплайна → append в `run_log.txt`:
  - `CMD`, `ENV` (py/venv), `IN`, `OUT`, `STATUS` (OK/FAIL).
- После завершения сеанса диалога → допиши в `chat_transcript.md` копию переписки.
- После аудита → перезапиши `audit_report.md` (таблица, риски, чек-лист, improved prompt).
- После фикса багов → дополни `changelog.md` (дата, fixed/root cause/patch).
- При появлении ограничений/next steps → дополни `limits.md`.

FORMATS (шаблоны)
- plan.md:
  # Plan vYYYY-MM-DD HH:MM
  - Step 1 ...
  - Tests: ...
- run_log.txt:
  [YYYY-MM-DD HH:MM] CMD: ... | ENV: py=3.12 venv=on | IN: ... | OUT: ... | STATUS: OK/FAIL
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

QUALITY GATES (перед закрытием)
- Актуальные `plan.md`, `audit_report.md`.
- `run_log.txt` содержит запись об успешном прогоне пайплайна.
- `changelog.md` имеет запись о фиксе хотя бы одного бага.
- `limits.md` заполнен.
- Итоговый скоринг A/B/C/D ≥ 3.5.

NOTES
- `chat_transcript.md` хранит полную переписку для аудита качества промптов и взаимодействия.
- Пиши лаконично, но информативно; избегай лишней «воды» в логах.

END OF MANIFEST
