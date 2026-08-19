#!/usr/bin/env python3
import yaml

filename = "Volume_1_Part_1.yaml"

with open(filename, "r", encoding="utf-8") as f:
    items = yaml.safe_load(f)

for item in items:
    print()
    print(item["chapter"])
    print()
    print(item["words"])
     