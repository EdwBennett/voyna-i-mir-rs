import yaml

filename = "Volume_1_Part_1.yaml"
output_filename = "Volume_1_Part_1.txt"

with open(filename, "r", encoding="utf-8") as f:
    items = yaml.safe_load(f)

with open(output_filename, "w", encoding="utf-8") as out:
    for item in items:
        print(file=out)
        print(item["chapter"], file=out)
        print(item["words"], file=out)
