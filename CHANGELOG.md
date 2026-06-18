# Changelog

## [0.2.1] - 2026-06-18

### Config: Persistencia de Layouts Personalizados
- `Config` ahora tiene campo `layout_name` que almacena el nombre del layout seleccionado (soporta layouts mas alla de los 7 built-in `LayoutMode`)
- En configuraciones existentes, `layout_name` es opcional (`#[serde(default)]`) y se usa como respaldo `layout_mode`
- `save_config` guarda el nombre del layout actual por su indice en `layout_defs`, permitiendo restaurar layouts personalizados al reiniciar
- `AppState::new` prioriza `config.layout_name` si no es vacio, con fallback a `layout_index_from_mode`
- Config se guarda automaticamente al seleccionar tema o layout desde la paleta de comandos

### Paleta de Comandos: Soporte macOS
- Agregada ruta directa para `ctrl+p` en el bucle de eventos, independiente del sistema de keybindings (soluciona problemas donde crossterm reporta la tecla con distinto casing o el keybinding no se resuelve)
- `key_event_to_str` aplica `to_ascii_lowercase()` al caracter cuando Ctrl esta activo, normalizando `"ctrl+P"` a `"ctrl+p"`
- Agregado `"ctrl+P"` como variante alternativa en el keybinding por defecto
- Debug output de teclas en compilaciones debug (`cargo build` sin `--release`) para diagnosticar problemas: `[key] 'ctrl+p'`

### Plugins: Seguridad y Arquitectura
- `PluginContext` ahora verifica capabilities antes de ejecutar acciones sensibles: `kill_process`, `set_alert_thresholds`, `set_theme_by_name`, `set_layout_by_name`, `set_update_interval` requieren `KillProcesses` o `ModifyConfig` segun corresponda
- `PluginCapability::Custom` migrado de `&'static str` a `String` para flexibilidad
- `PluginManifest` usa `String` en vez de `&'static str`, permitiendo plugins que generen metadata dinamicamente
- `#[non_exhaustive]` agregado a `PluginCapability` para evolucion segura del enum

### PluginManager: Robustez
- `with_plugin_manager_mut()` reemplaza el patron inseguro `take()` + `Some()` que podia perder el manager si una ruta de error no lo restauraba. Todos los callers migrados: `on_tick`, manejo de teclas, shutdown, y MCP server
- `register()` ahora devuelve `Result<(), PluginError>` en vez de `Result<(), String>`, consistente con el resto del sistema
- Metodo `build_context()` elimina la duplicacion de construccion de `PluginContext` en 5 metodos distintos
- `data_dir` del plugin ahora apunta al directorio del plugin (`plugins/<id>/`), no a `plugins/<id>/config.json`

### Capabilities: Validacion Real
- `collect_widgets()` solo acepta widgets si el plugin declara `RenderWidgets`
- `collect_data_providers()` solo acepta providers si el plugin declara `ReadSystemInfo`
- `PluginContext` inyecta las capabilities declaradas y verifica en cada metodo sensible

### MCP Server
- Migrado de `take()`/`Some()` a `with_plugin_manager_mut()` para seguridad
- Eliminado doble `tick_all()` innecesario en el handler de herramientas

### Sistema de Providers
- `SystemDataProvider` ahora tiene metodo `add_extras()` con default no-op, eliminando la necesidad de `downcast_mut::<CompositeProvider>()`
- `kill_process()` verifica que el UID del proceso coincida con el usuario actual antes de enviar SIGTERM
- Limite de procesos configurable via `SysinfoProvider::max_processes` (default 200)
- Eliminados `NoopBatteryProvider`, `NoopGpuProvider`, `NoopDockerProvider` — codigo muerto no utilizado

### Sentinel: Calidad de Datos
- Migracion completa de construccion manual de JSON (`format!` con strings escapados) a `serde_json::json!()` — elimina riesgo de inyeccion JSON y corrupcion por caracteres especiales

### TUI
- `LayoutConstraint::Fill` mapeado correctamente a `ratatui::Constraint::Fill(1)` en vez de `Min(0)`

### Infraestructura
- `config_dir()` centralizada en `xtop_core::infrastructure::config::config_dir()` — eliminada duplicacion en 5 modulos
- CI workflow creado en `.github/workflows/ci.yml`: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`

## [0.2.0] - 2026-06-03

### Refactorizacion Total del Proyecto
- Migrado a workspace multi-crate: `xtop-core`, `xtop-tui`, `xtop-cli`
- Eliminado el monolito `src/` — ahora cada capa vive en su propio crate
- Eliminadas 5 dependencias muertas: `serde`, `serde_json`, `clap`, `chrono`, `tokio` (se redujo de ~84 a ~55 crates)
- Eliminado codigo muerto: `InputMode`, `show_help`, `swap_history`, `process_table_state`, `graph_colors()`
- Eliminados todos los `#[allow(dead_code)]`

### Nueva Arquitectura Hexagonal
- Capa de Dominio (`xtop-core/domain/`): modelos de datos puros + trait `SystemDataProvider`
- Capa de Aplicacion (`xtop-core/application/`): `AppState`, `MetricsHistory` (con `VecDeque`), `LayoutMode`, `EffectiveLayout`
- Capa de Infraestructura (`xtop-core/infrastructure/`): `SysinfoProvider`, `theme_loader`, `config`, providers stub
- Capa de Presentacion (`xtop-tui/`): terminal, render widgets separados, format helpers
- Binary (`xtop-cli/`): entry point con inyeccion de dependencias

### Layout Responsive
- `detect_effective_layout(width, height, mode)` adapta el layout automaticamente:
  - **Dashboard** (>100x30): layout completo 2-columnas
  - **Compact** (>80x24): mas compacto
  - **Vertical** (<80): todo apilado
  - **Minimal** (<60 ancho o <18 alto): solo CPU + Mem + procesos
  - **Too Small** (<40x8): mensaje de advertencia

### Nuevos Layouts (7 modos, ciclo con `l`)
| Modo | Descripcion |
|------|-------------|
| Dashboard | Default, 2-columnas con graficos |
| Vertical | Apilado, para terminales estrechas |
| Horizontal | 4 columnas: CPU/Mem/Storage/Network |
| CPU Focus | CPU grande + procesos |
| Memory Focus | Memoria grande con chart + procesos |
| Network Focus | Network + Disk I/O lado a lado + procesos |
| Process Focus | Stats pequenos + procesos maximizados |

### Full Screen (`f` / `F`)
- `f` activa/desactiva modo fullscreen
- `F` cicla entre widgets (CPU, Memory, Storage, Network, Processes, Disk I/O, GPU, Battery, salir)
- Widget seleccionado ocupa toda la terminal (menos header)

### Busqueda de Procesos (`/`)
- Filtrado en tiempo real por nombre de proceso
- `Enter` confirma el filtro, `Esc` cancela, `Backspace` borra
- Overlay centrado con indicador `/query_`

### Ayuda en Pantalla (`?`)
- Muestra todas las keybindings disponibles
- Cierra con `Esc` o `?` otra vez

### Nuevas Metricas
- **Disk I/O**: velocidad de lectura/escritura por disco (bytes/s) con widget dedicado
- **Per-interface Network**: RX/TX y velocidad por interfaz de red
- **GPU**, **Battery**, **Docker** stubs preparados para implementacion futura

### Alertas por Threshold
- **CPU > 90%**: color cambia a rojo
- **Memoria > 90%**: color rojo + icono de advertencia en el titulo
- Thresholds configurables en `AlertThresholds` (cpu_high, mem_high, disk_high)

### Mejoras de Codigo
- `Vec` + `remove(0)` reemplazado por `VecDeque` con `pop_front()` (O(1))
- Helper `format_bytes()` elimina repeticion de `1024.0 / 1024.0 / 1024.0`
- Helper `format_uptime()` para formato legible de tiempo activo
- `MetricsHistory::set_max_points()` para configurar puntos del historico

### Configuracion Persistente
- `~/.config/xtop/config.json`: guarda tema, layout, intervalo, history_points, alerts
- `~/.config/xtop/themes/*.json`: temas personalizados por el usuario
- Guardado automatico al salir con `q`
- Temas built-in (13) se fusionan con temas personalizados

### Tests
- 39 tests unitarios (de 0): layout detection, history, themes, format helpers, config
- CI workflow `.github/workflows/ci.yml`: check, fmt, clippy, test, build

### Keybindings Completos
| Tecla | Accion |
|-------|--------|
| `q` | Salir (guarda config) |
| `?` | Ayuda |
| `t` / `T` | Siguiente/anterior tema |
| `l` | Siguiente layout |
| `f` / `F` | Toggle fullscreen / ciclar widget |
| `/` | Buscar procesos |
| `Esc` | Cancelar busqueda / cerrar ayuda |
