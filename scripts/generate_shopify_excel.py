import argparse
import os
import random
import secrets
from datetime import datetime, timedelta
from typing import List, Dict, Optional

try:
    import pandas as pd
except ImportError as exc:  # pragma: no cover
    raise SystemExit("Missing dependency 'pandas'. Install with: pip install pandas") from exc

try:
    from faker import Faker
except ImportError as exc:  # pragma: no cover
    raise SystemExit("Missing dependency 'faker'. Install with: pip install Faker") from exc


def generate_product_catalog(faker: Faker, num_products: int = 250) -> List[Dict[str, str]]:
    categories = [
        "Electronics",
        "Apparel",
        "Home & Kitchen",
        "Beauty",
        "Sports",
        "Books",
        "Toys",
        "Health",
        "Jewelry",
        "Grocery",
        "Pets",
    ]

    adjectives = [
        "Premium",
        "Classic",
        "Eco",
        "Smart",
        "Portable",
        "Wireless",
        "Limited",
        "Pro",
        "Mini",
        "Ultra",
        "Comfort",
        "Essential",
    ]

    nouns = [
        "Headphones",
        "T-Shirt",
        "Backpack",
        "Coffee Maker",
        "Blender",
        "Running Shoes",
        "Yoga Mat",
        "Face Cream",
        "Necklace",
        "Vacuum",
        "Water Bottle",
        "Notebook",
        "Board Game",
        "Sunglasses",
        "Desk Lamp",
        "Gaming Mouse",
    ]

    catalog: List[Dict[str, str]] = []
    for i in range(1, num_products + 1):
        category = random.choice(categories)
        title = f"{random.choice(adjectives)} {random.choice(nouns)}"
        product_id = f"PROD-{i:05d}"
        sku = f"SKU-{i:05d}-{random.randint(100, 999)}"
        variant = random.choice(["Standard", "Large", "Small", "XL", "Red", "Blue", "Black", "White"])
        catalog.append({
            "product_id": product_id,
            "product_title": title,
            "product_category": category,
            "sku": sku,
            "variant": variant,
        })
    return catalog


def generate_customers(faker: Faker, num_customers: int = 1500) -> List[Dict[str, str]]:
    customers: List[Dict[str, str]] = []
    for i in range(1, num_customers + 1):
        first_name = faker.first_name()
        last_name = faker.last_name()
        customers.append({
            "customer_id": f"CUST-{i:06d}",
            "customer_name": f"{first_name} {last_name}",
            "customer_email": faker.unique.email(),
            "customer_phone": faker.phone_number(),
            "customer_country": faker.country(),
        })
    return customers


def random_monetary_amount(min_value: float, max_value: float) -> float:
    value = random.uniform(min_value, max_value)
    return round(value, 2)


def build_records(
    num_rows: int,
    currency: str,
    faker: Faker,
    customers: List[Dict[str, str]],
    products: List[Dict[str, str]],
) -> List[Dict[str, object]]:
    payment_methods = [
        "credit_card",
        "paypal",
        "apple_pay",
        "google_pay",
        "bank_transfer",
        "cash_on_delivery",
    ]
    fulfillment_statuses = [
        "fulfilled",
        "partially_fulfilled",
        "unfulfilled",
        "pending",
    ]

    base_order_number = random.randint(10000, 90000)
    records: List[Dict[str, object]] = []

    for i in range(num_rows):
        customer = random.choice(customers)
        product = random.choice(products)

        order_date = faker.date_time_between(start_date="-365d", end_date="now")
        quantity = random.choices([1, 2, 3, 4, 5], weights=[60, 20, 10, 6, 4], k=1)[0]
        unit_price = random_monetary_amount(5.0, 500.0)
        subtotal = round(unit_price * quantity, 2)

        # Discount applied to some orders
        discount_rate = random.choice([0.0, 0.05, 0.10, 0.15, 0.20, 0.25, 0.30])
        discount_applied = random.choices([True, False], weights=[35, 65], k=1)[0]
        discount = round(subtotal * discount_rate, 2) if discount_applied else 0.0

        # Simple tax model between 0% and 21%
        tax_rate = random.choice([0.0, 0.05, 0.07, 0.10, 0.12, 0.15, 0.18, 0.21])
        taxable_amount = max(subtotal - discount, 0.0)
        tax = round(taxable_amount * tax_rate, 2)

        shipping_cost = random.choice([0.0, 3.99, 5.99, 9.99, 14.99])

        total = round(taxable_amount + tax + shipping_cost, 2)

        order_number = base_order_number + i
        shopify_order_name = f"#{order_number}"
        order_id = f"SHP-{order_number}-{random.randint(1000, 9999)}"

        address = {
            "shipping_address_line1": faker.street_address(),
            "shipping_city": faker.city(),
            "shipping_state": faker.state_abbr() if faker.random.choice([True, False]) else faker.state(),
            "shipping_zip": faker.postcode(),
            "shipping_country": customer["customer_country"],
        }

        record = {
            "order_id": order_id,
            "shopify_order_number": shopify_order_name,
            "order_date": order_date,
            "fulfillment_status": random.choice(fulfillment_statuses),
            "payment_method": random.choice(payment_methods),
            "currency": currency,
            "customer_id": customer["customer_id"],
            "customer_name": customer["customer_name"],
            "customer_email": customer["customer_email"],
            "customer_phone": customer["customer_phone"],
            "customer_country": customer["customer_country"],
            "product_id": product["product_id"],
            "product_title": product["product_title"],
            "product_category": product["product_category"],
            "sku": product["sku"],
            "variant": product["variant"],
            "quantity": quantity,
            "unit_price": unit_price,
            "subtotal": subtotal,
            "discount": discount,
            "tax": tax,
            "shipping_cost": shipping_cost,
            "total": total,
            **address,
        }

        records.append(record)

    return records


def write_excel(records: List[Dict[str, object]], output_dir: str, file_prefix: str = "shopify_orders") -> str:
    os.makedirs(output_dir, exist_ok=True)
    # Добавляем микросекунды, чтобы избежать коллизий имён при частых запусках
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S_%f")
    output_file = os.path.join(output_dir, f"{file_prefix}_{timestamp}.xlsx")

    df = pd.DataFrame.from_records(records)
    # Sort columns for a nicer layout
    preferred_order = [
        "order_id",
        "shopify_order_number",
        "order_date",
        "fulfillment_status",
        "payment_method",
        "currency",
        "customer_id",
        "customer_name",
        "customer_email",
        "customer_phone",
        "customer_country",
        "shipping_address_line1",
        "shipping_city",
        "shipping_state",
        "shipping_zip",
        "shipping_country",
        "product_id",
        "product_title",
        "product_category",
        "sku",
        "variant",
        "quantity",
        "unit_price",
        "subtotal",
        "discount",
        "tax",
        "shipping_cost",
        "total",
    ]
    # Keep only available columns and order them
    columns = [c for c in preferred_order if c in df.columns]
    df = df[columns]

    try:
        df.to_excel(output_file, index=False, engine="openpyxl")
    except Exception:
        # Fallback engine if openpyxl is not installed but xlsxwriter is
        df.to_excel(output_file, index=False, engine="xlsxwriter")

    return output_file


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Generate fake Shopify order data into an Excel file.")
    default_out = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "data_in"))
    parser.add_argument("--rows", type=int, default=10000, help="Number of rows to generate (default: 10000)")
    parser.add_argument("--currency", type=str, default="USD", help="Currency code to use (default: USD)")
    parser.add_argument("--out", type=str, default=default_out, help="Output directory for the Excel file")
    parser.add_argument("--seed", type=int, default=None, help="Random seed for reproducibility (omit for unique data each run)")
    return parser.parse_args()


def main() -> None:
    args = parse_args()

    # Если сид не указан, генерируем криптографически стойкий сид для уникальности каждого запуска
    seed: int
    if args.seed is None:
        seed = secrets.randbits(64) ^ (os.getpid() << 16) ^ int(datetime.now().timestamp() * 1e9)
    else:
        seed = int(args.seed)

    random.seed(seed)
    faker = Faker("en_US")
    Faker.seed(seed)

    products = generate_product_catalog(faker)
    customers = generate_customers(faker)
    records = build_records(args.rows, args.currency, faker, customers, products)
    output_file = write_excel(records, args.out)

    print(f"Generated {len(records)} rows → {output_file} | seed={seed}")


if __name__ == "__main__":
    main()


