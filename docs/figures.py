"""Draws the README's figures as SVG: run `python3 docs/figures.py` from the workspace root."""
import math
from pathlib import Path

CHAIN, BLUE, RED, GREY, AMBER, INK, MUTED = "#1b2b28", "#2c63d6", "#cf3b33", "#9aa3b2", "#d9811a", "#1b2b28", "#5a6d69"
FONT = 'font-family="Helvetica, Arial, sans-serif"'

def svg(w, h, body):
    return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}" {FONT} font-size="13">\n<rect width="{w}" height="{h}" fill="white"/>\n{body}</svg>\n'

def block(x, y, fill="white", stroke=CHAIN, r=11, width=2):
    return f'<circle cx="{x}" cy="{y}" r="{r}" fill="{fill}" stroke="{stroke}" stroke-width="{width}"/>\n'

def edge(a, b, stroke=CHAIN, width=2, dash=""):
    (x1, y1), (x2, y2) = a, b
    d = f' stroke-dasharray="{dash}"' if dash else ""
    return f'<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="{stroke}" stroke-width="{width}"{d}/>\n'

def cone(points, colour, r=22, fill_opacity=0.08):
    """A smooth outline around `points`: their convex hull, pushed out by `r`, with round corners."""
    pts = sorted(set(points))
    cross = lambda o, a, b: (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])
    lower, upper = [], []
    for q in pts:
        while len(lower) >= 2 and cross(lower[-2], lower[-1], q) <= 0:
            lower.pop()
        lower.append(q)
    for q in reversed(pts):
        while len(upper) >= 2 and cross(upper[-2], upper[-1], q) <= 0:
            upper.pop()
        upper.append(q)
    hull = lower[:-1] + upper[:-1]
    d, first = "", None
    for i, a in enumerate(hull):
        b = hull[(i + 1) % len(hull)]
        dx, dy = b[0] - a[0], b[1] - a[1]
        n = math.hypot(dx, dy)
        nx, ny = dy / n * r, -dx / n * r
        start, end = (a[0] + nx, a[1] + ny), (b[0] + nx, b[1] + ny)
        d += f"M{start[0]:.1f},{start[1]:.1f} " if i == 0 else f"A{r},{r} 0 0 1 {start[0]:.1f},{start[1]:.1f} "
        d += f"L{end[0]:.1f},{end[1]:.1f} "
        first = first or start
    d += f"A{r},{r} 0 0 1 {first[0]:.1f},{first[1]:.1f} Z"
    return f'<path d="{d}" fill="{colour}" fill-opacity="{fill_opacity}" stroke="{colour}" stroke-opacity="0.35" stroke-dasharray="3 3"/>\n'

def curve(a, b, stroke, width=1.3, dash="4 3"):
    (x1, y1), (x2, y2) = a, b
    mx = (x1 + x2) / 2
    return f'<path d="M{x1},{y1} C{mx},{y1} {mx},{y2} {x2},{y2}" fill="none" stroke="{stroke}" stroke-width="{width}" stroke-dasharray="{dash}"/>\n'

def text(x, y, s, fill=INK, size=13, anchor="start", weight="normal"):
    return f'<text x="{x}" y="{y}" fill="{fill}" font-size="{size}" text-anchor="{anchor}" font-weight="{weight}">{s}</text>\n'


# Figures 0 and 0b: a block's record, as its parent S knew it and as B knows it, and the same record
# after folding one fork. Chain blocks carry one unit of work each; at a fork the side is the chain
# above it plus the rivals recorded above it, the rival is the work recorded behind its branch.
# Rival blocks as (dx, dy, parent): the parent is the index of an earlier block of the branch, or
# None for the fork block. The branch at height 2 forks internally: two blocks on its first.
RIVALS = {2: [(26, -54, None), (82, -78, 0), (88, -34, 0)], 4: [(30, -58, None), (86, -76, 0)], 6: [(34, -56, None), (96, -70, 0)]}
def record(y, tip, known, gauges, labels, merged=None, folded=None, changed=()):
    """One row: the chain to `tip`, the rival branches with `known[fork]` blocks each, a gauge of
    side/rival per fork, and, if `folded`, the fork at 2 folded away behind the threshold."""
    b = ""
    X = lambda i: 56 + i * 72
    if folded is not None:
        tx = (X(2) + X(3)) / 2
        b += f'<rect x="{X(0)-28}" y="{y-28}" width="{X(2)-X(0)+56}" height="56" rx="28" fill="{AMBER}" fill-opacity="0.12"/>\n'
        b += edge((tx, y - 120), (tx, y + 120), stroke=AMBER, width=2, dash="6 5")
        b += text(tx, y - 130, "folding threshold", fill=AMBER, anchor="middle", weight="bold")
    for i in range(tip):
        b += edge((X(i), y), (X(i + 1), y))
    for i in range(tip + 1):
        b += block(X(i), y, fill=("#fbe9d3" if folded is not None and i <= 2 else "white"), r=11, width=(3 if i == tip else 2))
    for i, name in labels.items():
        b += text(X(i), y - 22, name, anchor="middle", weight="bold")
    for fork, pts in RIVALS.items():
        if folded is not None and fork == 2:
            continue
        n = known.get(fork, 0)
        at = [(X(fork) + dx, y + dy) for dx, dy, _ in pts[:n]]
        for k, (dx, dy, parent) in enumerate(pts[:n]):
            b += edge(at[parent] if parent is not None else (X(fork), y), at[k], stroke=RED)
        for k, (dx, dy, parent) in enumerate(pts[:n]):
            p = at[k]
            b += block(*p, fill=RED, stroke=RED, r=8)
            if merged == (fork, k):
                b += block(*p, fill="none", stroke=RED, r=13, width=1.5)
                b += curve((X(tip), y - 11), (p[0] + 8, p[1] + 6), RED)
                b += text(p[0] + 26, p[1] - 18, "merged by B", fill=RED, size=11, anchor="middle")
        b += text(X(fork), y + 28, "fork", fill=MUTED, anchor="middle", size=11)
    u = 6
    for fork, (side, riv) in gauges.items():
        gx, gy = X(fork) - 20, y + 48
        hs, hr = side in changed and (fork, "side") in changed or (fork, "side") in changed, (fork, "rival") in changed
        b += f'<rect x="{gx}" y="{gy + 64 - side * u}" width="14" height="{side * u}" fill="{CHAIN}" rx="3"/>\n'
        b += f'<rect x="{gx + 20}" y="{gy + 64 - riv * u}" width="14" height="{riv * u}" fill="{RED}" rx="3"/>\n'
        b += text(gx + 7, gy + 58 - side * u, str(side), fill=(AMBER if hs else INK), anchor="middle", size=12, weight="bold")
        b += text(gx + 27, gy + 58 - riv * u, str(riv), fill=(AMBER if hr else RED), anchor="middle", size=12, weight="bold")
        b += text(gx + 7, gy + 78, "side", fill=MUTED, anchor="middle", size=10)
        b += text(gx + 27, gy + 78, "rival", fill=MUTED, anchor="middle", size=10)
    if folded is not None:
        gx, gy = X(1) - 7, y + 48
        b += f'<rect x="{gx}" y="{gy + 64 - folded * u}" width="14" height="{folded * u}" fill="{AMBER}" rx="3"/>\n'
        b += text(gx + 7, gy + 58 - folded * u, str(folded), fill=AMBER, anchor="middle", size=12, weight="bold")
        b += text(gx + 7, gy + 78, "folded", fill=AMBER, anchor="middle", size=10)
    return b

def perception():
    b = ""
    # What S knew: forks at 2, 4 and 6, one rival block at 6 so far.
    b += text(28, 30, "what S knows", fill=MUTED, weight="bold")
    b += record(150, 8, {2: 3, 4: 2, 6: 1}, {2: (8, 3), 4: (4, 2), 6: (1, 1)}, {8: "S"})
    # What B knows: S's block has joined every side below it, and the block B merges the rival at 6.
    b += text(28, 330, "what B knows", fill=MUTED, weight="bold")
    b += text(132, 330, "every side grew by S's block; the rival at the last fork by the block B merges", fill=MUTED, size=12)
    b += record(450, 9, {2: 3, 4: 2, 6: 2}, {2: (10, 3), 4: (6, 2), 6: (2, 2)}, {8: "S", 9: "B"}, merged=(6, 1),
                changed=[(2, "side"), (4, "side"), (6, "side"), (6, "rival")])
    return svg(760, 600, b)

def folding():
    b = ""
    b += record(180, 9, {4: 2, 6: 2}, {4: (6, 2), 6: (2, 2)}, {8: "S", 9: "B"}, folded=3)
    b += text(56 + 72, 330, "decided: 10 against 3, the rival's 3 folds", fill=AMBER, anchor="middle", size=12)
    b += text(56 + 72 * 5, 330, "the forks still open, judged by every block", fill=MUTED, anchor="middle", size=12)
    return svg(760, 350, b)

# Figure 1: a block's forks, the folding threshold, blue and red.
def forks():
    b = ""
    X = lambda i: 70 + i * 68
    Y = 170
    # The decided part of the chain, shaded.
    b += f'<rect x="{X(0)-30}" y="{Y-28}" width="{X(3)-X(0)+56}" height="56" rx="28" fill="{AMBER}" fill-opacity="0.12"/>\n'
    # Chain edges and blocks.
    for i in range(9):
        b += edge((X(i), Y), (X(i + 1), Y))
    for i in range(10):
        b += block(X(i), Y, fill=("#fbe9d3" if i <= 3 else "white"), stroke=CHAIN, r=12, width=(3 if i == 9 else 2))
    b += text(X(8), Y - 24, "S", anchor="middle", weight="bold")
    b += text(X(9), Y - 24, "B", anchor="middle", weight="bold")
    # Folding threshold between heights 3 and 4.
    tx = (X(3) + X(4)) / 2
    b += edge((tx, 60), (tx, 300), stroke=AMBER, width=2, dash="6 5")
    b += text(tx, 48, "folding threshold", fill=AMBER, anchor="middle", weight="bold")
    b += text(tx - 8, 318, "decided", fill=AMBER, anchor="end", size=12)
    b += text(tx + 8, 318, "open", fill=MUTED, anchor="start", size=12)
    # A red lineage leaving at height 2, below the threshold.
    r = [(X(2) + 34, Y + 62), (X(2) + 102, Y + 92), (X(2) + 170, Y + 104)]
    b += edge((X(2), Y), r[0], stroke=RED) + edge(r[0], r[1], stroke=RED) + edge(r[1], r[2], stroke=RED)
    for p in r:
        b += block(*p, fill=RED, stroke=RED)
    b += curve((X(9), Y + 10), (r[2][0] + 8, r[2][1] - 8), RED)
    b += text(r[2][0] + 16, r[2][1] + 28, "red: votes on a fork decided before it arrived", fill=RED, size=12)
    b += text(X(2), Y + 30, "fork", fill=MUTED, anchor="middle", size=11)
    # A blue lineage leaving at height 6, above the threshold.
    c = [(X(6) + 34, Y - 62), (X(6) + 102, Y - 80)]
    b += edge((X(6), Y), c[0], stroke=BLUE) + edge(c[0], c[1], stroke=BLUE)
    for p in c:
        b += block(*p, fill=BLUE, stroke=BLUE)
    b += curve((X(9), Y - 10), (c[1][0] + 8, c[1][1] + 8), BLUE)
    b += text(c[0][0] - 100, c[1][1] - 24, "blue: votes on a fork still open", fill=BLUE, size=12)
    b += text(X(6), Y + 30, "fork", fill=MUTED, anchor="middle", size=11)
    return svg(760, 340, b)

# Figure 2: judging one fork: the side's cone against the rival's.
def judging():
    b = ""
    kx, ky = 90, 170
    side = [(kx + 70 * i, ky) for i in range(1, 5)]
    above = [(side[0][0] + 35, ky - 52), (side[1][0] + 35, ky - 58), (side[2][0] + 35, ky - 50)]
    rival = [(kx + 70, ky + 72), (kx + 140, ky + 92), (kx + 210, ky + 104)]
    beside = [(rival[0][0] + 35, ky + 130), (rival[1][0] + 35, ky + 140)]
    # Cones.
    b += cone(side + above, BLUE)
    b += cone(rival + beside, RED, fill_opacity=0.07)
    # The side: chain above the fork, rivals recorded above it beside.
    prev = (kx, ky)
    for p in side:
        b += edge(prev, p); prev = p
    for p, q in zip(side, above):
        b += edge(p, q, stroke=GREY, width=1.3)
    for p in side:
        b += block(*p)
    for q in above:
        b += block(*q, fill=GREY, stroke=GREY, r=8)
    # The rival: its deepest chain, the rest beside.
    prev = (kx, ky)
    for p in rival:
        b += edge(prev, p, stroke=RED); prev = p
    for p, q in zip(rival[:2], beside):
        b += edge(p, q, stroke=GREY, width=1.3)
    for p in rival:
        b += block(*p, fill=RED, stroke=RED)
    for q in beside:
        b += block(*q, fill=GREY, stroke=GREY, r=8)
    b += block(kx, ky, fill="#fbe9d3", r=13, width=3)
    b += text(kx, ky + 34, "the fork", anchor="middle", size=12, fill=MUTED)
    b += text(side[-1][0] + 40, ky - 40, "my side", fill=BLUE, weight="bold")
    b += text(side[-1][0] + 40, ky - 22, "the chain above the fork, c", fill=MUTED, size=12)
    b += text(side[-1][0] + 40, ky - 6, "with the rivals recorded above it, r", fill=MUTED, size=12)
    b += text(rival[-1][0] + 46, ky + 96, "the rival", fill=RED, weight="bold")
    b += text(rival[-1][0] + 46, ky + 114, "its deepest chain, c", fill=MUTED, size=12)
    b += text(rival[-1][0] + 46, ky + 130, "with the rest of its work beside it, r", fill=MUTED, size=12)
    b += text(380, 40, "each cone weighs  min(c, (c + r) / E) + r", anchor="middle", weight="bold")
    b += text(380, 60, "E: the network's width, blocks per chain step, learned from decided history", anchor="middle", size=12, fill=MUTED)
    return svg(760, 345, b)

# Figure 3: wide, not long: a lone chain against an entangled cone of the same work.
def width():
    b = ""
    y = 80
    lone = [(60 + i * 60, y) for i in range(7)]
    b += cone(lone, RED, fill_opacity=0.07)
    for a, c in zip(lone, lone[1:]):
        b += edge(a, c, stroke=RED)
    for p in lone:
        b += block(*p, fill=RED, stroke=RED)
    b += text(60, y + 38, "a lone chain: 7 blocks, 7 chain steps, width 1", fill=RED, size=12)
    b += text(60, y + 56, "weighs 1 / E of its work", fill=RED, size=12, weight="bold")
    # An entangled cone: 3 chain steps, 7 blocks.
    y2 = 230
    chain = [(60, y2), (150, y2), (240, y2)]
    others = [(105, y2 - 48), (195, y2 - 52), (120, y2 + 50), (210, y2 + 46)]
    b += cone(chain + others, BLUE)
    for a, c in zip(chain, chain[1:]):
        b += edge(a, c, stroke=BLUE)
    for o in others:
        b += edge(o, (chain[1][0] + (20 if o[0] > 150 else -20), y2), stroke=GREY, width=1.3)
        b += edge(o, (chain[2][0] if o[0] > 150 else chain[1][0], y2), stroke=GREY, width=1.3)
    for p in chain:
        b += block(*p, fill=BLUE, stroke=BLUE)
    for o in others:
        b += block(*o, fill=GREY, stroke=GREY, r=9)
    b += text(300, y2 - 6, "an entangled cone: 7 blocks, 3 chain steps, width 2.3", fill=BLUE, size=12)
    b += text(300, y2 + 12, "weighs in full once as wide as the network", fill=BLUE, size=12, weight="bold")
    return svg(760, 320, b)


# Figure 4: one fork, two verdicts. A private chain and the honest branch merge each other; the
# honest tip leads and holds the private work, the private tip trails and holds nothing.
def verdict():
    b = ""
    fx, fy = 80, 190
    honest = [(160, 160), (240, 145), (320, 135), (400, 130)]
    beside = [(195, 105), (275, 92), (355, 86), (238, 180), (318, 172)]
    private = [(160, 232), (240, 256), (320, 272)]
    h5, a4 = (480, 128), (400, 282)
    # The honest branch: a chain with work beside it.
    prev = (fx, fy)
    for p in honest:
        b += edge(prev, p, stroke=BLUE); prev = p
    for q, p in zip(beside, [honest[0], honest[1], honest[2], honest[1], honest[2]]):
        b += edge(q, p, stroke=GREY, width=1.3)
    b += edge(honest[-1], h5, stroke=BLUE)
    # The private chain.
    prev = (fx, fy)
    for p in private:
        b += edge(prev, p, stroke=RED); prev = p
    b += edge(private[-1], a4, stroke=RED)
    # H merges the hidden tip once revealed; A may list only honest blocks no heavier than its own tip.
    b += curve((h5[0] - 4, h5[1] + 12), (private[-1][0] + 6, private[-1][1] - 8), BLUE)
    b += curve((a4[0] - 2, a4[1] - 12), (honest[1][0] + 6, honest[1][1] + 10), RED)
    for p in honest:
        b += block(*p, fill=BLUE, stroke=BLUE)
    for q in beside:
        b += block(*q, fill=GREY, stroke=GREY, r=8)
    for p in private:
        b += block(*p, fill=RED, stroke=RED)
    b += block(*h5, fill="white", stroke=BLUE, width=3)
    b += block(*a4, fill="white", stroke=RED, width=3)
    b += block(fx, fy, fill="#fbe9d3", r=13, width=3)
    b += text(fx, fy + 34, "the fork", fill=MUTED, anchor="middle", size=12)
    b += text(h5[0], h5[1] - 22, "H", anchor="middle", weight="bold", fill=BLUE)
    b += text(a4[0], a4[1] + 30, "A", anchor="middle", weight="bold", fill=RED)
    b += text(260, 60, "the honest branch: 9 blocks", fill=BLUE, anchor="middle", size=12)
    b += text(250, 312, "a hidden chain: 3 blocks", fill=RED, anchor="middle", size=12)
    b += text(h5[0] + 8, h5[1] + 34, "H merges A", fill=BLUE, size=11, anchor="middle")
    b += text(h5[0] + 8, h5[1] + 48, "once revealed", fill=BLUE, size=11, anchor="middle")
    b += text(500, 206, "A may list only honest blocks", fill=RED, size=11, anchor="middle")
    b += text(500, 220, "no heavier than its own tip", fill=RED, size=11, anchor="middle")
    # The two verdicts.
    def panel(x, y, col, title, lines):
        out = f'<rect x="{x}" y="{y}" width="230" height="96" rx="10" fill="white" stroke="{col}" stroke-width="1.5"/>\n'
        out += text(x + 12, y + 22, title, fill=col, weight="bold", size=13)
        for k, l in enumerate(lines):
            out += text(x + 12, y + 44 + k * 18, l, fill=INK, size=12)
        return out
    b += panel(520, 40, BLUE, "H judges the fork", ["side 9, rival 3: it leads", "the hidden 3 is held", "blue work: 9 + 3"])
    b += panel(520, 230, RED, "A judges the fork", ["side 3, rival 4: it trails", "the honest 4 is evidence", "blue work: 3 / E, a lone chain"])
    return svg(760, 345, b)


# Figure 5: sealed off. The honest majority has folded the fork; from then on the hidden blocks are
# red in every honest record, and honest work is evidence or settled in the hidden record.
def sealed():
    b = ""
    fx, fy = 110, 190
    honest = [(190, 165), (270, 148), (350, 138), (430, 132)]
    beside = [(230, 100), (310, 92), (390, 88)]
    hidden = [(190, 240), (270, 262), (350, 278)]
    h5, a4 = (510, 128), (430, 290)
    # The honest record's decided part, and its threshold, now above the fork.
    b += f'<path d="M{fx-30},{fy+26} L{fx-30},{fy-26} L{honest[1][0]+22},{honest[1][1]-28} L{honest[1][0]+22},{honest[1][1]+30} Z" fill="{AMBER}" fill-opacity="0.12"/>\n'
    b += edge((312, 60), (312, 314), stroke=AMBER, width=2, dash="6 5")
    b += text(312, 50, "honest folding threshold", fill=AMBER, anchor="middle", weight="bold", size=12)
    # The mirror between the two lineages.
    b += edge((140, 208), (540, 208), stroke=GREY, width=1, dash="2 4")
    b += text(300, 334, "the mirror: neither lineage counts the other's work", fill=MUTED, anchor="middle", size=12)
    # Honest branch.
    prev = (fx, fy)
    for p in honest:
        b += edge(prev, p, stroke=BLUE); prev = p
    for q, p in zip(beside, honest[:3]):
        b += edge(q, p, stroke=GREY, width=1.3)
    b += edge(honest[-1], h5, stroke=BLUE)
    # Hidden chain.
    prev = (fx, fy)
    for p in hidden:
        b += edge(prev, p, stroke=RED); prev = p
    b += edge(hidden[-1], a4, stroke=RED)
    # H merges a hidden block: red. A merges an honest block built after the decision: settled.
    b += curve((h5[0] - 4, h5[1] + 12), (hidden[1][0] + 6, hidden[1][1] - 8), BLUE)
    b += curve((a4[0] - 2, a4[1] - 12), (honest[2][0] + 6, honest[2][1] + 10), RED)
    for p in honest:
        b += block(*p, fill=BLUE, stroke=BLUE)
    for q in beside:
        b += block(*q, fill=GREY, stroke=GREY, r=8)
    for p in hidden:
        b += block(*p, fill=RED, stroke=RED)
    b += block(*honest[2], fill=GREY, stroke=GREY)
    b += block(*honest[2], fill="none", stroke=GREY, r=16, width=1.5)
    b += block(*hidden[1], fill="none", stroke=RED, r=16, width=1.5, )
    b += block(*h5, fill="white", stroke=BLUE, width=3)
    b += block(*a4, fill="white", stroke=RED, width=3)
    b += block(fx, fy, fill="#fbe9d3", r=13, width=3)
    b += text(fx, fy + 34, "the fork, decided", fill=AMBER, anchor="middle", size=12)
    b += text(h5[0], h5[1] - 22, "H", anchor="middle", weight="bold", fill=BLUE)
    b += text(a4[0], a4[1] + 30, "A", anchor="middle", weight="bold", fill=RED)
    b += text(hidden[1][0], hidden[1][1] + 34, "red", fill=RED, anchor="middle", size=11)
    b += text(honest[2][0] + 2, honest[2][1] - 24, "settled", fill=MUTED, anchor="middle", size=11)
    def panel(x, y, col, title, lines):
        out = f'<rect x="{x}" y="{y}" width="200" height="96" rx="10" fill="white" stroke="{col}" stroke-width="1.5"/>\n'
        out += text(x + 12, y + 22, title, fill=col, weight="bold", size=13)
        for k, l in enumerate(lines):
            out += text(x + 12, y + 44 + k * 18, l, fill=INK, size=12)
        return out
    b += panel(552, 40, BLUE, "in every honest record", ["the hidden blocks join", "below the threshold:", "red, and red is final"])
    b += panel(552, 236, RED, "in the hidden record", ["honest work is evidence", "while A trails, settled", "once built after the fold"])
    return svg(760, 345, b)

here = Path(__file__).resolve().parent
for name, fig in [("perception", perception), ("folding", folding), ("forks", forks), ("judging", judging), ("verdict", verdict), ("sealed", sealed), ("width", width)]:
    (here / f"{name}.svg").write_text(fig())
    print(name)
