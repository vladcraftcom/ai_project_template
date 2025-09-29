from __future__ import annotations

import pandas as pd
import pytest

from excel_to_csv.code.process_shopify_orders import (
    ProcessingConfig,
    validate_columns,
    filter_fulfilled,
    aggregate_by_country,
    top_n_countries,
)


def make_df() -> pd.DataFrame:
    return pd.DataFrame(
        [
            {"order_id": 1, "shipping_country": "A", "quantity": 2, "total": 10.0, "fulfillment_status": "fulfilled"},
            {"order_id": 2, "shipping_country": "A", "quantity": 1, "total": 5.0, "fulfillment_status": "pending"},
            {"order_id": 3, "shipping_country": "B", "quantity": 3, "total": 12.0, "fulfillment_status": "fulfilled"},
            {"order_id": 4, "shipping_country": "B", "quantity": 1, "total": 4.0, "fulfillment_status": "fulfilled"},
            {"order_id": 5, "shipping_country": "C", "quantity": 10, "total": 100.0, "fulfillment_status": "unfulfilled"},
        ]
    )


def test_validate_columns_ok():
    df = make_df()
    # Should not raise
    validate_columns(df)


def test_validate_columns_missing():
    df = make_df().drop(columns=["total"])  # remove required column
    with pytest.raises(ValueError):
        validate_columns(df)


def test_filter_fulfilled():
    df = make_df()
    cfg = ProcessingConfig()
    out = filter_fulfilled(df, cfg)
    assert set(out["fulfillment_status"].unique()) == {"fulfilled"}
    # Expect rows 1,3,4 remain → 3 rows
    assert len(out) == 3


def test_aggregate_by_country_and_sorting():
    df = make_df()
    cfg = ProcessingConfig()
    df_f = filter_fulfilled(df, cfg)
    agg = aggregate_by_country(df_f, cfg)
    # Expected aggregates:
    # A: total=10.0, items=2; B: total=16.0, items=4
    a_row = agg[agg[cfg.country_column] == "A"].iloc[0]
    b_row = agg[agg[cfg.country_column] == "B"].iloc[0]
    assert pytest.approx(a_row["total_sales"], rel=1e-6) == 10.0
    assert a_row["items_sold"] == 2
    assert pytest.approx(b_row["total_sales"], rel=1e-6) == 16.0
    assert b_row["items_sold"] == 4
    # Sorted by total desc: B before A
    assert list(agg[cfg.country_column])[:2] == ["B", "A"]


def test_top_n_countries():
    df = pd.DataFrame({"shipping_country": list("ABCDE"), "total_sales": [5,4,3,2,1], "items_sold": [50,40,30,20,10]})
    cfg = ProcessingConfig(top_n=3)
    top = top_n_countries(df, cfg)
    assert len(top) == 3
    assert list(top["shipping_country"]) == ["A", "B", "C"]


