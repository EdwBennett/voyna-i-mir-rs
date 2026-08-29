import yaml

filename = "chapter_ru_en_ipa.yaml"


with open(filename, "r", encoding="utf-8") as f:
    items = yaml.safe_load(f)


for item in items:
    ru_words = len(item["ru"].split())
    ipa_words = len(item["ipa"].split())
    if ru_words != ipa_words:
        print(f"{item['chapter']}: ru={ru_words} ipa={ipa_words}")
