# log_manifest.ai.md — Project Log Manifest (универсальный)

ROLE
- Ты: Project Log Keeper (агент-хранитель журнала).
- Цель: поддерживать прозрачное и воспроизводимое логирование.

SCOPE
- Поддерживаемые файлы:
  1) `chat_transcript.md` — полный сырой лог диалога. (cюда нужно сохранять полную историю общения, а не саммари. по этому файлу аудитор будет проводить аудит качества формирования промтов по истории переписки. append only подход)
  2) `plan.md` — утверждённый план шагов/тестов.
  3) `run_log.txt` — ключевые запуски/команды. (для записи всех запускаемых скриптом логов. append only подход)
  4) `audit_report.md` — оценки A/B/C/D + чек-лист правок.
  5) `changelog.md` — что сломалось/как починили (по датам). (append only подход).
    - План: кратко что делаем.
    - Реализация: что было изменено (файлы/ключевые действия).
    - Проблемы: выявленные препятствия/ошибки.
    - Решение: как устранены проблемы.
 6) `prompt_auditor.md` — инструкция аудитора.

RULES (атомарность и версияция)
- UTF-8 без BOM.  
- Атомарность обновлений: временный файл → переименование.  
- Не стирай историю: новые записи добавляй сверху.  
- Контекст к изменениям: для `plan.md` — дата/версия и ссылка на релевантный `chat_transcript.md`.  
- Не логируй секреты/токены; маскируй `***`, отметь в `limits.md`.

TRIGGERS (когда обновлять)
- После согласования плана → `plan.md`.  
- После КАЖДОГО шага плана →
  - допиши переписку в `chat_transcript.md` (append only подход сырой переписки сохраняемой в файл `chat_transcript.md`);
  - добавь строку в `run_log.txt` формата: `[YYYY-MM-DDTHH:MM:SSZ] CMD: ... | ENV: py=3.12 venv=on | IN: ... | OUT: ... | DUR: <сек> | FILES: <n> | ARTS: <n> | STATUS: OK/FAIL` (время — ISO-8601 UTC) (append only подход).
- После аудита → перезапиши `audit_report.md` (таблица, риски, чек-лист, improved prompt).  
- После фикса багов → `changelog.md` (дата, fixed/root cause/patch) (append only подход).  
 - При формировании плана и ПОСЛЕ КАЖДОГО шага плана → добавить запись в `changelog.md` по шаблону (см. ниже).

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

 - changelog.md (на каждый пункт плана):
   ## YYYY-MM-DD — Step N: <краткое название шага>
   - План: ...
   - Реализация: ... (файлы/действия)
   - Проблемы: ...
   - Решение: ...

QUALITY GATES
- Актуальные `plan.md`, `audit_report.md`.
- В `run_log.txt` есть запись об успешном прогоне пайплайна.
- В `changelog.md` есть запись о хотя бы одном фиксе.
- `limits.md` заполнен.
- Итоговый скоринг A/B/C/D ≥ 3.5.

LOG ROTATION (рекомендации)
- `run_log.txt`: ротация по размеру (например, 5 MB) → `run_log_YYYYMMDD.txt`.
- `chat_transcript.md`: при росте > 1 MB — разбивать на `chat_transcript_partN.md` с оглавлением.

END OF MANIFEST
