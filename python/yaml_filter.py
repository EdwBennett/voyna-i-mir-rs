import sys

import yaml

filename = "Volume_1_Part_1.yaml"


with open(filename, "r", encoding="utf-8") as f:
    items = yaml.safe_load(f)


entries = [
    {
        "chapter": item["chapter"],
        "ru": item["ru"],
        "en": item["en"],
    }
    for item in items
]


yaml.dump(
    entries,
    sys.stdout,
    allow_unicode=True,
    sort_keys=False,
    width=float("inf"),
    default_style='"',
)
