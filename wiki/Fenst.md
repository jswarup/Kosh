# Module Reference: `fenst`

## 1. Overview & Purpose

The `fenst` module provides Kosh with an **extensible virtual explorer, Tauri desktop document viewer, and GPU-accelerated 3D point cloud visualizer**. Key features include:
1. **Virtual Explorer Abstraction (`Xplr`, `LeafXplr`, `BranchXplr`)**: Uniform node interface across filesystems, AST grammar trees, and symbolic expressions.
2. **Provider Registry (`XplrRegistry`)**: URL scheme-based provider routing (`file://`, `expr://`, `ast://`).
3. **High-Performance Content Streaming**: Windowed chunk reading (`XplrFetchChunk`) and safe size-guarded file readers (`XplrFetchContent`).
4. **Desktop GUI Application**: Tauri v2 desktop shell with native menus, multi-windowing, and IPC command handlers (`fenst::xplrcmds`).
5. **Interactive 3D Point Cloud Visualizer**: Real-time perspective projection (`Project3d`), rotating wireframe bounding box, and GPU-computed point rendering.

---

## 2. Architecture & Class Diagram

```mermaid
classDiagram
    class Xplr {
        <<trait>>
        +Name() &str
        +Path() &str
        +IsLeaf() bool
        +AsLeaf() Option~&dyn LeafXplr~
        +AsBranch() Option~&dyn BranchXplr~
        +ToDto(providerScheme) XplrNodeDto
    }

    class LeafXplr {
        <<trait>>
        +Size() u64
        +Extension() &str
    }

    class BranchXplr {
        <<trait>>
        +Children() Result~Vec~Box~dyn Xplr~~~
        +ChildCount() Result~U32~
    }

    class XplrProvider {
        <<trait>>
        +Scheme() &str
        +OpenRoot(uri) Result~Box~dyn BranchXplr~~
    }

    class XplrRegistry {
        -providers: HashMap~String, Box~XplrProvider~~
        +New() XplrRegistry
        +Register(provider)
        +GetProvider(scheme) Option
        +Schemes() Vec~String~
        +OpenRoot(uri) Result
    }

    class FsLeaf {
        +name: String
        +path: String
        +extension: String
    }
    class FsBranch {
        +name: String
        +path: String
    }
    class FsProvider
    class FrescoProvider
    class ShardProvider

    Xplr <|-- LeafXplr : extends
    Xplr <|-- BranchXplr : extends
    LeafXplr <|.. FsLeaf : implements
    BranchXplr <|.. FsBranch : implements
    XplrProvider <|.. FsProvider : implements
    XplrProvider <|.. FrescoProvider : implements
    XplrProvider <|.. ShardProvider : implements
    XplrRegistry o-- XplrProvider : manages
```

---

### 3. Desktop 3D Graphics Scene Multi-GPU Display Pipeline

```mermaid
flowchart TD
    Frontend["Tauri Frontend (WebGL/Canvas/HTML)"] -->|Invokes 'XplrProjectPts'| BackendCmd["fenst::xplrcmds::XplrProjectPts"]
    BackendCmd --> CheckState["Lookup or Initialize PtsSessionState (Singleton)"]
    
    CheckState -->|If initial load| GenPoints["Generate points via GPU Compute Shader (symph::pts_pointcloud_cs)"]
    GenPoints --> InitBBox["Initialize Bounding Box [-20, -20, -20] to [20, 20, 20]"]
    InitBBox --> RotateState["Update angle_x and angle_y based on speed/interaction"]
    
    CheckState -->|Existing session| RotateState
    RotateState --> Cluster["Dispatch ProjectSceneCluster via SwarmCluster (Multi-GPU)"]
    Cluster --> ShardGPU0["GPU 0 / Primary (Viewport Projection Chunk 0)"]
    Cluster --> ShardGPU1["GPU 1..N / Aux (Async Projection Chunks 1..N)"]
    Cluster --> GPUBox["GPU Bounding Box Wireframe (RunCameraTransform)"]
    
    ShardGPU0 --> MergeResults["Concatenate Projected Shards"]
    ShardGPU1 --> MergeResults
    GPUBox --> ConnectBox["Assemble 12 Projected Bounding Box Lines"]
    
    ConnectBox --> AssembleDto["Assemble PtsFrameDto"]
    MergeResults --> AssembleDto
    AssembleDto --> ReturnJSON["Return serializable frame DTO to Frontend for 60+ FPS redraw"]
```

### Multi-GPU Cluster & Dedicated Shaders (`swarm` & `symph`)
1. **Multi-Adapter Discovery (`SwarmCluster`)**:
   - Enumerates all available hardware adapters (discrete and integrated GPUs) via `RustGpuDevice::EnumerateDevices()`.
   - Splits massive point cloud datasets into contiguous chunks and projects them concurrently across all GPUs via `RunCameraTransformSharded`.
2. **GPU Frustum Culling (`frustum_cull_cs`)**:
   - Compute kernel testing 3D points against 6 camera frustum planes on GPU hardware, skipping off-screen points with zero CPU cost.
3. **Vertex Shader (`scene_vs`)**:
   - `#[spirv(vertex)]` entrypoint performing 3D camera model-view-projection to NDC coordinates.
   - Computes depth factor, point size (`gl_PointSize`), and vertex color in parallel per vertex.
4. **Fragment Shader (`scene_fs`)**:
   - `#[spirv(fragment)]` entrypoint evaluating point-sprite radial falloff, depth alpha blending, and glowing core.

---

## 4. 3D Camera & SceneGraph Projection Math

Perspective projection and camera transformations are managed through `Camera` and `SceneGraph` in `fenst::scene`:

$$\begin{aligned}
x_1 &= x \cos(\theta_y) + z \sin(\theta_y) \\
z_1 &= -x \sin(\theta_y) + z \cos(\theta_y) \\
y_2 &= y \cos(\theta_x) - z_1 \sin(\theta_x) \\
z_2 &= y \sin(\theta_x) + z_1 \cos(\theta_x) \\
\text{scale} &= \frac{\text{FOV} \cdot \text{Zoom}}{\text{Distance} + z_2} \quad (\text{FOV} = 350.0, \, \text{Distance} = 250.0) \\
\text{screen}_x &= \frac{\text{width}}{2} + \text{Pan}_x + x_1 \cdot \text{scale} \\
\text{screen}_y &= \frac{\text{height}}{2} + \text{Pan}_y - y_2 \cdot \text{scale}
\end{aligned}$$

---

## 5. Struct & DTO Reference

### Core Scene & Swarm Types (`fenst::scene` & `swarm::engine`)
- `Camera`: Viewport camera (`_PanX`, `_PanY`, `_Zoom`, `_RotX`, `_RotY`, `_Fov`, `_Distance`).
- `SceneGraph`: Active 3D visualization scene owning camera, points, and bounding box geometry.
- `SceneDisplayFrame`: GPU-computed 2D graphics scene display payload (`_Points: Buff<(f32, f32, f32, f32, String)>`, `_BoxLines: Buff<((f32, f32), (f32, f32))>`).
- `SwarmCluster`: Multi-GPU cluster managing primary and auxiliary compute engines (`_Primary`, `_Auxiliary`).

### DTOs (`fenst::mod.rs` & `fenst::xplr.rs`)
- `XplrEntry`: Directory entry (`name`, `path`, `is_dir`, `size`, `extension`).
- `XplrContent`: Full text file payload (`path`, `content`, `size`, `line_count`).
- `XplrLeafInfo`: File metadata (`path`, `name`, `size`, `is_dir`, `modified`, `extension`, `readonly`).
- `XplrNodeDto`: Virtual provider node (`id`, `name`, `is_leaf`, `provider`, `size`, `extension`).
- `StreamChunkDto`: Chunk slice (`path`, `offset`, `length`, `total_size`, `is_eof`, `content`).
- `PtsPointsDto`: Raw point cloud data (`points: Buff<[f32; 3]>`, `count`, `bbox_min`, `bbox_max`).

### GUI Frame DTOs (`fenst::xplrcmds.rs`)
- `ProjectedPoint`: 2D projected point (`x`, `y`, `radius`, `core_radius`, `color`).
- `ProjectedLine`: 2D projected line segment (`x1`, `y1`, `x2`, `y2`).
- `PtsFrameDto`: Complete frame payload containing points, wireframe lines, status text, and bounding box labels.

---

## 6. Tauri IPC Commands Reference (`xplrcmds`)

| Command | Signature | Description |
| :--- | :--- | :--- |
| `XplrListEntries` | `(path: String) -> Result<Buff<XplrEntry>, String>` | Lists files in a directory sorted directories-first. |
| `XplrFetchContent`| `(path: String) -> Result<XplrContent, String>` | Reads full text file up to 10 MB limit using `BuffStream`. |
| `XplrFetchChunk`  | `(path, offset, size) -> Result<StreamChunkDto, String>` | Reads windowed slice of a file. |
| `XplrLeafInfo`    | `(path: String) -> Result<XplrLeafInfo, String>` | Retrieves detailed filesystem metadata. |
| `XplrSelectBranch`| `() -> Result<Option<String>, String>` | Shows native OS folder picker dialog (`rfd`). |
| `XplrChildren`    | `(uri: String) -> Result<Buff<XplrNodeDto>, String>` | Queries virtual explorer provider for child nodes. |
| `XplrListProviders`| `() -> Result<Buff<String>, String>` | Returns registered URI schemes (`"file"`, `"expr"`, `"ast"`). |
| `XplrFetchPtsPoints`| `(path: Option<String>) -> Result<PtsPointsDto, String>` | Generates 3D points from `.pts` file or `rust-gpu` compute shader. |
| `XplrOpenContentWindow`| `(app, path) -> Result<(), String>` | Opens file in separate dedicated webview window. |
| `XplrOpenPtsGraphicsWindow`| `(app, path) -> Result<(), String>` | Opens dedicated 3D shader window for `.pts` point cloud files. |
| `XplrProjectPts`  | `(path, width, height, dpr, speed, color, pan_x, pan_y, zoom, rot_x, rot_y, is_interactive) -> Result<PtsFrameDto, String>` | Projects SceneGraph 3D points & bounding box with pan/zoom/rotate across Multi-GPU cluster. |
| `XplrResetCamera` | `(path: String) -> Result<Camera, String>` | Resets SceneGraph camera pan, zoom, and rotation to defaults. |
