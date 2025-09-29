from __future__ import annotations

from pathlib import Path
from typing import Any

import pytest

import excel_to_csv.run_pipeline as pipeline


def test_run_pipeline_processes_all_xlsx(monkeypatch: pytest.MonkeyPatch, tmp_path: Path):
    # Arrange: create fake data_in with 3 xlsx files
    data_in = tmp_path / "excel_to_csv/data_in"
    data_out = tmp_path / "excel_to_csv/data_out"
    data_in.mkdir(parents=True, exist_ok=True)
    data_out.mkdir(parents=True, exist_ok=True)
    for name in ["a.xlsx", "b.xlsx", "c.xlsx"]:
        (data_in / name).write_bytes(b"fake")

    # Patch paths used in pipeline
    monkeypatch.setattr(pipeline, "ROOT", tmp_path)
    monkeypatch.setattr(pipeline, "IN_DIR", data_in)
    monkeypatch.setattr(pipeline, "OUT_DIR", data_out)

    calls: list[list[str]] = []

    def fake_run(cmd: list[str], check: bool):  # type: ignore[no-untyped-def]
        calls.append(cmd)
        # create expected outputs to simulate processor success
        out_csv = Path(cmd[cmd.index("--out_csv") + 1])
        out_html = Path(cmd[cmd.index("--out_html") + 1])
        out_csv.write_text("shipping_country,total_sales,items_sold\n", encoding="utf-8")
        out_html.write_text("<html></html>", encoding="utf-8")

    monkeypatch.setattr(pipeline.subprocess, "run", fake_run)

    rc = pipeline.main()
    assert rc == 0
    # We expect 3 invocations and 3 pairs of outputs
    assert len(calls) == 3
    for name in ["a", "b", "c"]:
        assert (data_out / f"{name}.csv").exists()
        assert (data_out / f"{name}.html").exists()


