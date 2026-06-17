<h1 id="customization-guide">Customization Guide</h1>

<p>xtop supports runtime customization of color themes and layout modes via external JSONC files. This guide explains how to create and manage your own themes and layouts.</p>

---

<h2 id="table-of-contents">Table of Contents</h2>

<ul>
  <li><a href="#themes">Themes</a>
    <ul>
      <li><a href="#theme-location">Location</a></li>
      <li><a href="#theme-format">Format</a></li>
      <li><a href="#palette-reference">Palette Reference</a></li>
      <li><a href="#starter-themes">Starter Themes</a></li>
      <li><a href="#loading-order">Loading Order</a></li>
      <li><a href="#theme-tips">Tips</a></li>
    </ul>
  </li>
  <li><a href="#layouts">Layouts</a>
    <ul>
      <li><a href="#layout-location">Location</a></li>
      <li><a href="#layout-format">Format</a></li>
      <li><a href="#size-constraints">Size Constraints</a></li>
      <li><a href="#available-widgets">Available Widgets</a></li>
      <li><a href="#layout-examples">Examples</a></li>
      <li><a href="#starter-layouts">Starter Layouts</a></li>
      <li><a href="#cycling-order">Cycling Order</a></li>
      <li><a href="#layout-notes">Notes</a></li>
    </ul>
  </li>
</ul>

---

<h2 id="themes">Themes</h2>

<h3 id="theme-location">Location</h3>

<p>Place theme files in:</p>

<pre><code>~/.config/xtop/themes/*.jsonc
~/.config/xtop/themes/*.json
</code></pre>

<p>The directory is created automatically when you save your config on quit.</p>

<h3 id="theme-format">Format</h3>

<p>Each theme file defines a <code>name</code> and a 16-entry <code>palette</code>. Colors are hex strings with an optional <code>#</code> prefix. Comments (<code>//</code> and <code>/* */</code>) are supported in JSONC files.</p>

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
        "#56b6c2", //  6: cyan (accents, table headers)
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

<h3 id="palette-reference">Palette Reference</h3>

<table>
  <thead>
    <tr>
      <th>Index</th>
      <th>Usage</th>
      <th>Example</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><code>0</code></td>
      <td>Background</td>
      <td><code>#1a1b1c</code></td>
    </tr>
    <tr>
      <td><code>1</code></td>
      <td>Red / Alert</td>
      <td><code>#e06c75</code></td>
    </tr>
    <tr>
      <td><code>2</code></td>
      <td>Green (RAM gauge)</td>
      <td><code>#98c379</code></td>
    </tr>
    <tr>
      <td><code>3</code></td>
      <td>Yellow (Swap gauge)</td>
      <td><code>#e5c07b</code></td>
    </tr>
    <tr>
      <td><code>4</code></td>
      <td>Orange (Storage, Network TX)</td>
      <td><code>#d19a66</code></td>
    </tr>
    <tr>
      <td><code>5</code></td>
      <td>Purple (GPU)</td>
      <td><code>#c678dd</code></td>
    </tr>
    <tr>
      <td><code>6</code></td>
      <td>Cyan (Accents, table headers)</td>
      <td><code>#56b6c2</code></td>
    </tr>
    <tr>
      <td><code>7</code></td>
      <td>Foreground / Text</td>
      <td><code>#abb2bf</code></td>
    </tr>
    <tr>
      <td><code>8</code></td>
      <td>Bright black (Separators)</td>
      <td><code>#3e4451</code></td>
    </tr>
    <tr>
      <td><code>9</code></td>
      <td>Bright red</td>
      <td><code>#e06c75</code></td>
    </tr>
    <tr>
      <td><code>10</code></td>
      <td>Bright green</td>
      <td><code>#98c379</code></td>
    </tr>
    <tr>
      <td><code>11</code></td>
      <td>Bright yellow</td>
      <td><code>#e5c07b</code></td>
    </tr>
    <tr>
      <td><code>12</code></td>
      <td>Bright orange</td>
      <td><code>#d19a66</code></td>
    </tr>
    <tr>
      <td><code>13</code></td>
      <td>Bright purple</td>
      <td><code>#c678dd</code></td>
    </tr>
    <tr>
      <td><code>14</code></td>
      <td>Bright cyan</td>
      <td><code>#56b6c2</code></td>
    </tr>
    <tr>
      <td><code>15</code></td>
      <td>Bright white</td>
      <td><code>#abb2bf</code></td>
    </tr>
  </tbody>
</table>

<h3 id="starter-themes">Starter Themes</h3>

<p>The built-in default theme is <code>x</code> (almost-black background, purple-pink accents). It is compiled into the binary and always available.</p>

<p>When you run xtop for the first time, it automatically creates <code>~/.config/xtop/themes/</code> with all extra themes embedded in the binary. No manual copy is needed.</p>

<p>If you want to restore them later, copy from the repository:</p>

```bash
cp -r assets/themes/* ~/.config/xtop/themes/
```

<p><strong>Available themes:</strong> <code>x</code>, <code>madrid</code>, <code>lahabana</code>, <code>paris</code>, <code>tokio</code>, <code>oslo</code>, <code>helsinki</code>, <code>berlin</code>, <code>london</code>, <code>praha</code>, <code>bogota</code>, <code>miami</code>.</p>

<p>All theme definitions are documented in <a href="../colors.md"><code>colors.md</code></a>.</p>

<h3 id="loading-order">Loading Order</h3>

<ol>
  <li>Built-in <code>miami</code> theme (always available)</li>
  <li>Themes from <code>~/.config/xtop/themes/</code> loaded alphabetically</li>
  <li>If a custom theme has the same name as <code>miami</code>, it <strong>replaces</strong> the built-in</li>
</ol>

<h3 id="theme-tips">Tips</h3>

<ul>
  <li>Try the grayscale themes (<code>london</code>, <code>berlin</code>) as a base and add your own accent colors.</li>
  <li>Palette entries 8–15 (the "bright" variants) are used for separators and secondary text.</li>
  <li>Index 0 is the background, index 7 is the primary foreground — keep them readable together.</li>
</ul>

---

<h2 id="layouts">Layouts</h2>

<h3 id="layout-location">Location</h3>

<p>Place layout files in:</p>

<pre><code>~/.config/xtop/layouts/*.jsonc
~/.config/xtop/layouts/*.json
</code></pre>

<h3 id="layout-format">Format</h3>

<p>A layout is a recursive tree of <strong>splits</strong> and <strong>widgets</strong>:</p>

<pre><code>LayoutDef
 ├── name: string
 └── root: Area
      ├── direction: "horizontal" | "vertical"
      ├── size: constraint (optional, defaults to "*")
      └── areas: [Area, ...]
           ├── Area with "widget" → leaf node (renders a widget)
           └── Area with "direction" → nested split
</code></pre>

<h3 id="size-constraints">Size Constraints</h3>

<table>
  <thead>
    <tr>
      <th>Syntax</th>
      <th>Meaning</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><code>"*"</code> or omitted</td>
      <td>Fill remaining space</td>
    </tr>
    <tr>
      <td><code>3</code> (number)</td>
      <td>Fixed <em>n</em> rows/columns</td>
    </tr>
    <tr>
      <td><code>"45%"</code></td>
      <td>Percentage of parent</td>
    </tr>
  </tbody>
</table>

<h3 id="available-widgets">Available Widgets</h3>

<table>
  <thead>
    <tr>
      <th>Widget</th>
      <th>Description</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><code>header</code></td>
      <td>System info bar (uptime, load, keys)</td>
    </tr>
    <tr>
      <td><code>cpu</code></td>
      <td>Per-core CPU usage gauges</td>
    </tr>
    <tr>
      <td><code>memory</code></td>
      <td>RAM + Swap gauges + RAM history chart</td>
    </tr>
    <tr>
      <td><code>storage</code></td>
      <td>Disk usage gauges per mount point</td>
    </tr>
    <tr>
      <td><code>network</code></td>
      <td>Network RX/TX totals and speeds</td>
    </tr>
    <tr>
      <td><code>processes</code></td>
      <td>Process table with search filter</td>
    </tr>
    <tr>
      <td><code>disk_io</code></td>
      <td>Disk read/write speeds</td>
    </tr>
    <tr>
      <td><code>battery</code></td>
      <td>Battery charge gauges</td>
    </tr>
    <tr>
      <td><code>gpu</code></td>
      <td>GPU usage gauges</td>
    </tr>
  </tbody>
</table>

<h3 id="layout-examples">Examples</h3>

<h4>Simple custom layout</h4>

<p>A minimal three-row layout: header, CPU, and processes.</p>

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

<h4>Complex nested layout</h4>

<p>A full dashboard with a horizontal split in the middle section:</p>

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

<h3 id="starter-layouts">Starter Layouts</h3>

<p>The 7 built-in layouts are embedded in the binary and written to <code>~/.config/xtop/layouts/</code> on first run.</p>

<p>To restore them later, copy from the repository:</p>

```bash
cp -r assets/layouts/* ~/.config/xtop/layouts/
```

<p><strong>Available layouts:</strong> <code>dashboard</code>, <code>vertical</code>, <code>horizontal</code>, <code>cpu_focus</code>, <code>memory_focus</code>, <code>network_focus</code>, <code>process_focus</code>.</p>

<h3 id="cycling-order">Cycling Order</h3>

<ol>
  <li>Built-in layouts (Dashboard → Vertical → Horizontal → CPU Focus → Memory Focus → Network Focus → Process Focus)</li>
  <li>Custom layouts from <code>~/.config/xtop/layouts/</code> (in filesystem order)</li>
  <li>Wraps back to Dashboard</li>
</ol>

<p>Press <kbd>l</kbd> to cycle forward through all available layouts.</p>

<h3 id="layout-notes">Notes</h3>

<ul>
  <li>If a widget name in your layout doesn't match any available widget, that area is silently skipped.</li>
  <li>Nested splits can be arbitrarily deep, but very deep nesting may overflow small terminals.</li>
  <li>The terminal must be at least 40×8 for any layout to render; smaller terminals show a warning.</li>
  <li>Very small terminals (under 60×14) fall back to a minimal hardcoded layout (CPU + Memory gauges + process list).</li>
</ul>

---

<p align="center">
  <a href="../README.md">← Back to README</a>
</p>
