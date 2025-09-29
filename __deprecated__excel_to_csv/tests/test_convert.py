import os
import csv
import unittest
from pathlib import Path


class TestConvertBasic(unittest.TestCase):
    def test_cli_exists(self):
        self.assertTrue(Path('excel_to_csv/code/convert_xlsx_to_csv.py').exists())

    def test_result_file_created(self):
        out = Path('excel_to_csv/data_out/shopify_orders_20250918_202022.csv')
        self.assertTrue(out.exists())
        with out.open(newline='', encoding='utf-8') as f:
            r = csv.reader(f)
            header = next(r)
            self.assertEqual(len(header), 28)


if __name__ == '__main__':
    unittest.main()


