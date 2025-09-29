#!/usr/bin/env python3
"""
Universal XLSX -> CSV converter with CLI.

Defaults optimized for Shopify-like exports. Supports two modes:
- pandas (default): fast, convenient for moderate files
- streaming (openpyxl): memory-efficient for very large files

Logs to excel_to_csv/logs and prints concise messages to stdout.
"""

from __future__ import annotations

import argparse
import csv
import json
import logging
import os
import sys
from dataclasses import dataclass
from datetime import datetime, timedelta
from pathlib import Path
from typing import Any, List, Optional, Union


LOGS_DIR = Path("excel_to_csv/logs")
DATA_OUT_DIR = Path("excel_to_csv/data_out")


def setup_logger() -> logging.Logger:
    LOGS_DIR.mkdir(parents=True, exist_ok=True)
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    log_path = LOGS_DIR / f"run_{timestamp}.log"
    logger = logging.getLogger("xlsx2csv")
    logger.setLevel(logging.INFO)
    fmt = logging.Formatter("%(asctime)s [%(levelname)s] %(message)s")
    fh = logging.FileHandler(log_path, encoding="utf-8")
    fh.setFormatter(fmt)
    ch = logging.StreamHandler(sys.stdout)
    ch.setFormatter(fmt)
    logger.handlers = []
    logger.addHandler(fh)
    logger.addHandler(ch)
    logger.info("Logger initialized: %s", log_path)
    return logger


@dataclass
class ConversionOptions:
    input_path: str
    output_path: Optional[str]
    sheet: Optional[Union[str, int]]
    delimiter: str
    encoding: str
    header_row: Optional[int]
    line_ending: str
    quoting: str
    decimal: str
    date_format: str
    force: bool
    limit_rows: Optional[int]
    use_streaming: bool


def parse_args(argv: Optional[List[str]] = None) -> ConversionOptions:
    parser = argparse.ArgumentParser(description="Convert XLSX to CSV")
    parser.add_argument("--input", required=True, help="Path to input .xlsx file")
    parser.add_argument("--out", help="Path to output .csv (default: data_out/<name>.csv)")
    parser.add_argument("--sheet", help="Sheet name or 1-based index", default=None)
    parser.add_argument("--delimiter", default=",", help=", ; or \t")
    parser.add_argument("--encoding", default="utf-8", help="utf-8 or utf-8-sig")
    parser.add_argument("--header-row", type=int, default=None, help="1-based header row index; auto if omitted")
    parser.add_argument("--line-ending", choices=["LF", "CRLF"], default="LF")
    parser.add_argument("--quoting", choices=["minimal", "all", "nonnumeric", "none"], default="minimal")
    parser.add_argument("--decimal", choices=[".", ","], default=".")
    parser.add_argument("--date-format", choices=["iso", "excel_serial", "auto"], default="iso")
    parser.add_argument("--force", action="store_true", help="Overwrite output file if exists")
    parser.add_argument("--limit-rows", type=int, default=None, help="Limit rows for debugging")
    parser.add_argument("--streaming", action="store_true", help="Use streaming (openpyxl) mode")

    ns = parser.parse_args(argv)

    sheet_opt: Optional[Union[str, int]]
    if ns.sheet is None:
        sheet_opt = None
    else:
        try:
            sheet_opt = int(ns.sheet)
        except ValueError:
            sheet_opt = ns.sheet

    out = ns.out
    if not out:
        in_name = Path(ns.input).stem
        DATA_OUT_DIR.mkdir(parents=True, exist_ok=True)
        out = str(DATA_OUT_DIR / f"{in_name}.csv")

    return ConversionOptions(
        input_path=ns.input,
        output_path=out,
        sheet=sheet_opt,
        delimiter="\t" if ns.delimiter == "\\t" else ns.delimiter,
        encoding=ns.encoding,
        header_row=ns.header_row,
        line_ending=ns.line_ending,
        quoting=ns.quoting,
        decimal=ns.decimal,
        date_format=ns.date_format,
        force=ns.force,
        limit_rows=ns.limit_rows,
        use_streaming=ns.streaming,
    )


def csv_quoting_mode(mode: str) -> int:
    if mode == "minimal":
        return csv.QUOTE_MINIMAL
    if mode == "all":
        return csv.QUOTE_ALL
    if mode == "nonnumeric":
        return csv.QUOTE_NONNUMERIC
    return csv.QUOTE_NONE


def ensure_output_writable(path: str, force: bool) -> None:
    if os.path.exists(path) and not force:
        raise FileExistsError(f"Output already exists: {path}. Use --force to overwrite.")
    out_dir = os.path.dirname(path)
    if out_dir:
        os.makedirs(out_dir, exist_ok=True)


def excel_serial_to_datetime(value: float) -> datetime:
    # Excel's epoch starts at 1899-12-30 for Windows (accounting for 1900 leap year bug)
    base = datetime(1899, 12, 30)
    return base + timedelta(days=float(value))


def convert_with_pandas(opts: ConversionOptions, logger: logging.Logger) -> tuple[int, int]:
    try:
        import pandas as pd
    except Exception as e:
        raise RuntimeError("pandas is required for non-streaming mode. Install pandas and openpyxl.") from e

    read_kwargs: dict[str, Any] = {
        "engine": "openpyxl",
        "dtype": None,
    }
    if isinstance(opts.sheet, int):
        read_kwargs["sheet_name"] = opts.sheet - 1 if opts.sheet > 0 else 0
    elif isinstance(opts.sheet, str):
        read_kwargs["sheet_name"] = opts.sheet
    if opts.header_row is not None:
        read_kwargs["header"] = max(0, opts.header_row - 1)
    else:
        read_kwargs["header"] = 0

    logger.info("Reading Excel via pandas with args: %s", json.dumps(read_kwargs))
    df = pd.read_excel(opts.input_path, **read_kwargs)

    # Date handling
    if opts.date_format in ("iso", "auto"):
        for col in df.columns:
            # try to coerce to datetime; errors become NaT
            s = pd.to_datetime(df[col], errors="ignore")
            if hasattr(s, "dt") and getattr(s.dt, "tz", None) is None and str(s.dtype).startswith("datetime64"):
                df[col] = s.dt.tz_localize("UTC").dt.strftime("%Y-%m-%dT%H:%M:%SZ")

    if opts.limit_rows is not None:
        df = df.head(opts.limit_rows)

    lineterminator = "\n" if opts.line_ending == "LF" else "\r\n"
    quoting = csv_quoting_mode(opts.quoting)
    ensure_output_writable(opts.output_path or "output.csv", opts.force)
    df.to_csv(opts.output_path, index=False, sep=opts.delimiter, encoding=opts.encoding, lineterminator=lineterminator, quoting=quoting)
    return int(df.shape[0]), int(df.shape[1])


def convert_streaming_openpyxl(opts: ConversionOptions, logger: logging.Logger) -> tuple[int, int]:
    try:
        import openpyxl
    except Exception as e:
        raise RuntimeError("openpyxl is required for streaming mode.") from e

    wb = openpyxl.load_workbook(opts.input_path, read_only=True, data_only=True)
    if isinstance(opts.sheet, int):
        idx = opts.sheet - 1 if opts.sheet and opts.sheet > 0 else 0
        ws = wb.worksheets[idx]
    elif isinstance(opts.sheet, str):
        ws = wb[opts.sheet]
    else:
        ws = wb.active

    header_row_idx = opts.header_row if opts.header_row and opts.header_row > 0 else 1

    lineterminator = "\n" if opts.line_ending == "LF" else "\r\n"
    quoting = csv_quoting_mode(opts.quoting)
    ensure_output_writable(opts.output_path or "output.csv", opts.force)

    rows_written = 0
    columns_written = 0
    with open(opts.output_path, "w", newline="", encoding=opts.encoding) as f:
        writer = csv.writer(f, delimiter=opts.delimiter, quoting=quoting)

        for r_idx, row in enumerate(ws.iter_rows(values_only=True), start=1):
            if r_idx < header_row_idx:
                continue
            if r_idx == header_row_idx:
                headers = [str(cell) if cell is not None else "" for cell in row]
                columns_written = len(headers)
                writer.writerow(headers)
                continue

            out_row: List[str] = []
            for cell in row:
                if cell is None:
                    out_row.append("")
                    continue
                if isinstance(cell, datetime):
                    if opts.date_format in ("iso", "auto"):
                        out_row.append(cell.astimezone().strftime("%Y-%m-%dT%H:%M:%SZ"))
                    else:
                        out_row.append(str(cell))
                elif isinstance(cell, (int, float)):
                    out_row.append(str(cell))
                elif isinstance(cell, bool):
                    out_row.append("TRUE" if cell else "FALSE")
                else:
                    out_row.append(str(cell))

            writer.writerow(out_row)
            rows_written += 1
            if opts.limit_rows is not None and rows_written >= opts.limit_rows:
                break

    return rows_written, columns_written


def main(argv: Optional[List[str]] = None) -> int:
    logger = setup_logger()
    try:
        opts = parse_args(argv)
        logger.info("Options: %s", opts)
        if not os.path.exists(opts.input_path):
            logger.error("Input file not found: %s", opts.input_path)
            return 2

        if opts.use_streaming:
            mode = "streaming"
            rows, cols = convert_streaming_openpyxl(opts, logger)
        else:
            mode = "pandas"
            rows, cols = convert_with_pandas(opts, logger)

        logger.info("Conversion success (%s). Output: %s, rows=%d, cols=%d", mode, opts.output_path, rows, cols)
        print(json.dumps({"status": "ok", "mode": mode, "output": opts.output_path, "rows": rows, "cols": cols}, ensure_ascii=False))
        return 0
    except FileExistsError as e:
        logger.error(str(e))
        print(json.dumps({"error": str(e)}))
        return 4
    except Exception as e:
        logger.exception("Unhandled error")
        print(json.dumps({"error": str(e)}))
        return 1


if __name__ == "__main__":
    sys.exit(main())


