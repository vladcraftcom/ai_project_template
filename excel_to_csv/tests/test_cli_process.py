from __future__ import annotations

from pathlib import Path
from typing import Any

import pandas as pd
import pytest

import excel_to_csv.code.process_shopify_orders as mod


def test_process_end_to_end_with_mocks(monkeypatch: pytest.MonkeyPatch, tmp_path: Path):
    # Prepare synthetic dataset
    df_src = pd.DataFrame(
        [
            {"order_id": 1, "shipping_country": "A", "quantity": 2, "total": 10.0, "fulfillment_status": "fulfilled"},
            {"order_id": 2, "shipping_country": "B", "quantity": 1, "total": 5.0, "fulfillment_status": "fulfilled"},
        ]
    )

    def fake_read_excel(path: Any):  # type: ignore[no-untyped-def]
        return df_src

    monkeypatch.setattr(mod.pd, "read_excel", fake_read_excel)

    cfg = mod.ProcessingConfig(top_n=10)
    out_csv = tmp_path / "res.csv"
    out_html = tmp_path / "res.html"

    mod.process(cfg, Path("/tmp/in.xlsx"), out_csv, out_html)

    assert out_csv.exists()
    assert out_html.exists()
    content = out_csv.read_text(encoding="utf-8")
    assert "shipping_country,total_sales,items_sold" in content.splitlines()[0]


