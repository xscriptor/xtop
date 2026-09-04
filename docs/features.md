<h1>Features</h1>

<p>Detailed breakdown of xtop's monitoring and interface capabilities.</p>

<hr>

<h2 id="system-monitoring">System Monitoring</h2>

<h3 id="cpu">CPU</h3>

<ul>
  <li>Usage percentage per core and per thread, displayed as horizontal gauges.</li>
  <li>Maximum CPU temperature reading when hardware sensors are available.</li>
  <li>Color-coded bars that visually indicate load levels.</li>
</ul>

<h3 id="memory">Memory</h3>

<ul>
  <li>RAM usage gauge showing used, total, and percentage.</li>
  <li>Swap usage gauge.</li>
  <li>Historical line chart tracking RAM usage over time.</li>
  <li>Configurable number of history data points.</li>
</ul>

<h3 id="network">Network</h3>

<ul>
  <li>Real-time upload (TX) and download (RX) tracking per network interface.</li>
  <li>Total data transferred displayed alongside current transfer speeds.</li>
</ul>

<h3 id="storage">Storage</h3>

<ul>
  <li>Disk usage gauges per mount point showing used, available, and total space.</li>
  <li>Visual percentage bars for each mounted filesystem.</li>
</ul>

<h3 id="disk-io">Disk I/O</h3>

<ul>
  <li>Read and write speed tracking per disk device.</li>
  <li>Displayed in bytes per second with automatic unit scaling.</li>
</ul>

<h3 id="processes">Processes</h3>

<ul>
  <li>Scrolling list of running processes sorted by CPU usage.</li>
  <li>Live search filtering by process name.</li>
  <li>Displays process name, CPU usage, and memory usage.</li>
</ul>

<h3 id="gpu">GPU</h3>

<ul>
  <li>GPU usage gauges: real data on Linux (NVIDIA via <code>nvidia-smi</code>, AMD/Intel via <code>/sys/class/drm</code>); stub on macOS/Windows.</li>
</ul>

<h3 id="battery">Battery</h3>

<ul>
  <li>Battery charge level gauges: real data on Linux (<code>/sys/class/power_supply</code>); stub on macOS/Windows.</li>
</ul>

<hr>

<h2 id="theming">Theming</h2>

<ul>
  <li>12 color schemes: 12 JSONC theme files ship in <code>assets/themes/</code> and are embedded in the binary as first-run seeding templates.</li>
  <li>Custom themes defined as JSONC files with a 16-entry hex color palette.</li>
  <li>Instant theme cycling with <kbd>t</kbd> (next) and <kbd>T</kbd> (previous).</li>
  <li>Palette reference in <a href="colors.md">colors.md</a>.</li>
</ul>

<hr>

<h2 id="layouts">Layouts</h2>

<ul>
  <li>7 built-in layout modes: Dashboard, Vertical, Horizontal, CPU Focus, Memory Focus, Network Focus, Process Focus.</li>
  <li>Custom layouts defined as JSONC files with a recursive split/widget tree.</li>
  <li>Full-screen mode for any widget toggled with <kbd>f</kbd>.</li>
  <li>Responsive design that adapts to terminal width and height automatically.</li>
  <li>Minimal fallback layout for very small terminals.</li>
</ul>

<hr>

<h2 id="alert-thresholds">Alert Thresholds</h2>

<ul>
  <li>Visual warnings when CPU, memory, or disk usage exceeds configurable limits.</li>
  <li>Color changes to red and warning indicators in widget titles.</li>
</ul>

<hr>

<h2 id="persistence">Persistence</h2>

<ul>
  <li>Current theme, layout, update interval, history points, alert thresholds and glyph style are saved automatically on quit.</li>
  <li>Configuration is stored as <code>config.json</code> in the platform config dir: <code>~/.config/xtop/</code> on Linux, <code>~/Library/Application Support/xtop/</code> on macOS, <code>%APPDATA%\xtop</code> on Windows.</li>
</ul>

<hr>

<p align="center">
  <a href="../README.md">Back to README</a>
</p>
