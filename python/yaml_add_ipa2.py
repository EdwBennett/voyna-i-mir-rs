import json
import re

yaml_file = "chapter_id_ru_en_ipa_words.yaml"
ipa2_file = "ipa2.json"


with open(yaml_file, "r", encoding="utf-8") as f:
    yaml_lines = f.readlines()

with open(ipa2_file, "r", encoding="utf-8") as f:
    ipa2_values = [entry["ipa2"] for entry in json.load(f)]


output_lines = []
ipa2_iter = iter(ipa2_values)
for line in yaml_lines:
    if re.match(r'^\s*"ipa2": ".*"\s*$', line):
        ipa2_value = next(ipa2_iter)
        indent = line[: len(line) - len(line.lstrip())]
        output_lines.append(f'{indent}"ipa2": "{ipa2_value}"\n')
    else:
        output_lines.append(line)

remaining = list(ipa2_iter)
assert not remaining, f"unmatched ipa2 entries: {len(remaining)}"


with open(yaml_file, "w", encoding="utf-8") as f:
    f.writelines(output_lines)
