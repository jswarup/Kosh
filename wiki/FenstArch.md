# Fenst Architecture

The **Fenst** framework provides file exploration, multi-scheme content abstraction, and graphics rendering tools in Kosh. It consists of two primary components:

1. **`fenst` (Core Library)**: Defines URI schemes (`file://`, `ast://`, `fresco://`, `shard://`), file provider registries, content chunking, and GPU point cloud generation via `wgpu` and `rust-gpu`.
2. **`fenst-app` (Tauri Desktop GUI App)**: A lightweight desktop application built on top of Tauri 2.0, providing an interactive file explorer frontend and a dedicated Rust-GPU powered `.pts` point cloud graphics viewer window.

---

## Core Design Principles

1. **Multi-Scheme Provider Architecture**: Content sources are accessed uniformly via the `XplrProvider` trait and `XplrRegistry`, allowing seamless exploration across physical filesystems, abstract syntax trees, term repositories, and binary shards.
2. **Backend-Driven Computations**: All heavy computational tasks—including GPU shader invocation, 3D rotations, perspective projections, depth-sorting, color parsing, and text overlay formatting—are performed in the Rust backend.
3. **Passive Frontend Drawing Stub**: Frontend webviews (such as `pts_viewer.js`) act as pure drawing stubs, receiving pre-computed 2D coordinates and primitives via Tauri IPC and rendering them directly onto HTML5 Canvas elements.
4. **Build-Time Shader Compilation**: Rust-GPU shader crates (`src/gcomp`) are compiled directly to SPIR-V bytecode during `fenst-app` build time using `spirv-builder`, producing zero runtime compiler overhead.

---

## High-Level System Architecture

```
+-----------------------------------------------------------------------+
|                           Tauri Frontend                              |
|   +--------------------------+    +-------------------------------+   |
|   |   Main Explorer Window   |    |   Point Cloud Viewer Window   |   |
|   |  (index.html / app.js)   |    | (pts_viewer.html / viewer.js) |   |
|   +--------------------------+    +-------------------------------+   |
+-----------------------------|-----------------------------------------+
                              | Tauri IPC (invoke)
                              v
+-----------------------------------------------------------------------+
|                      fenst-app Backend (Rust)                         |
|   +---------------------------------------------------------------+   |
|   |   Commands (xplrcmds.rs)                                      |   |
|   |   - XplrListEntries, XplrFetchContent, XplrLeafInfo           |   |
|   |   - XplrOpenContentWindow, XplrOpenPtsGraphicsWindow          |   |
|   |   - XplrProjectPts (State Tracking, 3D Math, Depth Sorting)   |   |
|   +---------------------------------------------------------------+   |
|   |   Session State Cache (PTS_STATE: LazyLock<Mutex<HashMap>>)   |   |
|   +---------------------------------------------------------------+   |
+-----------------------------|-----------------------------------------+
                              |
                              v
+-----------------------------------------------------------------------+
|                         fenst Core Library                            |
|   +---------------------------------------------------------------+   |
|   |   Provider Registry (XplrRegistry)                            |   |
|   |   - FsProvider, FrescoProvider, ShardProvider                 |   |
|   +---------------------------------------------------------------+   |
|   |   GPU Pipeline Dispatch (wgpu)                                |   |
|   |   - XplrFetchPtsPoints (Buffer setup & pipeline dispatch)      |   |
|   +---------------------------------------------------------------+   |
+-----------------------------|-----------------------------------------+
                              | SPIR-V Bytecode Execution
                              v
+-----------------------------------------------------------------------+
|                    gcomp (rust-gpu Shader Crate)                      |
|   - Entry Point: pts_pointcloud_cs (Compute Shader)                   |
|   - PRNG: Wang Hash integer hash -> [-20, 20]³ Vec4 points            |
+-----------------------------------------------------------------------+
```

---

## 1. Provider Abstraction & Exploration Engine

The `fenst` library ([src/fenst](file:///mnt/c/Work/Taregna/Kosh/src/fenst)) defines a extensible provider registry for uniform tree navigation:

### Key Interfaces

* **`XplrProvider` Trait**: Implemented by content providers (`FsProvider`, `FrescoProvider`, `ShardProvider`).
  ```rust
  pub trait XplrProvider: Send + Sync {
      fn Scheme(&self) -> &'static str;
      fn OpenRoot(&self, uri: &str) -> Result<(String, Box<dyn BranchXplr>), String>;
  }
  ```
* **`BranchXplr` & `LeafXplr` Traits**: Abstract directories/branches (`BranchXplr`) and individual files/leaves (`LeafXplr`).
* **`XplrRegistry`**: Global thread-safe registry mapping URI schemes (e.g., `file://`, `ast://`) to their corresponding providers.

### IPC Commands ([xplrcmds.rs](file:///mnt/c/Work/Taregna/Kosh/src/fenst-app/src/xplrcmds.rs))

* **`XplrListEntries(path)`**: Enumerates directory contents sorted with directories first.
* **`XplrFetchContent(path)`**: Reads text content with configurable size guards.
* **`XplrFetchChunk(path, offset, size)`**: Reads windowed binary/text chunks using stream buffering.
* **`XplrChildren(uri)`**: Resolves URI trees dynamically across registered schemes.

---

## 2. Window Lifecycle & Routing

Window creation and focus management are centralized in `fenst-app`:

### `OpenWindowHelper`

A unified helper function in [xplrcmds.rs](file:///mnt/c/Work/Taregna/Kosh/src/fenst-app/src/xplrcmds.rs#L120) eliminates duplicate Tauri WebView construction code:
- Computes a deterministic DJB2 hash of the file path string (`5381` hash multiplier) to form unique window labels (`win_<hash>` or `pts_win_<hash>`).
- Focuses the existing window if a path is already open.
- Builds and opens a new Tauri WebView window when required.

---

## 3. Rust-GPU `.pts` Point Cloud Pipeline

`.pts` files trigger a dedicated hardware-accelerated pipeline integrating **rust-gpu**, **wgpu**, and backend 3D math projection.

### A. Build-Time Shader Compilation (`build.rs`)
During project compilation, `fenst-app`'s [build.rs](file:///mnt/c/Work/Taregna/Kosh/src/fenst-app/build.rs) invokes `spirv_builder`:
- Targets `spirv-unknown-vulkan1.1` to compile the `#![no_std]` crate in `src/gcomp`.
- Emits `GCOMP_SPV_PATH` environment variable.
- Embedded statically at runtime via `include_bytes!(env!("GCOMP_SPV_PATH"))`.

### B. GPU Compute Shader (`src/gcomp/src/lib.rs`)
The `pts_pointcloud_cs` compute shader runs directly on GPU hardware:
- Uses a deterministic 32-bit Wang Hash algorithm (`wang_hash`) to generate pseudo-random 3D coordinates in `[-20.0, 20.0]³`.
- Writes output `Vec4(x, y, z, 1.0)` structs into a GPU storage buffer.

### C. Backend Session State & Projection (`XplrProjectPts`)
Rather than running 3D math on the frontend Javascript engine, [XplrProjectPts](file:///mnt/c/Work/Taregna/Kosh/src/fenst-app/src/xplrcmds.rs#L283) manages state and projection in Rust:

1. **State Storage (`PTS_STATE`)**:
   Uses a thread-safe `LazyLock<Mutex<HashMap<String, PtsSessionState>>>` to store cached point clouds, bounding boxes, and incremental rotation angles (`angle_x`, `angle_y`) per file path.
2. **3D Rotation & Perspective Projection (`Project3d`)**:
   Applies Y-axis and X-axis rotation matrix transformations followed by perspective divide projection (`fov = 350.0`, `distance = 250.0`).
3. **Depth Sorting & Color Mapping**:
   Calculates point radii, depth factors (`alpha`), and parses `#RRGGBB` hex inputs via `ParseHexColor` to generate CSS-ready `rgba(...)` color strings.
4. **Pre-Formatted Overlays**:
   Formats stats overlay strings (`Points: 100 | BBox: [...]`, `Shader Active: gcomp::pts_pointcloud_cs`) directly in Rust.

---

## 4. Frontend Architecture

The frontend ([src/fenst-app/frontend](file:///mnt/c/Work/Taregna/Kosh/src/fenst-app/frontend)) consists of two lightweight WebView views:

1. **Explorer Interface (`index.html` / `app.js`)**:
   - Sidebar tree view navigation, provider scheme list, tab management, and text content viewing.
2. **Point Cloud Viewer (`pts_viewer.html` / `pts_viewer.js`)**:
   - **Minimal Stub Pattern**: Contains zero state tracking or matrix math.
   - Runs a `requestAnimationFrame` loop invoking `XplrProjectPts` with canvas dimensions, scale ratio (`dpr`), rotation speed, and user color selection.
   - Paints returned pre-projected `box_lines`, `points`, and `overlay_text` primitives onto an HTML5 `<canvas>`.

---

## Summary of Verification & Build Targets

- **Compilation**: `cargo build` compiles `gcomp` to SPIR-V, `fenst` core library, and `fenst-app` desktop application.
- **Testing**: `cargo test -p kosh --lib fenst::` runs automated unit tests covering providers, file info retrieval, chunking, and registry dispatch.
