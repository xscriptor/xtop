# Changelog

## [0.2.0] - 2026-06-03

### ♻️ Refactorización Total del Proyecto
- Migrado a **workspace multi-crate**: `xtop-core`, `xtop-tui`, `xtop-cli`
- Eliminado el monolito `src/` — ahora cada capa vive en su propio crate
- Eliminadas **5 dependencias muertas**: `serde`, `serde_json`, `clap`, `chrono`, `tokio` (se redujo de ~84 a ~55 crates)
- Eliminado **código muerto**: `InputMode`, `show_help`, `swap_history`, `process_table_state`, `graph_colors()`
- Eliminados todos los `#[allow(dead_code)]`

### 🏗️ Nueva Arquitectura Hexagonal
- **Capa de Dominio** (`xtop-core/domain/`): modelos de datos puros + trait `SystemDataProvider`
- **Capa de Aplicación** (`xtop-core/application/`): `AppState`, `MetricsHistory` (con `VecDeque`), `LayoutMode`, `EffectiveLayout`
- **Capa de Infraestructura** (`xtop-core/infrastructure/`): `SysinfoProvider`, `theme_loader`, `config`, providers stub
- **Capa de Presentación** (`xtop-tui/`): terminal, render widgets separados, format helpers
- **Binary** (`xtop-cli/`): entry point con inyección de dependencias

### 📐 Layout Responsive
- `detect_effective_layout(width, height, mode)` adapta el layout automáticamente:
  - **Dashboard** (>100×30): layout completo 2-columnas
  - **Compact** (>80×24): más compacto
  - **Vertical** (<80): todo apilado
  - **Minimal** (<60 ancho o <18 alto): solo CPU + Mem + procesos
  - **Too Small** (<40×8): mensaje de advertencia

### 🆕 Nuevos Layouts (7 modos, ciclo con `l`)
| Modo | Descripción |
|------|-------------|
| Dashboard | Default, 2-columnas con gráficos |
| Vertical | Apilado, para terminales estrechas |
| Horizontal | 4 columnas: CPU/Mem/Storage/Network |
| CPU Focus | CPU grande + procesos |
| Memory Focus | Memoria grande con chart + procesos |
| Network Focus | Network + Disk I/O lado a lado + procesos |
| Process Focus | Stats pequeños + procesos maximizados |

### 🆕 Full Screen (`f` / `F`)
- `f` activa/desactiva modo fullscreen
- `F` cicla entre widgets (CPU → Memory → Storage → Network → Processes → Disk I/O → GPU → Battery → salir)
- Widget seleccionado ocupa toda la terminal (menos header)

### 🆕 Búsqueda de Procesos (`/`)
- Filtrado en tiempo real por nombre de proceso
- `Enter` confirma el filtro, `Esc` cancela, `Backspace` borra
- Overlay centrado con indicador `/query_`

### 🆕 Ayuda en Pantalla (`?`)
- Muestra todas las keybindings disponibles
- Cierra con `Esc` o `?` otra vez

### 📊 Nuevas Métricas
- **Disk I/O**: velocidad de lectura/escritura por disco (bytes/s) con widget dedicado
- **Per-interface Network**: RX/TX y velocidad por interfaz de red
- **GPU** (stub): `NoopGpuProvider` — preparado para NVIDIA/AMD
- **Battery** (stub): `NoopBatteryProvider` — preparado para estado de batería
- **Docker** (stub): `NoopDockerProvider` — preparado para contenedores

### ⚠️ Alertas por Threshold
- **CPU > 90%**: color cambia a rojo
- **Memoria > 90%**: color rojo + icono ⚠ en el título
- Thresholds configurables en `AlertThresholds` (cpu_high, mem_high, disk_high)

### 🧹 Mejoras de Código
- `Vec` + `remove(0)` reemplazado por `VecDeque` con `pop_front()` (O(1))
- Helper `format_bytes()` elimina repetición de `1024.0 / 1024.0 / 1024.0`
- Helper `format_uptime()` para formato legible de tiempo activo
- `MetricsHistory::set_max_points()` para configurar puntos del histórico

### ⚙️ Configuración Persistente
- `~/.config/xtop/config.json`: guarda tema, layout, intervalo, history_points, alerts
- `~/.config/xtop/themes/*.json`: temas personalizados por el usuario
- Guardado automático al salir con `q`
- Temas built-in (13) se fusionan con temas personalizados

### 🧪 Tests
- **39 tests unitarios** (de 0): layout detection, history, themes, format helpers, config
- CI workflow `.github/workflows/ci.yml`: check, fmt, clippy, test, build

### 🔑 Keybindings Completos
| Tecla | Acción |
|-------|--------|
| `q` | Salir (guarda config) |
| `?` | Ayuda |
| `t` / `T` | Siguiente/anterior tema |
| `l` | Siguiente layout |
| `f` / `F` | Toggle fullscreen / ciclar widget |
| `/` | Buscar procesos |
| `Esc` | Cancelar búsqueda / cerrar ayuda |
