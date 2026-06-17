# Customization

xtop supports runtime customization of color themes and layout modes via external JSONC files. This guide explains how to create and manage your own themes and layouts.

---

## Themes

### Location

Place theme files in:

```
~/.config/xtop/themes/*.jsonc
~/.config/xtop/themes/*.json
```

The directory is created automatically when you save your config on quit.

### Format

Each theme file defines a name and a 16-entry color palette. Colors are hex strings with optional `#` prefix. Comments (`//` and `/* */`) are supported.

```jsonc
{
    // my-custom-theme -- Dark background, warm accents
    "name": "my-custom-theme",
    "palette": [
        "#1a1b1c", //  0: background
        "#e06c75", //  1: red / alert
        "#98c379", //  2: green (RAM gauge)
        "#e5c07b", //  3: yellow (Swap gauge)
        "#d19a66", //  4: orange (Storage, Network TX)
        "#c678dd", //  5: purple (GPU)
        "#56b6c2", //  6: cyan (table headers)
        "#abb2bf", //  7: foreground / text
        "#3e4451", //  8: bright black (separators)
        "#e06c75", //  9: bright red
        "#98c379", // 10: bright green
        "#e5c07b", // 11: bright yellow
        "#d19a66", // 12: bright orange
        "#c678dd", // 13: bright purple
        "#56b6c2", // 14: bright cyan
        "#abb2bf"  // 15: bright white
    ]
}
```

### Starter themes

The repository includes 12 ready-to-use themes you can copy as a starting point:

```bash
# Copy all starter themes
cp -r assets/themes/* ~/.config/xtop/themes/
```

Available starter themes: `x`, `madrid`, `lahabana`, `seul`, `paris`, `tokio`, `oslo`, `helsinki`, `berlin`, `london`, `praha`, `bogota`.

The built-in default theme is `miami` (black background, neon accents). It is compiled into the binary and always available.

### Loading order

1. Built-in `miami` theme (always available)
2. Themes from `~/.config/xtop/themes/` loaded alphabetically
3. If a custom theme has the same name as `miami`, it **replaces** the built-in

### Tips

- Try the grayscale themes (`london`, `berlin`) as a base and add your own accent colors
- Palette entries 8–15 (the "bright" variants) are used for separators and secondary text
- Index 0 is the background, index 7 is the primary foreground — keep them readable together

---

## Layouts

### Location

Place layout files in:

```
~/.config/xtop/layouts/*.jsonc
~/.config/xtop/layouts/*.json
```

### Format

A layout is a recursive tree of **splits** and **widgets**:

```
LayoutDef
 ├── name: string
 └── root: Area
      ├── direction: "horizontal" | "vertical"
      ├── size: constraint (optional, defaults to "*")
      └── areas: [Area, ...]
           ├── Area with "widget" → leaf node (renders a widget)
           └── Area with "direction" → nested split
```

**Size constraints:**

| Syntax | Meaning |
|--------|---------|
| `"*"` or omitted | Fill remaining space |
| `3` (number) | Fixed `n` rows/columns |
| `"45%"` | Percentage of parent |

**Available widget names:**

| Widget | Description |
|--------|-------------|
| `header` | System info bar (uptime, load, keys) |
| `cpu` | Per-core CPU usage gauges |
| `memory` | RAM + Swap gauges + RAM history chart |
| `storage` | Disk usage gauges per mount point |
| `network` | Network RX/TX totals and speeds |
| `processes` | Process table with search filter |
| `disk_io` | Disk read/write speeds |
| `battery` | Battery charge gauges |
| `gpu` | GPU usage gauges |

### Example: custom layout

```jsonc
{
    // "monitor" — CPU top-half, processes bottom-half
    "name": "monitor",
    "root": {
        "direction": "vertical",
        "areas": [
            { "widget": "header", "size": 3 },
            { "widget": "cpu", "size": "55%" },
            { "widget": "processes", "size": "*" }
        ]
    }
}
```

### Example: complex nested layout

```jsonc
{
    "name": "my-dashboard",
    "root": {
        "direction": "vertical",
        "areas": [
            { "widget": "header", "size": 3 },
            {
                "direction": "horizontal",
                "size": "50%",
                "areas": [
                    { "widget": "cpu", "size": "60%" },
                    {
                        "direction": "vertical",
                        "size": "40%",
                        "areas": [
                            { "widget": "network", "size": "50%" },
                            { "widget": "disk_io", "size": "50%" }
                        ]
                    }
                ]
            },
            { "widget": "processes", "size": "*" }
        ]
    }
}
```

### Starter layouts

The repository includes the 7 built-in layouts as JSONC files:

```bash
cp -r assets/layouts/* ~/.config/xtop/layouts/
```

Available: `dashboard`, `vertical`, `horizontal`, `cpu_focus`, `memory_focus`, `network_focus`, `process_focus`.

You can edit these directly or use them as templates.

### Cycling order

1. Built-in layouts (Dashboard → Vertical → Horizontal → CPU Focus → Memory Focus → Network Focus → Process Focus)
2. Custom layouts from `~/.config/xtop/layouts/` (in filesystem order)
3. Wraps back to Dashboard

Press `l` to cycle forward through all available layouts.

### Notes

- If a widget name in your layout doesn't match any available widget, that area is silently skipped
- Nested splits can be arbitrarily deep, but very deep nesting may overflow small terminals
- The terminal must be at least 40×8 for any layout to render; smaller terminals show a warning
- Very small terminals (under 60×14) fall back to a minimal hardcoded layout (CPU + Memory gauges + process list)
