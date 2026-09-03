<h1>Plugin System</h1>

<p>xtop has a compile-time plugin system based on Rust feature flags. Plugins live in the <code>plugins/</code> directory and are registered into the workspace as optional dependencies.</p>

<p>You can build xtop without any plugins:</p>

<pre><code>cargo build --release --no-default-features</code></pre>

<p>Or with a specific set of plugins:</p>

<pre><code>cargo build --release --features plugin-samurai</code></pre>

<hr>

<h2 id="architecture">Architecture</h2>

<p>The plugin system has four main components:</p>

<table>
  <thead>
    <tr><th>Component</th><th>Location</th><th>Purpose</th></tr>
  </thead>
  <tbody>
    <tr>
      <td><code>Plugin</code> trait</td>
      <td><code>xtop-core::domain::plugin</code></td>
      <td>Interface every plugin must implement</td>
    </tr>
    <tr>
      <td><code>PluginManager</code></td>
      <td><code>xtop-core::application::plugin_manager</code></td>
      <td>Loads, ticks, and dispatches events to plugins</td>
    </tr>
    <tr>
      <td><code>CompositeProvider</code></td>
      <td><code>xtop-core::infrastructure::composite_provider</code></td>
      <td>Merges data from primary provider + plugin providers</td>
    </tr>
    <tr>
      <td><code>WidgetRegistration</code></td>
      <td><code>xtop-core::domain::plugin</code></td>
      <td>Allows plugins to register custom TUI widgets</td>
    </tr>
  </tbody>
</table>

<hr>

<h2 id="the-plugin-trait">The <code>Plugin</code> Trait</h2>

<pre><code>pub trait Plugin: Debug + Send {
    fn manifest(&self) -> PluginManifest;
    fn on_enable(&mut self, ctx: &mut PluginContext) -> Result&lt;(), PluginError&gt;;
    fn on_disable(&mut self, ctx: &mut PluginContext) -&gt; Result&lt;(), PluginError&gt;;
    fn on_tick(&mut self, ctx: &mut PluginContext) -&gt; Result&lt;(), PluginError&gt;;
    fn on_key(&mut self, ctx: &mut PluginContext, key: &str) -&gt; Result&lt;bool, PluginError&gt;;
    fn data_provider(&self) -&gt; Option&lt;Box&lt;dyn SystemDataProvider&gt;&gt;;
    fn widget(&self) -&gt; Option&lt;WidgetRegistration&gt;;
    fn execute(&mut self, ctx: &mut PluginContext, action: &str, params: &str) -&gt; Result&lt;String, PluginError&gt;;
}</code></pre>

<table>
  <thead>
    <tr><th>Method</th><th>Default</th><th>Called When</th></tr>
  </thead>
  <tbody>
    <tr><td><code>manifest()</code></td><td>required</td><td>Any time metadata is needed</td></tr>
    <tr><td><code>on_enable()</code></td><td>no-op</td><td>Plugin is registered at startup</td></tr>
    <tr><td><code>on_disable()</code></td><td>no-op</td><td>xtop shuts down</td></tr>
    <tr><td><code>on_tick()</code></td><td>no-op</td><td>Every update cycle (~1s)</td></tr>
    <tr><td><code>on_key()</code></td><td>returns <code>false</code></td><td>Key press (consumes if returns <code>true</code>)</td></tr>
    <tr><td><code>data_provider()</code></td><td><code>None</code></td><td>Startup (merged into CompositeProvider)</td></tr>
    <tr><td><code>widget()</code></td><td><code>None</code></td><td>Every tick (refreshes widget registry)</td></tr>
    <tr><td><code>execute()</code></td><td><code>UnknownAction</code></td><td>External agent (AI, CLI, IPC) invokes command</td></tr>
  </tbody>
</table>

<h3 id="plugincapability">PluginCapability</h3>

<p>Each plugin declares what it needs via <code>manifest().capabilities</code>:</p>

<ul>
  <li><code>ReadSystemInfo</code> -- access system metrics</li>
  <li><code>KillProcesses</code> -- terminate processes</li>
  <li><code>ModifyConfig</code> -- change themes, layouts, thresholds</li>
  <li><code>RenderWidgets</code> -- register custom TUI widgets</li>
  <li><code>Custom(&amp;str)</code> -- anything not covered above</li>
</ul>

<h3 id="plugincontext">PluginContext</h3>

<p>Safe, limited access to application state:</p>

<pre><code>ctx.snapshot()             // Full SystemSnapshot
ctx.top_processes(n)       // Top N processes by CPU
ctx.kill_process(pid)      // Kill process by PID
ctx.set_alert_thresholds(cpu, mem, disk)
ctx.set_theme_by_name("tokio")
ctx.set_layout_by_name("Dashboard")
ctx.set_update_interval(500)
ctx.system_info()          // Hostname, OS, kernel
ctx.data_dir()             // ~/.config/xtop/plugins/&lt;id&gt;/</code></pre>

<hr>

<h2 id="compositeprovider">CompositeProvider</h2>

<p>The <code>CompositeProvider</code> wraps the primary <code>SysinfoProvider</code> and merges data from plugin providers:</p>

<ul>
  <li><code>refresh_all()</code> refreshes all providers</li>
  <li><code>snapshot()</code> delegates to primary for CPU, memory, disks, networks, processes</li>
  <li><code>gpu_info()</code>, <code>batteries()</code>, <code>docker_info()</code> check extras if primary returns empty</li>
  <li><code>kill_process()</code> tries primary first, then extras</li>
</ul>

<hr>

<h2 id="widget-registration">Widget Registration</h2>

<p>Plugins can register custom widgets via <code>widget()</code>:</p>

<pre><code>fn widget(&self) -> Option&lt;WidgetRegistration&gt; {
    Some(WidgetRegistration {
        name: "samurai".to_string(),
        render: Arc::new(|f, state, area| {
            // Draw using ratatui
        }),
    })
}</code></pre>

<p>Custom widgets are placed in layouts by name in JSONC layout files:</p>

<pre><code>{
    "name": "my-layout",
    "root": {
        "direction": "vertical",
        "areas": [
            { "widget": "header", "size": 3 },
            { "widget": "samurai", "size": "30%" },
            { "widget": "processes", "size": "*" }
        ]
    }
}</code></pre>

<p>Plugin widgets take precedence over built-in widgets with the same name.</p>

<hr>

<h2 id="cli-commands">CLI Commands</h2>

<table>
  <thead>
    <tr><th>Command</th><th>Description</th></tr>
  </thead>
  <tbody>
    <tr>
      <td><code>xtop plugin list</code></td>
      <td>List installed plugins from workspace members</td>
    </tr>
    <tr>
      <td><code>xtop plugin install &lt;name&gt;</code></td>
      <td>Install a plugin from <code>github.com/xtop-cli/xtop/plugins/</code></td>
    </tr>
    <tr>
      <td><code>xtop plugin install &lt;url&gt;</code></td>
      <td>Install a plugin from any git URL</td>
    </tr>
    <tr>
      <td><code>xtop plugin scaffold &lt;name&gt;</code></td>
      <td>Create a new plugin crate template in <code>plugins/</code></td>
    </tr>
  </tbody>
</table>

<h3 id="install-flow">Install Flow</h3>

<p>When running <code>xtop plugin install samurai</code>:</p>

<ol>
  <li>Clones <code>github.com/xtop-cli/xtop.git</code> (shallow, sparse)</li>
  <li>Looks for <code>plugins/xtop-plugin-samurai/</code> or <code>plugins/samurai/</code> in the clone</li>
  <li>Copies to local <code>plugins/</code> directory</li>
  <li>Adds entry to <code>[workspace].members</code> in root <code>Cargo.toml</code></li>
  <li>Adds optional dependency + feature flag in <code>crates/xtop-cli/Cargo.toml</code></li>
  <li>Runs <code>cargo build --release</code></li>
  <li>Cleans up temporary files</li>
</ol>

<p>The plugin is registered in the workspace but <strong>not enabled by default</strong>. To enable it:</p>

<ul>
  <li>Build with <code>--features plugin-&lt;name&gt;</code> for a one-off build</li>
  <li>Add it to the <code>default</code> list in <code>crates/xtop-cli/Cargo.toml</code> to enable permanently</li>
</ul>

<pre><code># Build xtop with samurai plugin enabled
cargo build --release --features plugin-samurai

# Build xtop with samurai + another plugin
cargo build --release --features "plugin-samurai,plugin-mything"</code></pre>

<hr>

<h2 id="mcp-server">MCP Server for AI Agents</h2>

<p>When the <code>plugin-samurai</code> feature is enabled, xtop can run an MCP (Model Context Protocol) server on stdio:</p>

<pre><code>xtop mcp</code></pre>

<p>This exposes Samurai's commands as MCP tools that any AI assistant can call.
Compatible clients include Claude Desktop, Cline, Cursor, and Continue.dev.</p>

<h3>Claude Desktop configuration</h3>

<pre><code>{
  "mcpServers": {
    "xtop": {
      "command": "xtop",
      "args": ["mcp"]
    }
  }
}</code></pre>

<h3>Interactive testing</h3>

<pre><code># Single command
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"system_summary","arguments":{}}}' | xtop mcp

# Interactive session
xtop mcp</code></pre>

<h3>Monitoreo activo (polling)</h3>

<p>La IA puede llamar <code>system_summary</code> o <code>alerts.status</code> periodicamente. Para monitoreo proactivo (push), haria falta implementar MCP Resources + notificaciones.</p>

<hr>

<h2 id="adding-a-plugin-manually">Adding a Plugin Manually</h2>

<ol>
  <li>
    <p>Create the crate in <code>plugins/</code>:</p>
    <pre><code>mkdir -p plugins/xtop-plugin-mything/src</code></pre>
  </li>
  <li>
    <p>Add to workspace <code>Cargo.toml</code>:</p>
    <pre><code>[workspace]
members = [
    ...
    "plugins/xtop-plugin-mything",
]</code></pre>
  </li>
  <li>
    <p>Add dependency + feature in <code>crates/xtop-cli/Cargo.toml</code>:</p>
    <pre><code>[dependencies]
xtop-plugin-mything = { path = "../../plugins/xtop-plugin-mything", optional = true }

[features]
plugin-mything = ["dep:xtop-plugin-mything"]</code></pre>
  </li>
  <li>
    <p>Register in <code>crates/xtop-cli/src/main.rs</code>:</p>
    <pre><code>#[cfg(feature = "plugin-mything")]
use xtop_plugin_mything::MythingPlugin;

// In build_plugin_manager():
#[cfg(feature = "plugin-mything")]
{
    let plugin = Box::new(MythingPlugin::new());
    if let Err(e) = mgr.register(plugin, state) {
        eprintln!("[xtop] failed to load mything plugin: {e}");
    }
}</code></pre>
  </li>
</ol>

<hr>

<p align="center">
  <a href="../README.md">Back to README</a>
</p>
