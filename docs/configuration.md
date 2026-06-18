<h1>Configuration</h1>

<hr>

<h2 id="config-file">Config File</h2>

<p>xtop automatically saves its configuration on quit. The configuration file is located at:</p>

<pre><code>~/.config/xtop/config.json</code></pre>

<hr>

<h2 id="persisted-settings">Persisted Settings</h2>

<table>
  <thead>
    <tr>
      <th>Setting</th>
      <th>Description</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><code>theme</code></td>
      <td>Currently selected color theme name</td>
    </tr>
    <tr>
      <td><code>layout</code></td>
      <td>Currently selected layout mode</td>
    </tr>
    <tr>
      <td><code>interval</code></td>
      <td>Update interval in milliseconds</td>
    </tr>
    <tr>
      <td><code>history_points</code></td>
      <td>Number of data points retained for the RAM history chart</td>
    </tr>
    <tr>
      <td><code>alert_thresholds</code></td>
      <td>Threshold values for CPU, memory, and disk alerts</td>
    </tr>
  </tbody>
</table>

<hr>

<h2 id="alert-thresholds">Alert Thresholds</h2>

<p>When a metric exceeds its configured threshold, the corresponding widget changes color to red and displays a warning indicator in its title.</p>

<table>
  <thead>
    <tr>
      <th>Threshold</th>
      <th>Description</th>
      <th>Default</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><code>cpu_high</code></td>
      <td>CPU usage percentage that triggers a warning</td>
      <td>90%</td>
    </tr>
    <tr>
      <td><code>mem_high</code></td>
      <td>Memory usage percentage that triggers a warning</td>
      <td>90%</td>
    </tr>
    <tr>
      <td><code>disk_high</code></td>
      <td>Disk usage percentage that triggers a warning</td>
      <td>90%</td>
    </tr>
  </tbody>
</table>

<hr>

<h2 id="custom-themes-and-layouts">Custom Themes and Layouts</h2>

<p>For custom themes and layouts, see the <a href="customization.md">customization guide</a>.</p>

<hr>

<p align="center">
  <a href="../README.md">Back to README</a>
</p>
