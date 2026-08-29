import sys

import yaml

file1 = "chapter_ru_en_ipa.yaml"
file2 = "words.yaml"


with open(file1, "r", encoding="utf-8") as f:
    items1 = yaml.safe_load(f)
with open(file2, "r", encoding="utf-8") as f:
    items2 = yaml.safe_load(f)


entries = []
for item1, item2 in zip(items1, items2):
    assert item1["chapter"] == item2["chapter"]
    entries.append(
        {
            "chapter": item1["chapter"],
            "ru": item1["ru"],
            "en": item1["en"],
            "ipa": item1["ipa"],
            "words": item2["words"],
        }
    )


yaml.dump(
    entries,
    sys.stdout,
    allow_unicode=True,
    sort_keys=False,
    width=float("inf"),
    default_style='"',
)
