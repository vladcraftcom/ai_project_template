from __future__ import annotations

from pathlib import Path

import pandas as pd

import excel_to_csv.code.process_shopify_orders as mod


def test_logging_messages(caplog):  # type: ignore[no-untyped-def]
    caplog.set_level("INFO")
    mod.setup_logging()

    df = pd.DataFrame(
        [
            {"order_id": 1, "shipping_country": "A", "quantity": 2, "total": 10.0, "fulfillment_status": "fulfilled"},
            {"order_id": 2, "shipping_country": "B", "quantity": 1, "total": 5.0, "fulfillment_status": "fulfilled"},
        ]
    )
    cfg = mod.ProcessingConfig(top_n=1)
    agg = mod.aggregate_by_country(df, cfg)
    top = mod.top_n_countries(agg, cfg)
    out_csv = Path("/tmp/out.csv")
    out_html = Path("/tmp/out.html")
    mod.save_csv(top, out_csv)
    mod.save_plot_html(top, out_html)

    messages = " ".join(rec.getMessage() for rec in caplog.records)
    assert "CSV сохранён:" in messages
    assert "HTML диаграмма сохранена:" in messages


