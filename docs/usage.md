<h1>Usage</h1>

<hr>

<h2 id="table-of-contents">Table of Contents</h2>

<ul>
  <li><a href="#keybindings">Keybindings</a></li>
  <li><a href="#modules">Modules</a></li>
  <li><a href="#help-overlay">Help Overlay</a></li>
  <li><a href="#command-palette">Command Palette</a></li>
  <li><a href="#full-screen-mode">Full-Screen Mode</a></li>
  <li><a href="#responsive-layouts">Responsive Layouts</a></li>
</ul>

<hr>

<h2 id="keybindings">Keybindings</h2>

<table>
  <thead>
    <tr>
      <th>Key</th>
      <th>Action</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><kbd>q</kbd></td>
      <td>Quit application (saves configuration)</td>
    </tr>
    <tr>
      <td><kbd>?</kbd></td>
      <td>Toggle help overlay</td>
    </tr>
    <tr>
      <td><kbd>t</kbd></td>
      <td>Next color theme</td>
    </tr>
    <tr>
      <td><kbd>T</kbd></td>
      <td>Previous color theme</td>
    </tr>
    <tr>
      <td><kbd>l</kbd></td>
      <td>Next layout mode (cycles through built-in and custom layouts)</td>
    </tr>
    <tr>
      <td><kbd>f</kbd></td>
      <td>Toggle full-screen mode for the current widget</td>
    </tr>
    <tr>
      <td><kbd>F</kbd></td>
      <td>Cycle full-screen focus through all available widgets</td>
    </tr>
    <tr>
      <td><kbd>/</kbd></td>
      <td>Open process search / filter</td>
    </tr>
    <tr>
      <td><kbd>Enter</kbd></td>
      <td>Confirm search filter</td>
    </tr>
    <tr>
      <td><kbd>Backspace</kbd></td>
      <td>Delete character in search input</td>
    </tr>
    <tr>
      <td><kbd>Esc</kbd></td>
      <td>Cancel search / close help overlay</td>
    </tr>
  </tbody>
</table>

<hr>

<h2 id="modules">Modules</h2>

<ol>
  <li>
    <p><strong>Header</strong> -- Shows system uptime, load average, current theme name, and active layout mode.</p>
  </li>
  <li>
    <p><strong>CPU</strong> -- Horizontal usage bars for each CPU core. If hardware sensors are available, displays the maximum CPU temperature.</p>
  </li>
  <li>
    <p><strong>Memory</strong> -- Gauges for RAM and Swap usage, plus a line chart showing RAM usage over a configurable history window.</p>
  </li>
  <li>
    <p><strong>Storage</strong> -- Disk usage gauges per mount point showing capacity and used space.</p>
  </li>
  <li>
    <p><strong>Network</strong> -- Total downloaded (RX) and uploaded (TX) data per interface, with current transfer rates.</p>
  </li>
  <li>
    <p><strong>Disk I/O</strong> -- Read and write speeds per disk device in bytes per second.</p>
  </li>
  <li>
    <p><strong>Processes</strong> -- Scrolling list of processes sorted by CPU usage (top 200 by default; override with <code>XTOP_MAX_PROCESSES</code>), with live search filtering by process name.</p>
  </li>
  <li>
    <p><strong>GPU</strong> -- GPU usage gauges (available on supported hardware).</p>
  </li>
  <li>
    <p><strong>Battery</strong> -- Battery charge level gauges (available on supported hardware).</p>
  </li>
</ol>

<hr>

<h2 id="help-overlay">Help Overlay</h2>

<p>Press <kbd>?</kbd> at any time to display a full list of available keybindings on screen. Press <kbd>?</kbd> again or <kbd>Esc</kbd> to close the overlay.</p>

<hr>

<h2 id="process-search">Process Search</h2>

<p>Search filters the process list in real time:</p>

<ul>
  <li>Press <kbd>/</kbd> to open the search bar at the top of the process list.</li>
  <li>Type any query -- results filter instantly by process name.</li>
  <li>Press <kbd>Enter</kbd> to confirm the filter, <kbd>Esc</kbd> to cancel, <kbd>Backspace</kbd> to delete characters.</li>
  <li>A centered overlay with a <code>/query_</code> indicator shows the current search input.</li>
</ul>

<hr>

<h2 id="command-palette">Command Palette</h2>

<p>Press <kbd>ctrl+p</kbd> to open the command palette, a searchable list of
actions (also reachable with the <code>command_palette</code> keybinding).
The palette has three pages: <strong>Main</strong> (go to themes/layouts,
toggle or cycle full-screen, search, help, cycle the process sort, random
theme, exit), <strong>Themes</strong> (jump to any loaded theme) and
<strong>Layouts</strong> (jump to any layout). Type to filter, use
<kbd>up</kbd>/<kbd>down</kbd> to move, <kbd>Enter</kbd> to run the selected
action and <kbd>Esc</kbd> to close.</p>

<hr>

<h2 id="full-screen-mode">Full-Screen Mode</h2>

<ul>
  <li>Press <kbd>f</kbd> to toggle full-screen mode for the currently focused widget. The widget expands to fill the entire terminal area (minus the header bar).</li>
  <li>Press <kbd>F</kbd> to cycle full-screen focus through widgets in sequence: CPU, Memory, Storage, Network, Processes, Disk I/O, GPU, Battery, then exit.</li>
</ul>

<hr>

<h2 id="responsive-layouts">Responsive Layouts</h2>

<p>The interface adapts automatically to the terminal size:</p>

<table>
  <thead>
    <tr>
      <th>Terminal Size</th>
      <th>Behavior</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>Dashboard mode, 100+ cols and 28+ rows</td>
      <td>Full dashboard layout with 2 columns</td>
    </tr>
    <tr>
      <td>Dashboard mode, 80&ndash;99 cols or under 28 rows</td>
      <td>Compact layout</td>
    </tr>
    <tr>
      <td>Dashboard mode, narrower than 80 cols</td>
      <td>Vertically stacked layout</td>
    </tr>
    <tr>
      <td>Other modes</td>
      <td>The requested mode layout (Vertical, Horizontal, CPU/Memory/Network/Process Focus)</td>
    </tr>
    <tr>
      <td>Narrower than 60 cols or shorter than 14 rows</td>
      <td>Minimal layout: CPU, Memory, and Processes only</td>
    </tr>
    <tr>
      <td>Smaller than 40 x 8</td>
      <td>Warning message displayed (terminal too small)</td>
    </tr>
  </tbody>
</table>

<hr>

<p align="center">
  <a href="../README.md">Back to README</a>
</p>
