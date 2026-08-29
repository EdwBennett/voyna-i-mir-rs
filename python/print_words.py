import yaml

filename = "Volume_1_Part_1.yaml"
output_filename = "output.yaml"

with open(filename, "r", encoding="utf-8") as f:
    items = yaml.safe_load(f)

output = [
    {
        "chapter": item["chapter"],
        "id": item["id"],
        "ru": item["ru"],
        "en": item["en"],
    }
    for item in items
]

with open(output_filename, "w", encoding="utf-8") as out:
    yaml.dump(
        output,
        out,
        allow_unicode=True,
        sort_keys=False,
        width=float("inf"),
        default_style='"',
    )
