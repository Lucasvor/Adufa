"""Generate Adufa's raster icon package and README demonstration GIF.

The frames reproduce the native compact popup, output selector, and taskbar
companion for a running application, even before it has an active audio session,
without including a personal desktop capture. Keeping the demo deterministic
makes documentation updates reviewable.
"""

from __future__ import annotations

from pathlib import Path
from typing import Callable

from PIL import Image, ImageDraw, ImageFont


ROOT = Path(__file__).resolve().parents[1]
ASSETS = ROOT / "docs" / "assets"
SCALE = 2

BG = "#0F1217"
PANEL = "#1B1E23"
PANEL_2 = "#20242A"
LINE = "#3A414B"
TEXT = "#F4F7FB"
MUTED = "#AEB8C5"
ACCENT = "#78AFFF"
ACCENT_BG = "#26364B"
INACTIVE = "#7C8796"


def font(size: int, bold: bool = False) -> ImageFont.FreeTypeFont:
    candidates = [
        Path(r"C:\Windows\Fonts\segoeuib.ttf" if bold else r"C:\Windows\Fonts\segoeui.ttf"),
        Path(r"C:\Windows\Fonts\arialbd.ttf" if bold else r"C:\Windows\Fonts\arial.ttf"),
    ]
    for candidate in candidates:
        if candidate.exists():
            return ImageFont.truetype(str(candidate), size * SCALE)
    return ImageFont.load_default(size * SCALE)


def canvas(size: tuple[int, int]) -> tuple[Image.Image, ImageDraw.ImageDraw]:
    image = Image.new("RGB", (size[0] * SCALE, size[1] * SCALE), BG)
    return image, ImageDraw.Draw(image)


def box(draw: ImageDraw.ImageDraw, xy: tuple[int, int, int, int], radius: int, fill: str, outline: str | None = None, width: int = 1) -> None:
    draw.rounded_rectangle(tuple(v * SCALE for v in xy), radius * SCALE, fill=fill, outline=outline, width=width * SCALE)


def line(draw: ImageDraw.ImageDraw, points: list[tuple[int, int]], fill: str, width: int) -> None:
    draw.line([(x * SCALE, y * SCALE) for x, y in points], fill=fill, width=width * SCALE, joint="curve")


def label(draw: ImageDraw.ImageDraw, xy: tuple[int, int], value: str, size: int, fill: str = TEXT, bold: bool = False, anchor: str | None = None) -> None:
    draw.text((xy[0] * SCALE, xy[1] * SCALE), value, font=font(size, bold), fill=fill, anchor=anchor)


def brand_mark(draw: ImageDraw.ImageDraw, origin: tuple[int, int], unit: float = 1.0, background: bool = True) -> None:
    ox, oy = origin
    if background:
        box(draw, (ox, oy, int(ox + 64 * unit), int(oy + 64 * unit)), int(14 * unit), "#171A1F", "#303640")
    def p(x: float, y: float) -> tuple[int, int]:
        return int(ox + x * unit), int(oy + y * unit)
    line(draw, [p(13, 32), p(28, 32), p(41, 19), p(52, 19)], ACCENT, max(2, int(7 * unit)))
    line(draw, [p(28, 32), p(41, 45), p(47, 45)], INACTIVE, max(1, int(4 * unit)))
    line(draw, [p(47, 40), p(47, 50)], INACTIVE, max(1, int(4 * unit)))
    r = max(1, int(3.5 * unit))
    cx, cy = p(28, 32)
    draw.ellipse(((cx-r)*SCALE, (cy-r)*SCALE, (cx+r)*SCALE, (cy+r)*SCALE), fill="#EAF2FF")


def app_dot(draw: ImageDraw.ImageDraw, x: int, y: int, color: str, letter: str) -> None:
    draw.ellipse(((x-10)*SCALE, (y-10)*SCALE, (x+10)*SCALE, (y+10)*SCALE), fill=color)
    label(draw, (x, y+1), letter, 10, "#101318", True, "mm")


def application_icon(draw: ImageDraw.ImageDraw, x: int, y: int, kind: str) -> None:
    if kind == "library":
        draw.rounded_rectangle(((x-11)*SCALE, (y-8)*SCALE, (x+11)*SCALE, (y+8)*SCALE), 2*SCALE, fill="#F2F6FB")
        draw.rectangle(((x-8)*SCALE, (y-6)*SCALE, (x-3)*SCALE, (y+6)*SCALE), fill="#315B8D")
        draw.rectangle(((x-1)*SCALE, (y-5)*SCALE, (x+8)*SCALE, (y-2)*SCALE), fill="#C2CFDC")
        draw.rectangle(((x-1)*SCALE, (y+1)*SCALE, (x+8)*SCALE, (y+4)*SCALE), fill="#C2CFDC")
    elif kind == "voicemeeter":
        draw.rounded_rectangle(((x-11)*SCALE, (y-9)*SCALE, (x+11)*SCALE, (y+9)*SCALE), 2*SCALE, fill="#A93D3D")
        label(draw, (x, y-2), "VOICE", 5, "#FFFFFF", True, "mm")
        label(draw, (x, y+5), "METER", 4, "#FFFFFF", True, "mm")
    elif kind == "chrome":
        draw.ellipse(((x-11)*SCALE, (y-11)*SCALE, (x+11)*SCALE, (y+11)*SCALE), fill="#EEC13E")
        draw.pieslice(((x-11)*SCALE, (y-11)*SCALE, (x+11)*SCALE, (y+11)*SCALE), 120, 240, fill="#D9544D")
        draw.pieslice(((x-11)*SCALE, (y-11)*SCALE, (x+11)*SCALE, (y+11)*SCALE), 240, 360, fill="#4CA06A")
        draw.ellipse(((x-5)*SCALE, (y-5)*SCALE, (x+5)*SCALE, (y+5)*SCALE), fill="#4385E5")
    elif kind == "helium":
        draw.ellipse(((x-11)*SCALE, (y-11)*SCALE, (x+11)*SCALE, (y+11)*SCALE), fill="#5D8FE8")
        label(draw, (x, y), "H", 13, "#FFFFFF", True, "mm")
    elif kind == "zen":
        draw.ellipse(((x-10)*SCALE, (y-10)*SCALE, (x+10)*SCALE, (y+10)*SCALE), outline="#B8B8B2", width=2*SCALE)
        draw.ellipse(((x-5)*SCALE, (y-5)*SCALE, (x+5)*SCALE, (y+5)*SCALE), outline="#8D938C", width=2*SCALE)
    else:
        app_dot(draw, x, y, "#1ED760", "S")


def check_icon(draw: ImageDraw.ImageDraw, x: int, y: int, color: str = ACCENT) -> None:
    line(draw, [(x-5, y), (x-1, y+4), (x+6, y-5)], color, 2)


def speaker_icon(draw: ImageDraw.ImageDraw, x: int, y: int) -> None:
    draw.polygon([((x-7)*SCALE, (y-3)*SCALE), ((x-3)*SCALE, (y-3)*SCALE), ((x+1)*SCALE, (y-7)*SCALE), ((x+1)*SCALE, (y+7)*SCALE), ((x-3)*SCALE, (y+3)*SCALE), ((x-7)*SCALE, (y+3)*SCALE)], fill=ACCENT)
    line(draw, [(x+4, y-4), (x+6, y), (x+4, y+4)], ACCENT, 1)


def settings_icon(draw: ImageDraw.ImageDraw, x: int, y: int) -> None:
    draw.ellipse(((x-6)*SCALE, (y-6)*SCALE, (x+6)*SCALE, (y+6)*SCALE), outline=ACCENT, width=2*SCALE)
    draw.ellipse(((x-2)*SCALE, (y-2)*SCALE, (x+2)*SCALE, (y+2)*SCALE), outline=ACCENT, width=SCALE)


def power_icon(draw: ImageDraw.ImageDraw, x: int, y: int) -> None:
    draw.arc(((x-7)*SCALE, (y-7)*SCALE, (x+7)*SCALE, (y+7)*SCALE), 312, 228, fill=ACCENT, width=2*SCALE)
    line(draw, [(x, y-9), (x, y)], ACCENT, 2)


def locator_icon(draw: ImageDraw.ImageDraw, x: int, y: int) -> None:
    speaker_icon(draw, x-4, y)
    draw.arc(((x-2)*SCALE, (y-7)*SCALE, (x+10)*SCALE, (y+7)*SCALE), 300, 60, fill=ACCENT, width=SCALE)


def popup(draw: ImageDraw.ImageDraw, x: int, y: int, selected: int | None = None) -> None:
    box(draw, (x, y, x+304, y+328), 10, PANEL, LINE)
    speaker_icon(draw, x+24, y+29)
    label(draw, (x+50, y+22), "Adufa", 16, TEXT, True, "lm")
    label(draw, (x+50, y+41), "Audio router", 12, MUTED, False, "lm")
    locator_icon(draw, x+202, y+28)
    label(draw, (x+220, y+28), "Find sound", 12, MUTED, False, "lm")
    line(draw, [(x+14, y+58), (x+290, y+58)], LINE, 1)
    apps = [("library", "LibraryServer"), ("voicemeeter", "voicemeeter"), ("chrome", "chrome"), ("zen", "zen")]
    for index, (kind, name) in enumerate(apps):
        top = y + 68 + index * 44
        if selected == index:
            draw.rectangle((x+8*SCALE, top*SCALE, (x+296)*SCALE, (top+40)*SCALE), fill=ACCENT_BG)
            draw.rectangle((x+8*SCALE, top*SCALE, (x+11)*SCALE, (top+40)*SCALE), fill=ACCENT)
        application_icon(draw, x+27, top+20, kind)
        label(draw, (x+48, top+20), name, 13, TEXT, False, "lm")
        label(draw, (x+270, top+20), "System default", 11, MUTED, False, "rm")
        label(draw, (x+284, top+20), "›", 16, MUTED, False, "mm")
    divider = y + 246
    line(draw, [(x+14, divider), (x+290, divider)], LINE, 1)
    settings_icon(draw, x+22, y+270)
    label(draw, (x+48, y+270), "Settings", 13, TEXT, False, "lm")
    power_icon(draw, x+22, y+306)
    label(draw, (x+48, y+306), "Exit", 13, TEXT, False, "lm")


def native_menu(draw: ImageDraw.ImageDraw, x: int, y: int) -> None:
    box(draw, (x, y, x+248, y+322), 10, "#242428", "#45454A")
    label(draw, (x+16, y+22), "Helium", 11, "#D4D4D8")
    items = ["New window", "Restore window", "Pin to taskbar", "Close window"]
    for i, item in enumerate(items):
        yy = y + 55 + i*34
        application_icon(draw, x+22, yy, "helium")
        label(draw, (x+44, yy), item, 12, "#F4F4F5", False, "lm")
    line(draw, [(x+10, y+186), (x+238, y+186)], "#414146", 1)
    for i, item in enumerate(["Unpin from taskbar", "End task", "Close window"]):
        yy = y + 216 + i*32
        label(draw, (x+20, yy), "×" if i == 2 else "—", 14, "#D7D7DB", False, "mm")
        label(draw, (x+42, yy), item, 12, "#F4F4F5", False, "lm")


def selector(draw: ImageDraw.ImageDraw, x: int, y: int, volume: int = 72, choice: int = 4) -> None:
    box(draw, (x, y, x+250, y+322), 10, PANEL, LINE)
    application_icon(draw, x+24, y+25, "helium")
    label(draw, (x+47, y+25), "Helium", 14, TEXT, True, "lm")
    line(draw, [(x+12, y+48), (x+238, y+48)], LINE, 1)
    speaker_icon(draw, x+18, y+75)
    line(draw, [(x+44, y+75), (x+180, y+75)], "#48515E", 5)
    line(draw, [(x+44, y+75), (x+44+int(136*volume/100), y+75)], ACCENT, 5)
    cx = x+44+int(136*volume/100)
    draw.ellipse(((cx-5)*SCALE, (y+70)*SCALE, (cx+5)*SCALE, (y+80)*SCALE), fill="#BBD6FF")
    label(draw, (x+231, y+75), f"{volume}%", 11, MUTED, False, "rm")
    line(draw, [(x+12, y+98), (x+238, y+98)], LINE, 1)
    label(draw, (x+14, y+117), "Audio output", 11, MUTED)
    outputs = ["System default", "VoiceMeeter Input", "Digital Audio (S/PDIF)", "FIFINE K658 speakers", "Headphones"]
    for i, output in enumerate(outputs):
        yy = y + 146 + i*34
        if i == choice:
            draw.rectangle(((x+8)*SCALE, (yy-16)*SCALE, (x+242)*SCALE, (yy+16)*SCALE), fill="#30353D")
            check_icon(draw, x+17, yy)
        label(draw, (x+34, yy), output, 12, TEXT, False, "lm")


def taskbar(draw: ImageDraw.ImageDraw) -> None:
    draw.rectangle((342*SCALE, 428*SCALE, 884*SCALE, 478*SCALE), fill="#20242A")
    for index, kind in enumerate(["chrome", "helium", "zen"]):
        x = 500 + index * 48
        application_icon(draw, x, 452, kind)
        draw.rounded_rectangle(((x-12)*SCALE, 470*SCALE, (x+12)*SCALE, 473*SCALE), SCALE, fill="#B8CBE7" if kind == "spotify" else "#596575")


def cursor(draw: ImageDraw.ImageDraw, x: int, y: int) -> None:
    points = [(x, y), (x, y+22), (x+6, y+17), (x+11, y+27), (x+15, y+25), (x+10, y+15), (x+18, y+14)]
    draw.polygon([(px*SCALE, py*SCALE) for px, py in points], fill="#FFFFFF", outline="#111318")


def scene(kind: int, progress: float) -> Image.Image:
    image, draw = canvas((900, 520))
    brand_mark(draw, (54, 52), 1.1)
    label(draw, (54, 151), "Adufa", 40, TEXT, True)
    label(draw, (56, 203), "Every app. The right output.", 18, MUTED)
    steps = ["Open from the tray", "Right-click any running app", "Adjust volume", "Choose an output"]
    for i, step in enumerate(steps):
        yy = 292 + i*38
        active = i == kind
        if active:
            box(draw, (48, yy-12, 296, yy+22), 7, "#1D2735", "#334258")
        draw.ellipse(((60-4)*SCALE, (yy+5-4)*SCALE, (60+4)*SCALE, (yy+5+4)*SCALE), fill=ACCENT if active else "#4F5967")
        label(draw, (76, yy+5), step, 13, TEXT if active else MUTED, active, "lm")
    line(draw, [(328, 42), (328, 478)], "#252B34", 1)
    box(draw, (342, 42, 884, 478), 18, "#15191F", "#2A3039")
    if kind == 0:
        popup(draw, 461, 88, None)
        cursor(draw, int(748 - progress*52), int(445 - progress*145))
    elif kind == 1:
        native_menu(draw, 360, 88)
        selector(draw, 616, 88, 72, 4)
        taskbar(draw)
        cursor(draw, int(548 + progress*105), int(426 - progress*92))
    elif kind == 2:
        native_menu(draw, 360, 88)
        selector(draw, 616, 88, int(45 + progress*35), 4)
        taskbar(draw)
        cursor(draw, int(698 + progress*70), 160)
    else:
        native_menu(draw, 360, 88)
        choice = 4 if progress < 0.6 else 1
        selector(draw, 616, 88, 80, choice)
        taskbar(draw)
        cursor(draw, 706, int(350 - progress*132))
    label(draw, (611, 492), "LOCAL ONLY  •  NO TELEMETRY", 10, "#697585", True, "mm")
    return image.resize((900, 520), Image.Resampling.LANCZOS)


def closed_scene() -> Image.Image:
    image, draw = canvas((900, 520))
    brand_mark(draw, (54, 52), 1.1)
    label(draw, (54, 151), "Adufa", 40, TEXT, True)
    label(draw, (56, 203), "Every app. The right output.", 18, MUTED)
    steps = ["Open from the tray", "Right-click any running app", "Adjust volume", "Choose an output"]
    for i, step in enumerate(steps):
        yy = 292 + i*38
        draw.ellipse(((60-4)*SCALE, (yy+5-4)*SCALE, (60+4)*SCALE, (yy+5+4)*SCALE), fill=ACCENT if i == 3 else "#4F5967")
        label(draw, (76, yy+5), step, 13, TEXT if i == 3 else MUTED, i == 3, "lm")
    line(draw, [(328, 42), (328, 478)], "#252B34", 1)
    box(draw, (342, 42, 884, 478), 18, "#15191F", "#2A3039")
    draw.ellipse((593*SCALE, 216*SCALE, 633*SCALE, 256*SCALE), fill=ACCENT_BG, outline="#334258", width=SCALE)
    check_icon(draw, 613, 236, ACCENT)
    label(draw, (613, 279), "Route changed", 15, TEXT, True, "mm")
    label(draw, (613, 303), "Both menus close automatically", 12, MUTED, False, "mm")
    label(draw, (611, 492), "LOCAL ONLY  •  NO TELEMETRY", 10, "#697585", True, "mm")
    return image.resize((900, 520), Image.Resampling.LANCZOS)


def blend(a: Image.Image, b: Image.Image, amount: float) -> Image.Image:
    return Image.blend(a, b, amount)


def render_demo() -> None:
    frames: list[Image.Image] = []
    duration: list[int] = []
    for kind in range(4):
        for step in range(14):
            frames.append(scene(kind, step/13))
            duration.append(110)
    frames.extend([scene(3, 1)] * 5)
    duration.extend([120] * 5)
    start = frames[-1]
    end = closed_scene()
    for step in range(1, 7):
        frames.append(blend(start, end, step/7))
        duration.append(90)
    frames.extend([end] * 10)
    duration.extend([120] * 10)
    frames[0].save(
        ASSETS / "adufa-demo.gif",
        save_all=True,
        append_images=frames[1:],
        duration=duration,
        loop=0,
        disposal=2,
        optimize=True,
    )


def render_icons() -> None:
    sizes = [16, 24, 32, 48, 64, 128, 256, 512]
    rendered: list[Image.Image] = []
    for size in sizes:
        image, draw = canvas((64, 64))
        brand_mark(draw, (0, 0), 1.0)
        image = image.resize((size, size), Image.Resampling.LANCZOS)
        image.save(ASSETS / f"adufa-icon-{size}.png")
        rendered.append(image.convert("RGBA"))
    icon_sizes = [(size, size) for size in sizes if size <= 256]
    rendered[-1].save(ASSETS / "adufa.ico", sizes=icon_sizes)
    windows_assets = ROOT / "platforms" / "windows" / "assets"
    windows_assets.mkdir(parents=True, exist_ok=True)
    rendered[-1].save(windows_assets / "adufa.ico", sizes=icon_sizes)


if __name__ == "__main__":
    ASSETS.mkdir(parents=True, exist_ok=True)
    render_icons()
    render_demo()
    print(f"Generated brand assets in {ASSETS}")
