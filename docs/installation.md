<h1>Installation Guide</h1>

<hr>

<h2 id="table-of-contents">Table of Contents</h2>

<ul>
  <li><a href="#quick-install-macos-linux">Quick Install (macOS/Linux)</a></li>
  <li><a href="#quick-install-windows">Quick Install (Windows)</a></li>
  <li><a href="#installer-options">Installer Options</a></li>
  <li><a href="#build-from-source">Build from Source</a></li>
  <li><a href="#uninstall">Uninstall</a></li>
  <li><a href="#supported-distributions">Supported Distributions</a></li>
</ul>

<hr>

<h2 id="quick-install-macos-linux">Quick Install (macOS/Linux)</h2>

<p>The installer script automatically detects your distribution and installs all required dependencies, including Rust if needed.</p>

<h3>Using curl</h3>

<pre><code>curl -fsSL https://raw.githubusercontent.com/xtop-cli/xtop/main/install.sh | bash</code></pre>

<h3>Using wget</h3>

<pre><code>wget -qO- https://raw.githubusercontent.com/xtop-cli/xtop/main/install.sh | bash</code></pre>

<hr>

<h2 id="quick-install-windows">Quick Install (Windows)</h2>

<p>Requires <a href="https://rustup.rs/">Rust (Cargo)</a> to be installed. Run in PowerShell:</p>

<pre><code>irm https://raw.githubusercontent.com/xtop-cli/xtop/main/install.ps1 | iex</code></pre>

<hr>

<h2 id="installer-options">Installer Options</h2>

<p>You can run the installer script with additional flags for more control:</p>

<pre><code># Check dependencies without installing
./install.sh --check-deps

# Install only dependencies (Rust, build tools)
./install.sh --install-deps

# Show help
./install.sh --help</code></pre>

<h3>Supported Distributions</h3>

<ul>
  <li>Arch Linux and derivatives</li>
  <li>Debian / Ubuntu and derivatives</li>
  <li>Fedora / RHEL and derivatives</li>
  <li>openSUSE and derivatives</li>
  <li>Alpine Linux and derivatives</li>
</ul>

<hr>

<h2 id="build-from-source">Build from Source</h2>

<ol>
  <li>
    <p>Clone the repository:</p>
    <pre><code>git clone https://github.com/xtop-cli/xtop.git
cd xtop</code></pre>
  </li>
  <li>
    <p>Build and run with release optimizations:</p>
    <pre><code>cargo run --release</code></pre>
  </li>
</ol>

<hr>

<h2 id="uninstall">Uninstall</h2>

<h3>macOS / Linux</h3>

<pre><code>curl -fsSL https://raw.githubusercontent.com/xtop-cli/xtop/main/install.sh | bash -s -- --uninstall</code></pre>

<h3>Windows</h3>

<pre><code>irm https://raw.githubusercontent.com/xtop-cli/xtop/main/uninstall.ps1 | iex</code></pre>

<hr>

<p align="center">
  <a href="../README.md">Back to README</a>
</p>
