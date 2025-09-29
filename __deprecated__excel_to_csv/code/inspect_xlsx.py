#!/usr/bin/env python3
"""
Lightweight XLSX inspector using only Python standard library.

Parses workbook, shared strings, styles, and first worksheet rows to report:
- sheet names
- header row index and headers
- row/column counts
- inferred column types and sample values

Usage:
  python excel_to_csv/code/inspect_xlsx.py --input path/to/file.xlsx [--limit-rows 500]
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from collections import defaultdict, Counter
from dataclasses import dataclass
from typing import Dict, Iterable, List, Optional, Tuple, Any
from zipfile import ZipFile
import xml.etree.ElementTree as ET


EXCEL_NS = {
    "main": "http://schemas.openxmlformats.org/spreadsheetml/2006/main",
    "rel": "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
}


BUILTIN_DATE_NUMFMT_IDS = {
    14, 15, 16, 17, 22, 27, 30, 36, 45, 46, 47,
}


@dataclass
class SheetInfo:
    name: str
    sheet_id: str
    rel_id: Optional[str]
    path: Optional[str]


def read_workbook_sheets(zf: ZipFile) -> Tuple[List[SheetInfo], Dict[str, str]]:
    """Return list of sheets and relationships (rId -> target path)."""
    sheets: List[SheetInfo] = []
    rels_map: Dict[str, str] = {}

    # workbook
    with zf.open("xl/workbook.xml") as f:
        tree = ET.parse(f)
    root = tree.getroot()
    for sheet in root.findall("main:sheets/main:sheet", EXCEL_NS):
        name = sheet.attrib.get("name", "")
        sheet_id = sheet.attrib.get("sheetId", "")
        rel_id = sheet.attrib.get(f"{{{EXCEL_NS['rel']}}}id")
        sheets.append(SheetInfo(name=name, sheet_id=sheet_id, rel_id=rel_id, path=None))

    # relationships
    try:
        with zf.open("xl/_rels/workbook.xml.rels") as f:
            tree = ET.parse(f)
        root = tree.getroot()
        for rel in root.findall("rel:Relationship", EXCEL_NS):
            r_id = rel.attrib.get("Id")
            target = rel.attrib.get("Target")
            if r_id and target:
                rels_map[r_id] = target
    except KeyError:
        rels_map = {}

    # map paths
    for s in sheets:
        if s.rel_id and s.rel_id in rels_map:
            s.path = f"xl/{rels_map[s.rel_id]}" if not rels_map[s.rel_id].startswith("/") else rels_map[s.rel_id][1:]
        else:
            # fallback to common default
            s.path = "xl/worksheets/sheet1.xml"

    return sheets, rels_map


def read_shared_strings(zf: ZipFile) -> List[str]:
    """Read shared strings table if present."""
    try:
        with zf.open("xl/sharedStrings.xml") as f:
            tree = ET.parse(f)
        root = tree.getroot()
        strings: List[str] = []
        for si in root.findall("main:si", EXCEL_NS):
            # concatenate all <t> in <si>
            text_parts: List[str] = []
            for t in si.findall(".//main:t", EXCEL_NS):
                text_parts.append(t.text or "")
            strings.append("".join(text_parts))
        return strings
    except KeyError:
        return []


def read_styles(zf: ZipFile) -> Tuple[Dict[int, int], Dict[int, str]]:
    """Return (xf_index -> numFmtId) and custom numFmtId -> formatCode."""
    xf_to_numfmt: Dict[int, int] = {}
    custom_numfmts: Dict[int, str] = {}
    try:
        with zf.open("xl/styles.xml") as f:
            tree = ET.parse(f)
        root = tree.getroot()

        # custom formats
        numFmts = root.find("main:numFmts", EXCEL_NS)
        if numFmts is not None:
            for numFmt in numFmts.findall("main:numFmt", EXCEL_NS):
                numFmtId = int(numFmt.attrib.get("numFmtId", "0"))
                formatCode = numFmt.attrib.get("formatCode", "")
                custom_numfmts[numFmtId] = formatCode

        # cellXfs to map style index -> numFmtId
        cellXfs = root.find("main:cellXfs", EXCEL_NS)
        if cellXfs is not None:
            for idx, xf in enumerate(cellXfs.findall("main:xf", EXCEL_NS)):
                numFmtId = int(xf.attrib.get("numFmtId", "0"))
                xf_to_numfmt[idx] = numFmtId
    except KeyError:
        pass
    return xf_to_numfmt, custom_numfmts


_COL_RE = re.compile(r"([A-Z]+)([0-9]+)")


def col_letters_to_index(col_letters: str) -> int:
    idx = 0
    for ch in col_letters:
        idx = idx * 26 + (ord(ch) - ord("A") + 1)
    return idx - 1


def is_date_numfmt(numfmt_id: int, custom_formats: Dict[int, str]) -> bool:
    if numfmt_id in BUILTIN_DATE_NUMFMT_IDS:
        return True
    fmt = custom_formats.get(numfmt_id)
    if not fmt:
        return False
    # a very loose heuristic: presence of date tokens
    fmt_lower = fmt.lower()
    return any(token in fmt_lower for token in ["d", "y", "m/", "h", "s"]) and not ("0" in fmt_lower and "." in fmt_lower)


def decode_cell_value(c_elem: ET.Element, sst: List[str], xf_to_numfmt: Dict[int, int], custom_formats: Dict[int, str]) -> Tuple[Any, str]:
    """Decode cell value and return (python_value, inferred_kind). kind in {empty,str,int,float,bool,date}."""
    t = c_elem.attrib.get("t")
    s_attr = c_elem.attrib.get("s")
    v_elem = c_elem.find("main:v", EXCEL_NS)
    is_elem = c_elem.find("main:is", EXCEL_NS)

    if v_elem is None and is_elem is None:
        return None, "empty"

    if t == "s":  # shared string
        if v_elem is None:
            return "", "str"
        idx = int(v_elem.text or "0")
        return sst[idx] if 0 <= idx < len(sst) else "", "str"

    if t == "b":
        return (v_elem.text == "1"), "bool"

    if t == "str":
        return (v_elem.text or ""), "str"

    if t == "inlineStr":
        # concatenate <t>
        text_parts: List[str] = []
        for tnode in is_elem.findall(".//main:t", EXCEL_NS):
            text_parts.append(tnode.text or "")
        return "".join(text_parts), "str"

    # default: numeric; might be date per style
    text = v_elem.text or ""
    try:
        num = float(text)
        if s_attr is not None:
            style_idx = int(s_attr)
            numfmt_id = xf_to_numfmt.get(style_idx, 0)
            if is_date_numfmt(numfmt_id, custom_formats):
                # do not convert to datetime here; report as Excel serial
                return num, "date"
        if num.is_integer():
            return int(num), "int"
        return num, "float"
    except ValueError:
        return text, "str"


def infer_column_kind(kinds: Counter) -> str:
    if not kinds:
        return "empty"
    # prioritize by occurrence; but ensure date outweighs numeric if present
    ordered = kinds.most_common()
    labels = {k for k, _ in ordered}
    if "date" in labels:
        return "date"
    return ordered[0][0]


def unique_headers(headers: List[str]) -> List[str]:
    counts: Dict[str, int] = defaultdict(int)
    result: List[str] = []
    for h in headers:
        base = (h or "").strip()
        counts[base] += 1
        if counts[base] == 1:
            result.append(base)
        else:
            result.append(f"{base}__{counts[base]}")
    return result


def parse_sheet(zf: ZipFile, sheet_path: str, sst: List[str], xf_to_numfmt: Dict[int, int], custom_formats: Dict[int, str], row_limit: Optional[int]) -> Dict[str, Any]:
    with zf.open(sheet_path) as f:
        tree = ET.parse(f)
    root = tree.getroot()
    sheet_data = root.find("main:sheetData", EXCEL_NS)
    if sheet_data is None:
        return {"header_row_index": None, "headers": [], "data_rows": 0, "columns": [], "samples": []}

    header_row_index: Optional[int] = None
    headers_by_col_index: Dict[int, str] = {}
    data_rows: List[List[Any]] = []

    for row_idx, row in enumerate(sheet_data.findall("main:row", EXCEL_NS), start=1):
        # build sparse row by col index
        cells: Dict[int, Tuple[Any, str]] = {}
        for c in row.findall("main:c", EXCEL_NS):
            r = c.attrib.get("r", "")
            m = _COL_RE.match(r)
            if not m:
                continue
            col_letters = m.group(1)
            col_index = col_letters_to_index(col_letters)
            value, kind = decode_cell_value(c, sst, xf_to_numfmt, custom_formats)
            cells[col_index] = (value, kind)

        if header_row_index is None:
            # header row is the first row with any non-empty cell
            if any(v is not None and v != "" for (v, _k) in cells.values()):
                max_col = max(cells.keys()) if cells else -1
                headers = ["" for _ in range(max_col + 1)]
                for idx, (v, _k) in cells.items():
                    headers[idx] = str(v) if v is not None else ""
                headers = unique_headers(headers)
                headers_by_col_index = {i: h for i, h in enumerate(headers)}
                header_row_index = row_idx
            continue

        # data rows after header
        if header_row_index is not None:
            max_col = max(list(headers_by_col_index.keys()) + list(cells.keys())) if cells else max(headers_by_col_index.keys(), default=-1)
            row_values: List[Any] = []
            for i in range(max_col + 1):
                v = cells.get(i, (None, "empty"))[0]
                row_values.append(v)
            data_rows.append(row_values)
            if row_limit is not None and len(data_rows) >= row_limit:
                break

    # infer column kinds
    col_kinds: Dict[int, Counter] = defaultdict(Counter)
    for row_values in data_rows:
        for i, v in enumerate(row_values):
            kind: str
            if v is None or v == "":
                kind = "empty"
            elif isinstance(v, bool):
                kind = "bool"
            elif isinstance(v, int):
                kind = "int"
            elif isinstance(v, float):
                kind = "float"
            else:
                kind = "str"
            col_kinds[i][kind] += 1

    columns_summary: List[Dict[str, Any]] = []
    for i in sorted(headers_by_col_index.keys()):
        kinds = col_kinds.get(i, Counter())
        inferred = infer_column_kind(kinds)
        non_null = sum(c for k, c in kinds.items() if k != "empty")
        sample_vals: List[Any] = []
        for rv in data_rows[:5]:
            if i < len(rv):
                sample_vals.append(rv[i])
        columns_summary.append({
            "index": i,
            "name": headers_by_col_index[i],
            "inferred_kind": inferred,
            "non_null_count": non_null,
            "samples": sample_vals,
        })

    return {
        "header_row_index": header_row_index,
        "headers": [headers_by_col_index[i] for i in sorted(headers_by_col_index.keys())],
        "data_rows": len(data_rows),
        "columns": columns_summary,
        "samples": data_rows[:5],
    }


def inspect_xlsx(path: str, row_limit: Optional[int]) -> Dict[str, Any]:
    with ZipFile(path) as zf:
        sheets, _rels = read_workbook_sheets(zf)
        sst = read_shared_strings(zf)
        xf_to_numfmt, custom_formats = read_styles(zf)

        all_sheets = []
        for s in sheets:
            all_sheets.append({"name": s.name, "path": s.path})

        # assume first sheet present
        if not sheets:
            return {"sheets": [], "error": "No sheets found"}
        target_sheet = sheets[0]
        sheet_result = parse_sheet(zf, target_sheet.path or "xl/worksheets/sheet1.xml", sst, xf_to_numfmt, custom_formats, row_limit)

        result: Dict[str, Any] = {
            "sheets": all_sheets,
            "inspected_sheet": target_sheet.name,
            "sheet_path": target_sheet.path,
        }
        result.update(sheet_result)
        return result


def main() -> None:
    parser = argparse.ArgumentParser(description="Inspect XLSX structure and content")
    parser.add_argument("--input", required=True, help="Path to .xlsx file")
    parser.add_argument("--limit-rows", type=int, default=500, help="Max data rows to sample (after header)")
    args = parser.parse_args()

    try:
        report = inspect_xlsx(args.input, args.limit_rows)
        print(json.dumps(report, ensure_ascii=False, indent=2))
    except FileNotFoundError:
        print(json.dumps({"error": f"File not found: {args.input}"}, ensure_ascii=False))
        sys.exit(2)
    except KeyError as e:
        print(json.dumps({"error": f"Invalid XLSX structure: missing {e}"}, ensure_ascii=False))
        sys.exit(3)
    except Exception as e:
        print(json.dumps({"error": str(e)}), ensure_ascii=False)
        sys.exit(1)


if __name__ == "__main__":
    main()


