import sys

import yaml

filename = "chapter_ru_en_ipa_words.yaml"


with open(filename, "r", encoding="utf-8") as f:
    items = yaml.safe_load(f)


entries = [
    {
        "chapter": item["chapter"],
        "ru": item["ru"],
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
