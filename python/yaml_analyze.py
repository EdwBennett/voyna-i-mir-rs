import re

import yaml

filename = "chapter_id_ru_en_ipa_words.yaml"


with open(filename, "r", encoding="utf-8") as f:
    items = yaml.safe_load(f)


for item in items:
    ru_words = len(item["ru"].split())
    ipa2_words = len(re.split(r"[ ‿]+", item["ipa2"]))
    # ipa2_words = len(item["ipa2"].split())
    if ru_words != ipa2_words:
        print(f"{item['chapter']}: ru={ru_words} ipa2={ipa2_words}")
