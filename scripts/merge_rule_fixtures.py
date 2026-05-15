"""Merge _positive.txt and _negative.txt into corresponding rule-name files, then delete them."""
import os
import shutil

BASE = "tests/fixtures/python/rules_fixtures"

for folder in os.listdir(BASE):
    folder_path = os.path.join(BASE, folder)
    if not os.path.isdir(folder_path):
        continue

    # Find the rule name by looking for a .json file
    json_files = [f for f in os.listdir(folder_path) if f.endswith(".json")]
    if not json_files:
        continue

    rule_name = json_files[0].replace(".json", "")

    # Process _positive.txt -> rule_name_positive.txt
    src_pos = os.path.join(folder_path, "_positive.txt")
    dst_pos = os.path.join(folder_path, f"{rule_name}_positive.txt")
    if os.path.exists(src_pos) and os.path.exists(dst_pos):
        with open(src_pos) as f_src, open(dst_pos, "a") as f_dst:
            f_dst.write(f_src.read())
        os.remove(src_pos)
        print(f"Merged and removed {src_pos}")
    elif os.path.exists(src_pos) and not os.path.exists(dst_pos):
        shutil.move(src_pos, dst_pos)
        print(f"Moved {src_pos} -> {dst_pos}")

    # Process _negative.txt -> rule_name_negative.txt
    src_neg = os.path.join(folder_path, "_negative.txt")
    dst_neg = os.path.join(folder_path, f"{rule_name}_negative.txt")
    if os.path.exists(src_neg) and os.path.exists(dst_neg):
        with open(src_neg) as f_src, open(dst_neg, "a") as f_dst:
            f_dst.write(f_src.read())
        os.remove(src_neg)
        print(f"Merged and removed {src_neg}")
    elif os.path.exists(src_neg) and not os.path.exists(dst_neg):
        shutil.move(src_neg, dst_neg)
        print(f"Moved {src_neg} -> {dst_neg}")
