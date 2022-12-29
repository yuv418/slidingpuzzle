#!/usr/bin/python3

# This tool converts an un-normalized theme, where colors are out of 255.0 instead of 1.0 to a normal theme.

import json
import sys

try:
    inp = sys.argv[1]
    output = sys.argv[2]
except IndexError:
    print("Usage: ./theme_normalize_rgba.py INPUT OUTPUT")
    exit(1)

print(f"Normalizing {inp} to {output}")

with open(inp) as inp_f:
    theme = json.load(inp_f)

for k in theme.keys():
    # Only normalize RGBA color lists
    if type(theme[k]) == list and len(theme[k]) == 4:
        theme[k] = [i / 255.0 for i in theme[k]]

with open(output, 'w') as out_f:
    json.dump(theme, out_f)

print(f"Successfully normalized {inp} to {output}")
