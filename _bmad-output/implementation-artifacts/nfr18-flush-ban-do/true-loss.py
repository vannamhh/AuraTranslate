#!/usr/bin/env python3
"""true-loss.py <project.db> <typing.log.keys> <kill_ts> [dump-file]
prints: k total loss_s kprefix loss_prefix_s

Primary: anchor on the highest complete "[n]" marker in the store, then extend over the keys typed
after its "]" while the store still matches the stream (a dropped earlier key cannot hide later
text). k = keys that survived; loss_s = kill_ts - stamp of the first key that did not (0 if all).
kprefix/loss_prefix_s: stricter variant, longest stream prefix present as a substring."""
import re, sqlite3, sys

db, keys, kill = sys.argv[1], sys.argv[2], float(sys.argv[3])
lines = [l for l in open(keys).read().splitlines() if l.strip()]
t0 = float(lines[0].split()[1])
rows = [l.split() for l in lines[1:]]
ts = [t0 + float(a) for a, _ in rows]
stream = "".join(chr(int(b)) for _, b in rows)
con = sqlite3.connect(f"file:{db}?mode=ro", uri=True)
texts = [(r[0] or "").replace(" ", " ") for r in con.execute("select target_text from segment")]
if len(sys.argv) > 4:
    open(sys.argv[4], "w").write("\n=====\n".join(texts))

def loss(k):
    return 0.0 if k >= len(stream) else kill - ts[k]

lo, hi = 0, len(stream)
while lo < hi:
    mid = (lo + hi + 1) // 2
    if any(stream[:mid] in t for t in texts):
        lo = mid
    else:
        hi = mid - 1

best_n, best_t, best_p = -1, None, -1
for t in texts:
    for m in re.finditer(r"\[(\d+)\]", t):
        if int(m.group(1)) > best_n:
            best_n, best_t, best_p = int(m.group(1)), t, m.end()
k = 0
if best_n >= 0:
    s = stream.find(f"[{best_n}]")
    if s >= 0:
        s += len(f"[{best_n}]")
        tail = best_t[best_p:]
        m = 0
        while s + m < len(stream) and m < len(tail) and tail[m] == stream[s + m]:
            m += 1
        k = s + m
print(k, len(stream), f"{loss(k):.3f}", lo, f"{loss(lo):.3f}")
