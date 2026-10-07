"""Draws the DAGLight logo as SVG: run `python3 docs/logo.py` from the workspace root.

Every letter is a skeleton of strokes. "DAG" is drawn solid; "Light" is the same kind of skeleton
dissolved into vertices and the edges between them, a wireframe that thins and fades to the right
inside a cone of faint vertices cast from the DAG. The layout is seeded, so the files redraw the
same."""
import math
import random
from pathlib import Path

# The letter grid: the cap height is 100, the x-height 62, the baseline at 100, y downwards.
CAP, XH, BASE = 0, 38, 100


def arc(cx, cy, r, a0, a1, n=24):
    """Points on a circle from angle `a0` to `a1`, in degrees with y downwards."""
    return [(cx + r * math.cos(math.radians(a0 + (a1 - a0) * i / n)),
             cy + r * math.sin(math.radians(a0 + (a1 - a0) * i / n))) for i in range(n + 1)]


# Each letter: its advance width and its strokes, polylines in the grid.
LETTERS = {
    "D": (86, [[(0, CAP), (0, BASE)], [(0, CAP), (30, CAP)] + arc(30, 50, 50, -90, 90) + [(0, BASE)]]),
    "A": (88, [[(0, BASE), (42, CAP), (84, BASE)], [(15, 64), (69, 64)]]),
    "G": (102, [arc(50, 50, 50, -40, -350, 40) + [(60, 58)]]),
    "L": (54, [[(0, CAP), (0, BASE), (46, BASE)]]),
    "i": (18, [[(0, XH), (0, BASE)], [(0, 12)]]),
    "g": (74, [arc(28, 69, 29, 0, 360, 28), [(57, XH), (57, 112)] + arc(29, 112, 28, 0, 125, 14)]),
    "h": (72, [[(0, CAP), (0, BASE)], [(0, 68)] + arc(28, 66, 28, 180, 360, 16) + [(56, 66), (56, BASE)]]),
    "t": (46, [[(12, 10), (12, 80)] + arc(30, 80, 18, 180, 95, 10), [(0, XH), (34, XH)]]),
}


def place(word, x, y, gap):
    """Lays the word's strokes out from `x` with the baseline at `y`; returns them and the end."""
    strokes = []
    for c in word:
        advance, letter = LETTERS[c]
        strokes.extend([[(x + px, y - BASE + py) for px, py in s] for s in letter])
        x += advance + gap
    return strokes, x - gap


def length(p, q):
    return math.hypot(q[0] - p[0], q[1] - p[1])


def sample(stroke, step, rng, jitter):
    """Vertices along a stroke every `step`, each nudged by up to `jitter`; a lone point is one."""
    if len(stroke) == 1:
        return list(stroke)
    total = sum(length(p, q) for p, q in zip(stroke, stroke[1:]))
    n = max(2, round(total / step))
    out, want, walked = [], 0.0, 0.0
    for p, q in zip(stroke, stroke[1:]):
        seg = length(p, q)
        while want <= walked + seg + 1e-9 and len(out) < n + 1:
            t = 0 if seg == 0 else (want - walked) / seg
            x, y = p[0] + (q[0] - p[0]) * t, p[1] + (q[1] - p[1]) * t
            out.append((x + rng.uniform(-jitter, jitter), y + rng.uniform(-jitter, jitter)))
            want += total / n
        walked += seg
    return out


def path(stroke):
    return "M" + " L".join(f"{x:.1f},{y:.1f}" for x, y in stroke)


def lerp(a, b, t):
    return a + (b - a) * max(0.0, min(1.0, t))


def logo(dark=True, seed=7, ground=None):
    """Returns the SVG of the logo for a dark or a light ground, transparent unless `ground` is a
    colour to paint behind it."""
    rng = random.Random(seed)
    W, H = 860, 270
    base = 178
    ink = "#f3f6fb" if dark else "#1b2b28"
    node, edge, glow = ("#e6f4ff", "#8cc4ff", "#5aa9ff") if dark else ("#2c63d6", "#5b8ae6", "#2c63d6")

    # "DAG", solid; "Light", as strokes still, a gap after the G.
    dag, dag_end = place("DAG", 40, base, 16)
    light, light_end = place("Light", dag_end + 44, base, 14)
    apex = (40, base - 50)  # the cone is cast from the D, at mid cap height

    body = [f'<rect width="{W}" height="{H}" fill="{ground}"/>'] if ground else []
    body.append('<defs>')
    peak = (dag_end + 30 - apex[0]) / (W - apex[0])
    body.append(f'<linearGradient id="cone" x1="{apex[0]}" y1="0" x2="{W}" y2="0" gradientUnits="userSpaceOnUse">'
                f'<stop offset="0" stop-color="{glow}" stop-opacity="0"/>'
                f'<stop offset="{peak:.2f}" stop-color="{glow}" stop-opacity="{0.17 if dark else 0.11}"/>'
                f'<stop offset="1" stop-color="{glow}" stop-opacity="0"/></linearGradient>')
    body.append('<filter id="soft" x="-20%" y="-20%" width="140%" height="140%">'
                '<feGaussianBlur stdDeviation="2.2"/></filter>')
    body.append('<filter id="haze" x="-10%" y="-30%" width="120%" height="160%">'
                '<feGaussianBlur stdDeviation="9"/></filter>')
    body.append('</defs>')

    # The cone, from the apex through the gap and out of the frame.
    half = math.radians(14)
    far = W + 40
    top = (far, apex[1] - (far - apex[0]) * math.tan(half))
    bottom = (far, apex[1] + (far - apex[0]) * math.tan(half))
    body.append(f'<polygon points="{apex[0]},{apex[1]} {top[0]:.0f},{top[1]:.0f} {bottom[0]:.0f},{bottom[1]:.0f}" '
                f'fill="url(#cone)" filter="url(#haze)"/>')

    # Faint vertices scattered through the cone, each hung on a neighbour to its left.
    haze = []
    for _ in range(170):
        x = dag_end + 20 + (W - dag_end - 20) * math.sqrt(rng.random())
        spread = (x - apex[0]) * math.tan(half)
        y = apex[1] + rng.uniform(-spread, spread)
        haze.append((x, y))
    haze.sort()
    fade = lambda x: lerp(1.0, 0.0, (x - dag_end) / (W - dag_end))
    for i, (x, y) in enumerate(haze):
        a = (0.22 if dark else 0.30) * fade(x) + 0.03
        near = sorted((length((x, y), q), q) for q in haze[:i] if q[0] < x - 4)[:2]
        for _, q in near:
            body.append(f'<line x1="{q[0]:.1f}" y1="{q[1]:.1f}" x2="{x:.1f}" y2="{y:.1f}" '
                        f'stroke="{edge}" stroke-width="0.7" stroke-opacity="{a * 0.6:.2f}"/>')
        body.append(f'<circle cx="{x:.1f}" cy="{y:.1f}" r="{1.6 * fade(x) + 0.8:.1f}" fill="{node}" fill-opacity="{a:.2f}"/>')

    # "Light": vertices along every stroke, joined along the stroke and to a few neighbours.
    span = light_end - light[0][0][0]
    at = lambda x: (x - light[0][0][0]) / span
    verts, strokes_v = [], []
    for s in light:
        t = at(s[0][0])
        vs = sample(s, step=lerp(13, 17, t), rng=rng, jitter=lerp(0.5, 2.6, t))
        strokes_v.append(vs)
        verts.extend(vs)
    edges = set()
    for vs in strokes_v:
        for p, q in zip(vs, vs[1:]):
            edges.add((p, q, 1.0))
    for v in verts:
        t = at(v[0])
        near = sorted((length(v, q), q) for q in verts if q != v and 8 < length(v, q) < lerp(32, 48, t))
        for _, q in near[:3 if rng.random() < lerp(0.8, 0.4, t) else 2]:
            if (q, v, 0.55) not in edges and (q, v, 1.0) not in edges and (v, q, 1.0) not in edges:
                edges.add((v, q, 0.55))
    for h in haze:
        near = [(d, q) for d, q in ((length(h, q), q) for q in verts) if d < 34]
        if near and rng.random() < 0.6:
            edges.add((h, min(near)[1], 0.22))
    soft = []
    for p, q, w in sorted(edges, key=lambda e: e[2]):
        t = at((p[0] + q[0]) / 2)
        a = lerp(0.92, 0.40, t) * w
        body.append(f'<line x1="{p[0]:.1f}" y1="{p[1]:.1f}" x2="{q[0]:.1f}" y2="{q[1]:.1f}" '
                    f'stroke="{edge}" stroke-width="{lerp(2.1, 1.0, t) * (1 if w == 1.0 else 0.55):.2f}" stroke-opacity="{a:.2f}"/>')
    for v in verts:
        t = at(v[0])
        r = lerp(2.4, 1.6, t) * (1.9 if any(len(vs) == 1 and vs[0] == v for vs in strokes_v) else 1)
        a = lerp(1.0, 0.5, t)
        soft.append(f'<circle cx="{v[0]:.1f}" cy="{v[1]:.1f}" r="{r * 1.9:.1f}" fill="{glow}" fill-opacity="{a * (0.35 if dark else 0.18):.2f}"/>')
        body.append(f'<circle cx="{v[0]:.1f}" cy="{v[1]:.1f}" r="{r:.1f}" fill="{node}" fill-opacity="{a:.2f}"/>')
    body.insert(len(body) - len(verts) - len(edges), f'<g filter="url(#soft)">{"".join(soft)}</g>')

    # "DAG", solid strokes with round ends, over everything.
    for s in dag:
        body.append(f'<path d="{path(s)}" fill="none" stroke="{ink}" stroke-width="21" '
                    f'stroke-linecap="round" stroke-linejoin="round"/>')

    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" height="{H}">\n'
            + "\n".join(body) + "\n</svg>\n")


if __name__ == "__main__":
    docs = Path(__file__).parent
    (docs / "logo.svg").write_text(logo(dark=True))
    (docs / "logo-light.svg").write_text(logo(dark=False))
