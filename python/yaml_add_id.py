import sys

import yaml

file1 = "chapter_ru_en_ipa_words.yaml"


with open(file1, "r", encoding="utf-8") as f:
    items1 = yaml.safe_load(f)
items2 = [{"id": i} for i in range(1, 26)]


entries = []
for item1, item2 in zip(items1, items2):
    entries.append(
        {
            "chapter": item1["chapter"],
            "id": item2["id"],
            "ru": item1["ru"],
            "en": item1["en"],
            "ipa": item1["ipa"],
            "words": item1["words"],
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
