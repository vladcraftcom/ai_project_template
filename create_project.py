#!/usr/bin/env python3
# create_project.py
import argparse, os, sys, shutil, platform, subprocess
from pathlib import Path
from datetime import datetime

# Устанавливаем кодировку UTF-8 для Windows
if platform.system().lower().startswith("win"):
    import codecs
    sys.stdout = codecs.getwriter("utf-8")(sys.stdout.detach())
    sys.stderr = codecs.getwriter("utf-8")(sys.stderr.detach())

ROOT = Path.cwd()

# ======== Встроенные шаблоны (используются, если в корне нет файлов) ========
TEMPLATES = {
    "prompt_cursor_skeleton.md": open("./prompt_cursor_skeleton.md", encoding="utf-8").read(),
    "prompt_auditor.md": open("./prompt_auditor.md", encoding="utf-8").read(),
    "prompt_expert.md": open("./prompt_expert.md", encoding="utf-8").read(),
}


# Базовые пакеты (добавлены pytest и python-dotenv)
PACKAGES = ["pandas", "openpyxl", "aiogram", "google-api-python-client", "pytest", "python-dotenv"]

def ensure_file(path: Path, content: str = ""):
    if not path.exists():
        path.write_text(content, encoding="utf-8")

def copy_or_generate(src_name: str, dst: Path, refresh: bool):
    """
    Копируем шаблон из корня (если есть).
    Если нет — генерим из встроенного TEMPLATES.
    Если refresh=True — перезаписываем dst.
    """
    if dst.exists() and not refresh:
        return
    src = ROOT / src_name
    if src.exists():
        shutil.copy2(src, dst)
    else:
        ensure_file(dst, TEMPLATES.get(src_name, ""))

def venv_paths(base: Path):
    if platform.system().lower().startswith("win"):
        py = base / "venv" / "Scripts" / "python.exe"
        pip = base / "venv" / "Scripts" / "pip.exe"
        activate = base / "venv" / "Scripts" / "activate"
        act_hint = f"{base}\\venv\\Scripts\\activate"
    else:
        py = base / "venv" / "bin" / "python"
        pip = base / "venv" / "bin" / "pip"
        activate = base / "venv" / "bin" / "activate"
        act_hint = f"source {base}/venv/bin/activate"
    return py, pip, activate, act_hint

def run(cmd, cwd=None):
    print(">", " ".join(map(str, cmd)))
    subprocess.check_call(cmd, cwd=cwd)

def main():
    ap = argparse.ArgumentParser(description="Create project scaffold (cross-platform).")
    ap.add_argument("project", help="project name, e.g. project04")
    ap.add_argument("--force", action="store_true", help="fill in missing files if folder exists")
    ap.add_argument("--venv", action="store_true", help="create venv inside project folder")
    ap.add_argument("--install", action="store_true", help="pip install base packages into venv")
    ap.add_argument("--refresh-templates", action="store_true", help="перезаписать шаблоны brief/prompt из корня (или встроенные) поверх существующих")
    args = ap.parse_args()

    project_dir = ROOT / args.project

    # 1) Папки
    if project_dir.exists() and not args.force and any(project_dir.iterdir()):
        print(f"[!] {project_dir} уже существует и не пуст. Добавь --force чтобы дозаполнить.", file=sys.stderr)
        sys.exit(1)
    project_dir.mkdir(parents=True, exist_ok=True)
    for sub in ("code", "data_in", "data_out", "logs"):
        (project_dir / sub).mkdir(parents=True, exist_ok=True)

    # 2) Brief/Prompt
    copy_or_generate("prompt_cursor_skeleton.md", project_dir / "cursor_prompt.md", refresh=args.refresh_templates)
    copy_or_generate("prompt_auditor.md", project_dir / "prompt_auditor.md", refresh=args.refresh_templates)
    copy_or_generate("log_manifest.ai.md", project_dir / "log_manifest.ai.md", refresh=args.refresh_templates)
    copy_or_generate("prompt_expert.md", project_dir / "prompt_expert.md", refresh=args.refresh_templates)

    # 3) Пустые журналы
    for name in ("chat_transcript.md","plan.md","run_log.txt","audit_report.md","changelog.md"):
        ensure_file(project_dir / name)

    # 4) README c краткой шпаргалкой
    summary = (
        "  0) prompt_expert.md → запонить `{ОТРАСЛЬ}` и `{РОЛЬ}` и попросить провести консультацию (не обязательный шаг) \n"
        "  1) cursor_prompt.md → запонить 'Контекст задачи' и доп. требования\n"
        "  2) cursor_prompt.md → скорми агенту этот файл\n"
        "  3) plan.md → сохранить план от ИИ если он сам не сохранил\n"
        "  4) code/, run_log.txt, chat_transcript.md → вести по мере работы\n"
        "  5) audit_report.md / changelog.md  → после аудита\n"
    )

    ensure_file(
        project_dir / "README.md",
        (
            f"# {args.project}\n\n"
            f"Создано: {datetime.now():%Y-%m-%d %H:%M}\n\n"
            "## Что дальше\n"
            f"{summary}"
        ),
    )

    # 5) (Опц.) venv и пакеты
    venv_hint = ""
    if args.venv:
        print("[i] Создаю виртуалку venv …")
        run([sys.executable, "-m", "venv", str(project_dir / "venv")])
        py, pip, activate, act_hint = venv_paths(project_dir)
        venv_hint = f"\nАктивируй окружение:\n  {act_hint}\n"
        if args.install:
            print("[i] Устанавливаю базовые пакеты:", ", ".join(PACKAGES))
            run([str(pip), "install", "--upgrade", "pip"])
            run([str(pip), "install", *PACKAGES])
    else:
        py, pip, act_hint = None, None, None  # не используется

    print("\n" + summary)
    if venv_hint:
        print(venv_hint)

if __name__ == "__main__":
    try:
        main()
    except subprocess.CalledProcessError as e:
        print(f"[!] Команда завершилась с ошибкой: {e}", file=sys.stderr)
        sys.exit(e.returncode)
