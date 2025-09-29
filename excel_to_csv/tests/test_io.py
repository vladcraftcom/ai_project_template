from __future__ import annotations

from pathlib import Path
from typing import Any

import pandas as pd
import pytest

import excel_to_csv.code.process_shopify_orders as mod


def test_load_data_monkeypatched(monkeypatch: pytest.MonkeyPatch):
    df_expected = pd.DataFrame({"a": [1, 2]})

    def fake_read_excel(path: Any):  # type: ignore[no-untyped-def]
        return df_expected

    monkeypatch.setattr(mod.pd, "read_excel", fake_read_excel)
    df = mod.load_data(Path("/tmp/fake.xlsx"))
    assert df.equals(df_expected)


def test_save_csv(tmp_path: Path):
    df = pd.DataFrame({"shipping_country": ["A", "B"], "total_sales": [10.0, 5.0], "items_sold": [2, 1]})
    out = tmp_path / "out.csv"
    mod.save_csv(df, out)
    assert out.exists()
    content = out.read_text(encoding="utf-8")
    assert "shipping_country,total_sales,items_sold" in content.splitlines()[0]


def test_save_plot_html(tmp_path: Path):
    df = pd.DataFrame({"shipping_country": ["A", "B"], "total_sales": [10.0, 5.0], "items_sold": [2, 1]})
    out = tmp_path / "chart.html"
    mod.save_plot_html(df, out)
    assert out.exists()
    html = out.read_text(encoding="utf-8")
    assert "Топ-10 стран по выручке" in html


