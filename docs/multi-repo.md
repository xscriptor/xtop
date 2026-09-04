# Multi-repo architecture (xtop-cli org)

> Estado: propuesta inicial. Este doc vive en el kernel pero describe todos los repos.

## Organización

| Repo (xtop-cli) | Rol | Contenido |
|---|---|---|
| `xtop` | **Kernel** — la app | monocrate `src/` por áreas (commands, config, plugins, providers, state, theme, ui). Nada más |
| `api` | **Contratos** | workspace: `crates/plugin-api`, `widget-api`, `effect-api`, `extension-api` → crates publicados `xtop-plugin-api`, `xtop-widget-api`, `xtop-effect-api`, `xtop-extension-api` |
| `layouts` | Layouts data-driven | repo `layouts`: crate `xtop-layout` (model + loader jsonc + modos, sin UI) + `layouts/default/` (7 built-ins) + `layouts/custom/` (comunidad, instalables) |
| `plugins` | Implementaciones de plugins | workspace `plugins/xtop-plugin-*` (1er miembro: samurai) |
| `effects` | Efectos visuales TUI | workspace `effects/xtop-effect-*` (+ `effects-lib` compartido) |
| `extensions` | Hooks/add-ons del kernel | workspace `extensions/xtop-extension-*` |
| `widgets` | Packs de widgets | repo `widgets`: pack base `xtop-widgets` + packs alternativos (`packs/xtop-widget-blocks`) + `custom/` comunidad, contra `xtop-widget-api`. El kernel solo conserva engine + estado |

Layout local de desarrollo (repos hermanos, como hoy):

```
/home/x/xtop-cli/xtop/
  xtop/  api/  plugins/  effects/  extensions/
```

## Principio: dependencias en árbol

Hoy el kernel define los traits de plugin y `xtop-plugin-samurai` depende de
`xtop-core`. Eso impide separar repos: el kernel es a la vez host y contrato.

Objetivo (espejo de `xfetch-cli`):

```
                ┌────────────┐
                │    api     │  crates puros de contrato (sin dep del kernel)
                └─────┬──────┘
         ┌────────────┼────────────────┐
         ▼            ▼                ▼
  ┌───────────┐ ┌─────────────┐ ┌──────────────┐
  │  kernel   │ │  plugins/   │ │ effects/     │
  │  xtop     │ │  effects/   │ │ extensions/  │
  │  (host)   │ │  extensions │ │              │
  └───────────┘ └─────────────┘ └──────────────┘
```

- **api**: tipos puros + protocolo (manifest, capabilities, errores, snapshot,
  provider trait, widget registration, frames de efecto, hooks de extensión).
  Depende solo de `ratatui`/`serde`. Publicado a crates.io en su momento.
- **kernel**: implementa el *host* (PluginManager, CompositeProvider, pipeline
  de render, hooks) contra los tipos de api. Sin plugins sigue compilando:
  la integración es opcional.
- **plugins/effects/extensions**: consumen api únicamente → cada repo compila
  standalone y nunca depende del kernel.

## Qué se mueve de xtop-core a api (Fase 1)

Candidatos directos (tipo "contrato"):

- `domain/plugin.rs` → `xtop-plugin-api`: `PluginCapability`, `PluginManifest`,
  `PluginError`, `PluginContext` (vía un trait de host, no `AppState` directo),
  `WidgetRegistration`, trait `Plugin`.
- `domain/system_info.rs` (trait `SystemDataProvider`) + tipos de datos de
  `domain/metrics.rs` (`SystemSnapshot`, `ProcessInfo`, `SystemInfo`) →
  `xtop-plugin-api` o crate de datos compartido, porque providers/widgets
  externos necesitan esos tipos sin importar el kernel.

Se queda en `xtop-core`: sysinfo real, `AppState`, `PluginManager`,
config/themes/layouts, keybindings, alerts. El kernel reexporta los tipos de
api para no romper los callers internos durante la transición.

Nota `PluginContext`: hoy expone `&mut AppState` (estado vivo). Para que el
contrato sea externo, `PluginContext` debe moverse al host: api define un trait
`HostContext`/`XtapContext` que el kernel implementa y el plugin consume.
(La otra vía —estado vivo por valor— choca con el modelo runtime futuro.)

## Formas de integración (modular opcional)

| Nivel | Mecanismo | Uso |
|---|---|---|
| Compile-time (hoy) | feature flag + dep opcional sobre api/plugins | Built-ins del kernel |
| Dev-time | `xtop plugin install <name>` (clona, compila, registra) | Primeros pasos |
| Runtime (futuro) | discovery de binarios `xtop-plugin-*` / `xtop-effect-*` / `xtop-extension-*` en dirs de config + env `XTOP_*_DEV_DIR` | Terceros, sin recompilar |

El kernel nunca exige ningún repo externo: `cargo build --release
--no-default-features` = core puro.

## Fases

1. **F0 (hecha)**: org `xtop-cli`, repos creados (`xtop` movido con historial;
   `api`, `plugins`, `effects`, `extensions` iniciados), clones locales.
2. **F1**: api crates esqueleto; extraer tipos contrato de xtop-core; kernel
   dependiendo de `../api` (path) y verde de nuevo.
3. **F2**: mover samurai → `plugins/` (subtree split con historia); convertir
   su dep a api (sin xtop-core); kernel: feature apunta al repo plugins
   (path dev → git dep); actualizar URLs `xtop-cli/xtop` → `xtop-cli/*`
   en help, docs, install.sh y CI; `plugin scaffold` apunta al nuevo repo.
4. **F3**: effects: `effect-api` + runner en el TUI + primer efecto demo.
5. **F4**: extensions: `extension-api` (hooks pre/post render, config, tema,
   layout) + primer add-on demo.
6. **F5**: publicar api a crates.io; deps registry versionadas + tags; CI por
   repo; release del kernel.

## Deuda detectada al mover

- 9 archivos del kernel aún referencian `xtop-cli/xtop` (help, docs,
  install.sh/ps1, PKGBUILD, README, CONTRIBUTING).
- `cmd_plugin_install`/`cmd_plugin_list` asumen plugin dentro del repo kernel
  (`plugins/` miembro del workspace) → deberán apuntar al repo `plugins`.
- LICENSE del kernel dice "Copyright (c) 2025 Xscriptor" → decidir si pasa a
  la org xtop-cli.
