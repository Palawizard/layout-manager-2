import sqlite3
import json
import sys

db = sys.argv[1] if len(sys.argv) > 1 else r"C:\Users\super\AppData\Roaming\com.layoutmanager2.app\layout-manager-2.sqlite"
name_filter = sys.argv[2] if len(sys.argv) > 2 else "%Palawi%"

conn = sqlite3.connect(db)
layouts = conn.execute("SELECT id, name FROM layouts WHERE name LIKE ?", (name_filter,)).fetchall()
print("layouts:", layouts)
for layout_id, layout_name in layouts:
    print(f"\n=== {layout_name} ({layout_id}) ===")
    rows = conn.execute(
        "SELECT position, kind, payload FROM layout_actions WHERE layout_id = ? ORDER BY position",
        (layout_id,),
    ).fetchall()
    for position, kind, payload in rows:
        print(f"\n[{position}] {kind}")
        print(json.dumps(json.loads(payload), indent=2, ensure_ascii=False))
