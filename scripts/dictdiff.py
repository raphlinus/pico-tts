#!/usr/bin/env python3
"""Compare letter-to-sound output of two pico-tts binaries over a word list.

Usage: dictdiff.py OLD_BIN NEW_BIN OUT.tsv [WORDS]

Writes word, old, new (ASCII phonemes) for every word whose output differs.
ERR means the binary failed on that word. To diff against a commit:

    git worktree add /tmp/old <commit>
    (cd /tmp/old && cargo build --release)
    scripts/dictdiff.py /tmp/old/target/release/pico-tts target/release/pico-tts diff.tsv
"""
import subprocess, sys

old, new, out = sys.argv[1:4]
src = sys.argv[4] if len(sys.argv) > 4 else '/usr/share/dict/words'
words = sorted({w.strip().lower() for w in open(src) if w.strip().isalpha()})


def run(b, ws):
    # Words are separated by commas so that no cross-word rule fires (and
    # not periods, which would make "mr." an abbreviation).
    r = subprocess.run([b, 'text', '--ascii', ', '.join(ws)],
                       capture_output=True, text=True)
    if r.returncode:
        return None
    out = r.stdout.strip().removesuffix(' .')
    parts = [p.strip() for p in out.split(' ,') if p.strip()]
    return parts if len(parts) == len(ws) else None


diffs = []
N = 500
for i in range(0, len(words), N):
    ws = words[i:i + N]
    a, b = run(old, ws), run(new, ws)
    if a is None or b is None:
        # A failure somewhere in the chunk: redo it word by word.
        a = [(run(old, [w]) or ['ERR'])[0] for w in ws]
        b = [(run(new, [w]) or ['ERR'])[0] for w in ws]
    diffs += [(w, x, y) for w, x, y in zip(ws, a, b) if x != y]
with open(out, 'w') as f:
    for w, x, y in diffs:
        f.write(f"{w}\t{x}\t{y}\n")
print(f"{len(words)} words, {len(diffs)} differ")
