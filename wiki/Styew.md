# Module Reference: `styew`

## Overview

`styew` is Kosh's primary native desktop application. It is built with `eframe` and `egui`, with `wgpu` available through the workspace dependencies. The root binary launches it by default:

```powershell
cargo run
```

The window is titled **Kosh — Native 3D GPU Workspace** and starts with a preferred size of 1360 x 840 pixels, with an 800 x 600 pixel minimum.

## Scope

The module lives in `src/styew/` and is composed of the following application surfaces:

| Source file | Responsibility |
| :--- | :--- |
| `app.rs` | Main `KoshApp` application type and frame lifecycle. |
| `state.rs` | Shared application state. |
| `tab_bar.rs` | Tab selection and workspace navigation. |
| `explorer.rs` | Explorer user interface. |
| `pts_view.rs` | Point-cloud view. |
| `obj_view.rs` | Wavefront OBJ view. |
| `fresco_view.rs` | Symbolic-expression view. |
| `mod.rs` | Public module declarations, `KoshApp` re-export, and `run()` entry point. |

## Launch Behavior

`src/main.rs` accepts these application modes:

| Command | Behavior |
| :--- | :--- |
| `cargo run` | Launches the primary native `styew` workspace. |
| `cargo run -- --aura` | Launches the secondary Tauri-based `fenst` application. |
| `cargo run -- --test [FILTER]` | Runs the Cargo test suite, optionally filtered by name. |

The `--verbose` flag enables debug-level tracing before application startup. The `--nocapture` flag is used with `--test` to expose test output.

## Relationship to Other Frontends

`styew` is the native application layer. [`fenst`](Fenst.md) provides the separate Tauri explorer and its provider/IPC interfaces, while [`aura`](Aura.md) contains the Tauri frontend assets and configuration. These are distinct launch paths rather than interchangeable UI implementations.
