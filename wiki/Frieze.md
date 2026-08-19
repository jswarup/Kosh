# Module Reference: `frieze`

## 1. Overview & Purpose

The `frieze` module contains **Tauri desktop application resources and frontend assets** for the Kosh GUI. It houses:
1. **Frontend Assets**: HTML, JavaScript, and CSS files comprising the Tauri desktop UI shell.
2. **Application Icons**: Multi-resolution icon files for platform branding and taskbar/window decoration.
3. **Tauri Capabilities**: Security permission manifests defining fine-grained feature access for the desktop application.
4. **Tauri Configuration**: Application metadata, build settings, and bundle configuration.

---

## 2. Directory Structure

```
src/frieze/
├── frontend/
│   ├── index.html              # Main application UI shell
│   ├── app.js                  # Primary JavaScript application logic
│   ├── pts_viewer.html         # 3D point cloud visualization viewport
│   ├── pts_viewer.js           # WebGL/Canvas rendering & interaction
│   └── styles.css              # Unified styling for all views
├── icons/
│   ├── 32x32.png               # Taskbar/small window icon
│   ├── 128x128.png             # Application launcher/dock icon
│   ├── icon.ico                # Windows executable icon
│   └── icon.png                # Generic application icon
├── capabilities/
│   └── default.json            # Default security permissions for Tauri commands
└── tauri.conf.json             # Tauri v2 application configuration & metadata
```

---

## 3. Frontend (`frieze/frontend/`)

### 3.1 `index.html`
Entry point for the Tauri desktop application UI. Defines the root HTML structure, loads `app.js`, and provides the DOM container for the virtual explorer sidebar and dynamic content views.

### 3.2 `app.js`
Primary TypeScript/JavaScript application logic handling:
- **Provider Navigation**: Switches between filesystem (`file://`), expression grammar (`expr://`), and AST grammar (`ast://`) views.
- **Command Invocation**: Bridges frontend UI events to Kosh backend commands via `window.__TAURI__.invoke()`.
- **State Management**: Maintains current explorer path, selected node, and content cache.
- **Content Rendering**: Dynamically loads and displays file contents, point cloud visualizations, and expression trees.

### 3.3 `pts_viewer.html`
Dedicated viewport for **3D point cloud visualization**. Contains a full-screen canvas element and minimal UI controls for:
- Camera rotation (mouse drag)
- Zoom (mouse wheel)
- Panning (CTRL + mouse drag)
- Wireframe bounding box display

### 3.4 `pts_viewer.js`
WebGL/Canvas rendering engine implementing:
- **Point Cloud Rendering**: GPU-accelerated rendering of millions of points using `pts_pointcloud_cs` compute shader.
- **Bounding Box Wireframe**: Draws 12 lines defining the 3D axis-aligned bounding box of loaded point data.
- **Camera Interaction**: Responds to mouse events and computes perspective projection matrices.
- **Real-Time Update**: Calls Tauri `XplrProjectPts` command at 60+ FPS to fetch updated projected frame data.

### 3.5 `styles.css`
Unified CSS stylesheet covering:
- Layout and flexbox for sidebar/content panes
- Dark/light theme support
- Typography and spacing conventions
- Modal dialogs and context menus
- WebGL canvas styling and fullscreen modes

---

## 4. Application Icons (`frieze/icons/`)

The `icons/` directory stores multi-resolution platform icons used in:
- **Windows (.ico)**: Embedded in executable, used by Windows shell.
- **PNG variants (32×32, 128×128)**: Used for macOS/Linux launchers, dock, and taskbar.

The Tauri build process (`build.rs`) bundles these icons as application metadata.

---

## 5. Tauri Capabilities (`frieze/capabilities/`)

### 5.1 `default.json`
Defines the security capability set for Tauri commands. Specifies which backend Rust functions (`fenst::xplrcmds`) are exposed to the frontend JavaScript layer, along with their allowed permissions:
- `XplrBrowseDirectory`: Browse and list files/folders
- `XplrFetchContent`: Read file contents
- `XplrFetchChunk`: Stream large files by chunk
- `XplrProjectPts`: Compute 3D point cloud projections
- `XplrParsePts`: Parse `.pts` point cloud files
- etc.

---

## 6. Tauri Configuration (`frieze/tauri.conf.json`)

```json
{
  "productName": "Fenst",
  "identifier": "com.kosh.fenst",
  "build": {
    "frontendDist": "./src/frieze/frontend",
    "beforeDevCommand": "",
    "beforeBuildCommand": ""
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "src/frieze/icons/32x32.png",
      "src/frieze/icons/128x128.png",
      "src/frieze/icons/icon.ico",
      "src/frieze/icons/icon.png"
    ]
  }
}
```

### Key Configuration Fields
- **`productName`**: Application display name ("Fenst").
- **`identifier`**: Unique bundle identifier for macOS/Linux (`com.kosh.fenst`).
- **`frontendDist`**: Relative path to frontend assets (now `./src/frieze/frontend`).
- **`bundle.targets`**: All platforms (Windows, macOS, Linux).
- **`bundle.icon`**: Array of icon paths for installer and application metadata.

The `build.rs` script copies this configuration to the project root during compilation for Tauri build system consumption.

---

## 7. Build Integration

The `build.rs` script integrates frieze resources into the Kosh build pipeline:

```rust
// Copy tauri.conf.json from src/frieze/ for Tauri build system
let _ = std::fs::copy("src/frieze/tauri.conf.json", "tauri.conf.json");

let attrs = tauri_build::Attributes::new()
    .capabilities_path_pattern("src/frieze/capabilities/*");
tauri_build::try_build(attrs).expect("Failed to build Tauri app");
```

This ensures:
1. Tauri reads application metadata from the frieze configuration
2. Security capabilities are loaded from the frieze manifest
3. Frontend assets are bundled for desktop distribution

---

## 8. Related Modules

- **[`fenst`](Fenst.md)**: Backend module providing IPC command handlers that the frieze frontend invokes.
- **[`symph`](Swarm.md)**: Compute shaders used for 3D point cloud rendering in `pts_viewer.js`.
- **[`swarm`](Swarm.md)**: GPU device abstractions coordinating multi-GPU rendering for `XplrProjectPts`.

---

## 9. Development & Customization

### Modifying Frontend UI
- Edit `src/frieze/frontend/{index.html, app.js, styles.css}` directly.
- Changes are reflected on `cargo run` (development mode) without rebuild.

### Updating Icons
- Replace PNG/ICO files in `src/frieze/icons/`.
- Rebuild with `cargo build` to rebundle icons.

### Extending Security Capabilities
- Add new capability grants in `src/frieze/capabilities/default.json`.
- Expose corresponding backend command handlers in `fenst::xplrcmds`.

### Modifying Application Metadata
- Edit `src/frieze/tauri.conf.json` directly.
- Changes are synced to `tauri.conf.json` (root) during `build.rs` execution.
