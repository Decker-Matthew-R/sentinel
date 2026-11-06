from PIL import Image, ImageDraw
import math, os, shutil

BG = (0x0f, 0x17, 0x2a, 255)
FG = (0x22, 0xc5, 0x5e, 255)
S = 2048  # supersample canvas

def shield_points(cx, top, w, h, n=60):
    # flat-ish top with slight notch-free shoulders, sides curving to a point
    hw = w / 2
    pts = []
    shoulder = top + h * 0.10
    pts.append((cx - hw, shoulder))
    # top edge: gentle inward dip toward center
    for i in range(n + 1):
        t = i / n
        x = cx - hw + t * w
        y = shoulder - (h * 0.10) * math.sin(math.pi * t) ** 0.6
        pts.append((x, y))
    # right side down to point (quadratic bezier)
    p0, p1, p2 = (cx + hw, shoulder), (cx + hw, top + h * 0.72), (cx, top + h)
    for i in range(1, n + 1):
        t = i / n
        x = (1-t)**2*p0[0] + 2*(1-t)*t*p1[0] + t*t*p2[0]
        y = (1-t)**2*p0[1] + 2*(1-t)*t*p1[1] + t*t*p2[1]
        pts.append((x, y))
    p0, p1, p2 = (cx, top + h), (cx - hw, top + h * 0.72), (cx - hw, shoulder)
    for i in range(1, n + 1):
        t = i / n
        x = (1-t)**2*p0[0] + 2*(1-t)*t*p1[0] + t*t*p2[0]
        y = (1-t)**2*p0[1] + 2*(1-t)*t*p1[1] + t*t*p2[1]
        pts.append((x, y))
    return pts

def thick_polyline(d, pts, width, color, closed=False):
    if closed:
        pts = pts + [pts[0]]
    d.line(pts, fill=color, width=int(width), joint="curve")
    r = width / 2
    for x, y in pts:
        d.ellipse((x - r, y - r, x + r, y + r), fill=color)

def render(round_mask=False):
    img = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    if round_mask:
        d.ellipse((0, 0, S - 1, S - 1), fill=BG)
    else:
        d.rounded_rectangle((0, 0, S - 1, S - 1), radius=int(S * 0.18), fill=BG)

    cx = S / 2
    sw, sh = S * 0.56, S * 0.66
    top = (S - sh) / 2 - S * 0.01
    stroke = S * 0.055
    thick_polyline(d, shield_points(cx, top, sw, sh), stroke, FG, closed=True)

    # pulse line: crosses the shield horizontally, slightly above center
    y = top + sh * 0.47
    L, R = cx - sw * 0.62, cx + sw * 0.62
    u = sw / 10
    pulse = [
        (L, y),
        (cx - 2.2*u, y),
        (cx - 1.3*u, y - 1.3*u),
        (cx - 0.2*u, y + 2.6*u),
        (cx + 0.9*u, y - 3.2*u),
        (cx + 1.8*u, y + 0.9*u),
        (cx + 2.4*u, y),
        (R, y),
    ]
    # knock out a gap of background behind pulse where it crosses the shield edge for legibility
    thick_polyline(d, pulse, stroke * 1.9, BG)
    thick_polyline(d, pulse, stroke, FG)
    # restore shield edge visibility beyond the knock-out is intentional (gap effect)
    return img

DENS = {"mdpi": 48, "hdpi": 72, "xhdpi": 96, "xxhdpi": 144, "xxxhdpi": 192}
base = render()
base_round = render(round_mask=True)
out = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "res")

for name, px in DENS.items():
    p = f"{out}/mipmap-{name}"
    os.makedirs(p, exist_ok=True)
    base.resize((px, px), Image.LANCZOS).save(f"{p}/ic_launcher.png", optimize=True)
    base_round.resize((px, px), Image.LANCZOS).save(f"{p}/ic_launcher_round.png", optimize=True)
base.resize((512, 512), Image.LANCZOS).save("ic_launcher_512_preview.png")

# preview sheet: all sizes at native res on a light + dark strip
sheet = Image.new("RGBA", (700, 460), (240, 240, 240, 255))
dk = ImageDraw.Draw(sheet); dk.rectangle((0, 230, 700, 460), fill=(30, 30, 30, 255))
x = 20
for name, px in DENS.items():
    ic = base.resize((px, px), Image.LANCZOS)
    for yoff in (20, 250):
        sheet.alpha_composite(ic, (x, yoff + (192 - px)))
    x += px + 30
sheet.save("preview_sheet.png")
