#!/usr/bin/env python3
"""Compare letter-to-sound output against CMUdict.

Usage: cmucheck.py BIN CMUDICT OUT.tsv [TEXT...]

With TEXT files, checks the words that occur in them, ranked by how often
they occur. Without, checks every CMUdict word that is also in
/usr/share/dict/words.

Pronunciations are compared as phoneme strings with a weighted edit
distance. Allophones that our front end produces (flap, syllabic L/N,
r-coloring) are mapped back to phonemes, and small, intelligible
deviations cost little: weak unstressed vowels (I/x/E), the cot-caught
merger (a/c), and t/d around a flap. Stress is compared separately (the
vowel carrying primary stress). A word passes if it matches any of
CMUdict's variants.

Output columns: word, count, cost, stress_ok, ours, cmu (best variant).
Sorted by count * cost, so the most damaging misses come first.
"""
import collections, re, subprocess, sys

ARPA = {
    'AA': 'a', 'AE': '@', 'AO': 'c', 'AW': 'W', 'AY': 'A', 'EH': 'E',
    'ER': 'R', 'EY': 'e', 'IH': 'I', 'IY': 'i', 'OW': 'o', 'OY': 'O',
    'UH': 'U', 'UW': 'u', 'B': 'b', 'CH': 'C', 'D': 'd', 'DH': 'D',
    'F': 'f', 'G': 'g', 'HH': 'h', 'JH': 'J', 'K': 'k', 'L': 'l', 'M': 'm',
    'N': 'n', 'NG': 'G', 'P': 'p', 'R': 'r', 'S': 's', 'SH': 'S', 'T': 't',
    'TH': 'T', 'V': 'v', 'W': 'w', 'Y': 'y', 'Z': 'z', 'ZH': 'Z',
}
VOWELS = set('iIeE@aAW^coOUuxR')


def cmu_to_seq(phones):
    """CMU phones to (symbols, index of primary-stressed vowel)."""
    seq, stress = [], None
    for ph in phones:
        base, digit = re.match(r'([A-Z]+)([012]?)', ph).groups()
        if base == 'AH':
            c = '^' if digit in ('1', '2') else 'x'
        else:
            c = ARPA[base]
        if digit == '1' and stress is None:
            stress = len(seq)
        seq.append(c)
    return seq, stress


def ours_to_seq(word_ascii):
    """Our ASCII output for one word to (symbols, primary-stress index)."""
    seq, stress, pending = [], None, False
    for ch in word_ascii:
        if ch == "'":
            pending = True
            continue
        if ch in '"-':
            continue
        if ch == 'L':
            out = ['x', 'l']
        elif ch == 'N':
            out = ['x', 'n']
        elif ch == '&':
            out = ['&']
        elif ch == 'Y':
            out = ['y', 'u']
        else:
            out = [ch]
        if pending and out[0] in VOWELS:
            if stress is None:
                stress = len(seq)
            pending = False
        seq += out
    # Post-vocalic R (r-coloring) is a consonant r.
    for i in range(1, len(seq)):
        if seq[i] == 'R' and seq[i - 1] in VOWELS and seq[i - 1] != 'R':
            seq[i] = 'r'
    return seq, stress


def norm_cmu(seq):
    seq = list(seq)
    # CMU writes "here" as IH1 R / IY1 R; r-coloring after a vowel is r.
    for i in range(1, len(seq)):
        if seq[i] == 'R' and seq[i - 1] in VOWELS and seq[i - 1] != 'R':
            seq[i] = 'r'
    return seq


NEAR = [set('Ix'), set('xE'), set('IE'), set('ac'), set('x^'), set('Ii'),
        set('xR'), set('iy')]


def sub_cost(a, b):
    if a == b:
        return 0.0
    if a == '&' and b in 'td' or b == '&' and a in 'td':
        return 0.0
    if any(a in s and b in s for s in NEAR):
        return 0.3
    if a in VOWELS and b in VOWELS:
        return 1.0
    if a in 'td' and b in 'td' or a in 'sz' and b in 'sz':
        return 0.5
    return 1.0


def indel_cost(a):
    # A dropped or inserted schwa is minor; consonants and full vowels not.
    return 0.4 if a in 'xh' else 1.0


def distance(a, b):
    n, m = len(a), len(b)
    d = [[0.0] * (m + 1) for _ in range(n + 1)]
    for i in range(1, n + 1):
        d[i][0] = d[i - 1][0] + indel_cost(a[i - 1])
    for j in range(1, m + 1):
        d[0][j] = d[0][j - 1] + indel_cost(b[j - 1])
    for i in range(1, n + 1):
        for j in range(1, m + 1):
            d[i][j] = min(d[i - 1][j] + indel_cost(a[i - 1]),
                          d[i][j - 1] + indel_cost(b[j - 1]),
                          d[i - 1][j - 1] + sub_cost(a[i - 1], b[j - 1]))
    return d[n][m]


def vowel_index(seq, i):
    """Which vowel (counting vowels) is at position i."""
    if i is None:
        return None
    return sum(1 for c in seq[:i] if c in VOWELS)


def run(binary, words):
    r = subprocess.run([binary, 'text', '--ascii', '. '.join(words) + '.'],
                       capture_output=True, text=True)
    if r.returncode:
        return None
    parts = [p.strip() for p in r.stdout.strip().split(' .') if p.strip()]
    if len(parts) != len(words):
        return None
    return [re.sub(r'^[FC]: ', '', p) for p in parts]


def main():
    binary, cmu_path, out = sys.argv[1:4]
    texts = sys.argv[4:]
    cmu = collections.defaultdict(list)
    for line in open(cmu_path, encoding='latin-1'):
        if line.startswith(';;;'):
            continue
        parts = line.split('#')[0].split()
        if not parts:
            continue
        w = re.sub(r'\(\d+\)$', '', parts[0])
        if w.isalpha():
            cmu[w.lower()].append(parts[1:])
    if texts:
        counts = collections.Counter()
        for t in texts:
            for w in re.findall(r"[A-Za-z]+(?:'[a-z]+)?", open(t).read()):
                w = w.lower().replace("'", '')
                counts[w] += 1
    else:
        dict_words = {w.strip().lower() for w in open('/usr/share/dict/words')}
        counts = collections.Counter({w: 1 for w in cmu if w in dict_words})
    words = sorted(w for w in counts if w in cmu)
    missing = sum(1 for w in counts if w not in cmu)
    rows = []
    N = 500
    for i in range(0, len(words), N):
        ws = words[i:i + N]
        outs = run(binary, ws)
        if outs is None:
            outs = [(run(binary, [w]) or ['ERR'])[0] for w in ws]
        for w, o in zip(ws, outs):
            if o == 'ERR':
                rows.append((w, counts[w], 9.0, False, o, ' '.join(cmu[w][0])))
                continue
            ours, ostress = ours_to_seq(o)
            best = None
            for variant in cmu[w]:
                cseq, cstress = cmu_to_seq(variant)
                cseq = norm_cmu(cseq)
                cost = distance(ours, cseq)
                # Unstressed function words make no stress claim.
                sok = (ostress is None
                       or vowel_index(ours, ostress) == vowel_index(cseq, cstress))
                key = (cost, not sok)
                if best is None or key < best[0]:
                    best = (key, cost, sok, variant)
            _, cost, sok, variant = best
            rows.append((w, counts[w], cost, sok, o, ' '.join(variant)))
    rows.sort(key=lambda r: (-r[1] * (r[2] + (0 if r[3] else 0.5)), r[0]))
    with open(out, 'w') as f:
        f.write('word\tcount\tcost\tstress_ok\tours\tcmu\n')
        for r in rows:
            f.write(f'{r[0]}\t{r[1]}\t{r[2]:.1f}\t{int(r[3])}\t{r[4]}\t{r[5]}\n')
    tot = sum(counts[w] for w in words)
    bad = sum(r[1] for r in rows if r[2] >= 1.0)
    bad_s = sum(r[1] for r in rows if not r[3])
    exact = sum(r[1] for r in rows if r[2] == 0 and r[3])
    print(f'{len(words)} words in CMUdict ({missing} not), {tot} tokens')
    print(f'tokens exact (within allophones): {100 * exact / tot:.1f}%')
    print(f'tokens with cost >= 1: {100 * bad / tot:.1f}%')
    print(f'tokens with wrong primary stress: {100 * bad_s / tot:.1f}%')


if __name__ == '__main__':
    main()
