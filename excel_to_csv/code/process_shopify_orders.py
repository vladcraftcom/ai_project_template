#!/usr/bin/env python3
"""
Обработка Shopify заказов по странам 🌍

Загружает Excel, фильтрует только доставленные заказы, агрегирует продажи по стране:
- total_sales: сумма total по стране 💵
- items_sold: сумма quantity (количество проданных товаров) по стране 📦

Сохраняет CSV с топ-10 стран и интерактивную HTML-диаграмму (Plotly) 📈
"""

from __future__ import annotations

import argparse
import logging
from dataclasses import dataclass
from pathlib import Path
from typing import List

import pandas as pd
import plotly.express as px


LOGS_DIR = Path("/home/codefather/Documents/ai_project_template/ai_project_template/excel_to_csv/logs")
DATA_IN = Path("/home/codefather/Documents/ai_project_template/ai_project_template/excel_to_csv/data_in/shopify_orders_20250929_202605.xlsx")
DATA_OUT_CSV = Path("/home/codefather/Documents/ai_project_template/ai_project_template/excel_to_csv/data_out/shopify_orders_20250929_202605.csv")
DATA_OUT_HTML = Path("/home/codefather/Documents/ai_project_template/ai_project_template/excel_to_csv/data_out/diagram.html")


def setup_logging() -> None:
    """Настроить логирование в файл и консоль 😀"""
    LOGS_DIR.mkdir(parents=True, exist_ok=True)
    log_file = LOGS_DIR / "run.log"
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s [%(levelname)s] %(message)s",
        handlers=[
            logging.FileHandler(log_file, encoding="utf-8"),
            logging.StreamHandler(),
        ],
    )


@dataclass
class ProcessingConfig:
    """Конфигурация обработки 📋"""

    country_column: str = "shipping_country"
    total_column: str = "total"
    quantity_column: str = "quantity"
    fulfillment_column: str = "fulfillment_status"
    fulfilled_value: str = "fulfilled"
    top_n: int = 10


REQUIRED_COLUMNS: List[str] = [
    "order_id",
    "shipping_country",
    "quantity",
    "total",
    "fulfillment_status",
]


def validate_columns(df: pd.DataFrame) -> None:
    """Проверить, что в данных есть нужные колонки ✅"""
    missing = [c for c in REQUIRED_COLUMNS if c not in df.columns]
    if missing:
        raise ValueError(f"Отсутствуют необходимые столбцы: {missing}")


def load_data(path: Path) -> pd.DataFrame:
    """Загрузить Excel-файл с заказами 📥"""
    logging.info("Загрузка входного Excel: %s", path)
    df = pd.read_excel(path)
    logging.info("Загружено строк: %d, столбцов: %d", df.shape[0], df.shape[1])
    return df


def filter_fulfilled(df: pd.DataFrame, cfg: ProcessingConfig) -> pd.DataFrame:
    """Оставить только полностью доставленные заказы 🚚"""
    if cfg.fulfillment_column not in df.columns:
        logging.warning("Колонка статуса исполненности отсутствует: %s", cfg.fulfillment_column)
        return df
    filtered = df[df[cfg.fulfillment_column].astype(str).str.lower() == cfg.fulfilled_value]
    logging.info("После фильтрации fulfilled: %d строк", filtered.shape[0])
    return filtered


def aggregate_by_country(df: pd.DataFrame, cfg: ProcessingConfig) -> pd.DataFrame:
    """Агрегировать по стране: сумма total и сумма quantity 🧮"""
    # Работать с копией, чтобы избежать SettingWithCopyWarning
    df = df.copy()
    # Привести числовые поля безопасно
    for col in [cfg.total_column, cfg.quantity_column]:
        if col in df.columns:
            df[col] = pd.to_numeric(df[col], errors="coerce").fillna(0)

    grouped = (
        df.groupby(cfg.country_column, dropna=False)
        .agg(total_sales=(cfg.total_column, "sum"), items_sold=(cfg.quantity_column, "sum"))
        .reset_index()
    )
    # Сортировка: по total_sales desc, items_sold desc, затем страна
    grouped = grouped.sort_values(
        by=["total_sales", "items_sold", cfg.country_column],
        ascending=[False, False, True],
        kind="mergesort",
    )
    logging.info("Стран после агрегации: %d", grouped.shape[0])
    return grouped


def top_n_countries(df: pd.DataFrame, cfg: ProcessingConfig) -> pd.DataFrame:
    """Взять топ-N стран по продажам 🏆"""
    return df.head(cfg.top_n).copy()


def save_csv(df: pd.DataFrame, path: Path) -> None:
    """Сохранить результат в CSV 💾"""
    path.parent.mkdir(parents=True, exist_ok=True)
    df.to_csv(path, index=False)
    logging.info("CSV сохранён: %s", path)


def save_plot_html(df: pd.DataFrame, path: Path) -> None:
    """Построить интерактивную диаграмму и сохранить в HTML 🎨"""
    path.parent.mkdir(parents=True, exist_ok=True)
    title_text = "Топ-10 стран по выручке (USD) и количеству проданных товаров"
    fig = px.bar(
        df,
        x="shipping_country",
        y="total_sales",
        hover_data={"items_sold": True, "total_sales": ":.2f"},
        text="items_sold",
        title=title_text,
        color="total_sales",
        color_continuous_scale="Blues",
    )
    fig.update_traces(textposition="outside")
    fig.update_layout(
        xaxis_title="Страна",
        yaxis_title="Выручка, USD",
        template="plotly_white",
        uniformtext_minsize=10,
        uniformtext_mode="hide",
    )
    # Вставляем явный заголовок в HTML, чтобы он присутствовал в статическом контенте
    div_html = fig.to_html(include_plotlyjs="cdn", full_html=False)
    page_html = (
        "<html>\n"
        "<head><meta charset=\"utf-8\" /><title>" + title_text + "</title></head>\n"
        "<body>\n"
        "<h1>" + title_text + "</h1>\n"
        + div_html +
        "\n</body>\n</html>"
    )
    path.write_text(page_html, encoding="utf-8")
    logging.info("HTML диаграмма сохранена: %s", path)


def process(cfg: ProcessingConfig, input_path: Path, out_csv: Path, out_html: Path) -> None:
    """Выполнить полный процесс обработки 🔄"""
    df = load_data(input_path)
    validate_columns(df)
    df = filter_fulfilled(df, cfg)
    agg = aggregate_by_country(df, cfg)
    top = top_n_countries(agg, cfg)
    save_csv(top, out_csv)
    save_plot_html(top, out_html)


def parse_args() -> argparse.Namespace:
    """Разобрать аргументы коммандной строки 🔧"""
    parser = argparse.ArgumentParser(description="Process Shopify orders by country")
    parser.add_argument("--input", type=str, default=str(DATA_IN), help="Путь к входному Excel")
    parser.add_argument("--out_csv", type=str, default=str(DATA_OUT_CSV), help="Путь к выходному CSV")
    parser.add_argument("--out_html", type=str, default=str(DATA_OUT_HTML), help="Путь к HTML диаграмме")
    return parser.parse_args()


def main() -> None:
    """Точка входа 🚀"""
    setup_logging()
    args = parse_args()
    cfg = ProcessingConfig()
    try:
        process(cfg, Path(args.input), Path(args.out_csv), Path(args.out_html))
        logging.info("Готово! ✅")
    except Exception as exc:  # noqa: BLE001
        logging.exception("Ошибка обработки: %s", exc)
        raise


if __name__ == "__main__":
    main()


