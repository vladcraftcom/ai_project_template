#!/usr/bin/env python3
"""
Простой запуск всего пайплайна 🚀

Пакетно обрабатывает все Excel-файлы из excel_to_csv/data_in,
для каждого создаёт отдельные CSV и интерактивную HTML-диаграмму в excel_to_csv/data_out.
"""

from __future__ import annotations

from pathlib import Path
import subprocess
import sys


ROOT = Path("/home/codefather/Documents/ai_project_template/ai_project_template")
PYTHON = ROOT / "venv/bin/python"
PROCESSOR = ROOT / "excel_to_csv/code/process_shopify_orders.py"
IN_DIR = ROOT / "excel_to_csv/data_in"
OUT_DIR = ROOT / "excel_to_csv/data_out"


def main() -> int:
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    excel_files = sorted(IN_DIR.glob("*.xlsx"))
    if not excel_files:
        print(f"Файлы .xlsx не найдены в {IN_DIR}")
        return 0

    overall_rc = 0
    for xlsx in excel_files:
        base = xlsx.stem
        out_csv = OUT_DIR / f"{base}.csv"
        out_html = OUT_DIR / f"{base}.html"
        cmd = [
            str(PYTHON),
            str(PROCESSOR),
            "--input",
            str(xlsx),
            "--out_csv",
            str(out_csv),
            "--out_html",
            str(out_html),
        ]
        print(f"→ Обработка: {xlsx.name} → {out_csv.name}, {out_html.name}")
        try:
            subprocess.run(cmd, check=True)
        except subprocess.CalledProcessError as e:  # noqa: BLE001
            print(f"Ошибка для {xlsx.name}: {e}")
            overall_rc = 1

    return overall_rc


if __name__ == "__main__":
    sys.exit(main())


