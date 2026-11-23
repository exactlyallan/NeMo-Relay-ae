# SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
# SPDX-License-Identifier: Apache-2.0
"""Rasterize exported Ratatui cells at one fixed font/cell size. No layout recreation."""

import json
import math
import sys
from pathlib import Path

# Pillow is an optional screenshot dependency supplied by `uv run --with pillow`.
from PIL import Image, ImageDraw, ImageFont  # ty: ignore[unresolved-import]

root = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).parent / "frames-v3"
regular = ImageFont.truetype("/System/Library/Fonts/Menlo.ttc", 15, index=0)
# Menlo's collection contains matching regular and bold at the same point size.
bold = regular
for index in range(1, 4):
    try:
        candidate = ImageFont.truetype("/System/Library/Fonts/Menlo.ttc", 15, index=index)
        if candidate.getname()[1].lower() == "bold":
            bold = candidate
            break
    except OSError:
        break
cell_w, cell_h, pad = regular.getlength("M"), 21, 16
for source in sorted(root.glob("*.json")):
    data = json.loads(source.read_text())
    width, height = data["width"], data["height"]
    image = Image.new("RGB", (math.ceil(width * cell_w) + 2 * pad, height * cell_h + 2 * pad), (10, 17, 23))
    draw = ImageDraw.Draw(image)
    # Backgrounds and glyphs are separate passes so adjacent cells cannot erase a
    # proportional antialiasing edge. Text positions come directly from the cell grid.
    for i, (symbol, fg, bg, heavy, dim, reverse) in enumerate(data["cells"]):
        if reverse:
            fg, bg = bg, fg
        x, y = i % width, i // width
        draw.rectangle(
            (
                pad + math.floor(x * cell_w),
                pad + y * cell_h,
                pad + math.ceil((x + 1) * cell_w) - 1,
                pad + (y + 1) * cell_h - 1,
            ),
            fill=tuple(bg),
        )
    for i, (symbol, fg, bg, heavy, dim, reverse) in enumerate(data["cells"]):
        if reverse:
            fg, bg = bg, fg
        if dim:
            fg = [(f + b) // 2 for f, b in zip(fg, bg)]
        x, y = i % width, i // width
        x0, x1 = pad + x * cell_w, pad + (x + 1) * cell_w
        y0, y1 = pad + y * cell_h, pad + (y + 1) * cell_h
        cx, cy = (x0 + x1) / 2, (y0 + y1) / 2
        color = tuple(fg)
        # Terminals commonly draw box/block glyphs to cell edges. Do the same
        # here; Menlo's font ascent/descent would otherwise leave artificial gaps.
        if symbol == "█":
            draw.rectangle((math.floor(x0), y0, math.ceil(x1) - 1, y1 - 1), fill=color)
        elif symbol in "▏▎▍▌▋▊▉":
            fraction = ("▏▎▍▌▋▊▉".index(symbol) + 1) / 8
            draw.rectangle((math.floor(x0), y0, x0 + cell_w * fraction, y1 - 1), fill=color)
        elif symbol in "▁▂▃▄▅▆▇":
            fraction = ("▁▂▃▄▅▆▇".index(symbol) + 1) / 8
            draw.rectangle((math.floor(x0), y1 - cell_h * fraction, math.ceil(x1) - 1, y1 - 1), fill=color)
        elif symbol in "╭╮╰╯":
            r = 3
            if symbol == "╭":
                draw.line((cx, cy + r, cx, y1), fill=color)
                draw.line((cx + r, cy, x1, cy), fill=color)
                draw.arc((cx, cy, cx + 2 * r, cy + 2 * r), 180, 270, fill=color)
            elif symbol == "╮":
                draw.line((x0, cy, cx - r, cy), fill=color)
                draw.line((cx, cy + r, cx, y1), fill=color)
                draw.arc((cx - 2 * r, cy, cx, cy + 2 * r), 270, 360, fill=color)
            elif symbol == "╰":
                draw.line((cx, y0, cx, cy - r), fill=color)
                draw.line((cx + r, cy, x1, cy), fill=color)
                draw.arc((cx, cy - 2 * r, cx + 2 * r, cy), 90, 180, fill=color)
            else:
                draw.line((cx, y0, cx, cy - r), fill=color)
                draw.line((x0, cy, cx - r, cy), fill=color)
                draw.arc((cx - 2 * r, cy - 2 * r, cx, cy), 0, 90, fill=color)
        elif symbol in "─│┌┐└┘├┤┬┴┼":
            directions = {
                "─": "lr",
                "│": "ud",
                "┌": "rd",
                "┐": "ld",
                "└": "ru",
                "┘": "lu",
                "├": "rud",
                "┤": "lud",
                "┬": "lrd",
                "┴": "lru",
                "┼": "lrud",
                "╭": "rd",
                "╮": "ld",
                "╰": "ru",
                "╯": "lu",
            }[symbol]
            for direction in directions:
                endpoint = {"l": (x0, cy), "r": (x1, cy), "u": (cx, y0), "d": (cx, y1)}[direction]
                draw.line((cx, cy, *endpoint), fill=color, width=1)
        else:
            draw.text((x0, y0), symbol, font=bold if heavy else regular, fill=color)
    image.save(source.with_suffix(".png"))
print(f"Rendered {len(list(root.glob('*.json')))} native cell buffers; font size = 15 throughout")
