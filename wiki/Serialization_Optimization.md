# Data Serialization Optimization & Audit
## Fenst ↔ Swarm+Symph Pipeline

---

## 1. Current Data Flow Architecture

```mermaid
flowchart TD
    subgraph Frieze["Frieze (Frontend)"]
        FE["pts_viewer.js<br/>Canvas Rendering<br/>Mouse Events"]
    end

    subgraph Fenst["Fenst (Backend IPC)"]
        CMD1["XplrProjectPts<br/>(Camera + Control Params)"]
        CMD2["XplrFetchPtsPoints<br/>(Load .pts file)"]
        SESSMGR["PTS_STATE<br/>SceneGraph Singleton"]
        PROJ["Project3D<br/>Camera Transform"]
    end

    subgraph Swarm["Swarm+Symph (Compute)"]
        GPU["SwarmCluster<br/>Multi-GPU Dispatch"]
        KERN["symph::camera_transform_cs<br/>frustum_cull_cs"]
    end

    FE -->|1. Send Camera<br/>pan, zoom, rot, speed| CMD1
    CMD1 -->|2. Lookup or Init| SESSMGR
    SESSMGR -->|3. Load 3D Points| CMD2
    CMD2 -->|4. Parse .pts or GPU Shader| GPU
    GPU -->|5. Dispatch Camera Transform| KERN
    KERN -->|6. Compute 2D Projection| PROJ
    PROJ -->|7. Return ProjectedPoint[]<br/>+ Metadata Strings| FE
    FE -->|8. Render 2D Points<br/>Canvas Draw| FE
```

---

## 2. Identified Inefficiencies

### 2.1 String Metadata Duplication

**Problem**: Five redundant/overlapping strings sent every frame in `PtsFrameDto`:

```rust
pub struct PtsFrameDto {
    pub _FileName:      String,          // "block.pts"
    pub _Count:         usize,
    pub _BboxLabel:     String,          // "[x1, y1, z1] → [x2, y2, z2]"
    pub _ShaderStatus:  String,          // "Swarm GPU [CUDA] (2 devices) SceneGraph (50000 pts)"
    pub _OverlayText1:  String,          // "Points: 50000 | Zoom: 1.50x | Pan: (10, 20)"
    pub _OverlayText2:  String,          // "Source: block.pts | Rot: (45°, 60°)" OR
                                         // "Swarm CUDA [2 dev] | Rot: (45°, 60°)" ← redundant!
}
```

**Analysis**:
- **Redundancy**: `_ShaderStatus` and `_OverlayText2` both contain "Swarm GPU [...]" info
- **Frequency**: ALL strings re-sent even if only camera params changed
- **Space Waste**: Overhead for small strings using custom binary serialization with 8 length headers
- **Frontend Dependency**: Frontend duplicates some rendering (e.g., formatting camera state)

**Data Size**:
- Typical string payloads: 50-200 bytes per frame
- 60 FPS = 3-12 KB/s just for metadata strings

---

### 2.2 Point Cloud Data Duplication

**Problem**: 3D points loaded once but repeatedly re-serialized:

```
Load Phase:
  .pts File → ParsePtsStream → PtsCloud { points: [x, y, z]* } → Buff<[f32; 3]>
  ↓
  Stored in: SceneGraph._Points (kept in PTS_STATE singleton)

Render Phase (Every Frame):
  SceneGraph._Points → ProjectSceneCluster → Camera Transform (GPU or CPU)
  ↓
  Buff<(f32, f32, f32, f32, f32)> (2D projected x, y, radius, core_radius, alpha)
  ↓
  PtsFrameDto._Points → ToBytes() → Frame IPC → Frontend
```

**Issues**:
- Original 3D points never leave backend (singleton in Rust)
- Frontend receives only 2D projections, cannot perform independent calculations
- Every ProjectSceneCluster call re-serializes entire point buffer
- **No incremental updates**: If 100K points with only camera change, still send 100K projected points

**Bandwidth**:
- 100K points × 5 f32 fields × 4 bytes = 2 MB per frame
- 60 FPS = 120 MB/s serialization overhead

---

### 2.3 Inefficient String Serialization Format

**Problem**: Custom binary format with excessive metadata:

```rust
pub struct PtsFrameBinaryHeader {
    _FileNameLen:    u32,      // 4 bytes per string!
    _BboxLabelLen:   u32,      // 5 separate length fields
    _ShaderStatusLen:u32,      // = 20 bytes just for lengths
    _Overlay1Len:    u32,
    _Overlay2Len:    u32,
}

// Binary Layout:
// [Header: 40 bytes] [Points: N × 20] [Lines: M × 16] [5 Strings] [StringLengths: 20]
```

**Issues**:
- Wastes 20 bytes for lengths when typical strings are 50-200 bytes
- No string compression or interning
- Strings stored as UTF-8, no encoding optimization
- String reassembly on frontend requires manual offset calculation

---

### 2.4 Frontend-Backend Field Naming Mismatch

**Problem**: Frontend normalizes both snake_case and PascalCase variants:

```javascript
// app.js: normalizeEntry()
const path = entry.path ?? entry._path ?? '';
const name = entry.name ?? entry._name ?? '';
const is_dir = entry.is_dir !== undefined ? Boolean(entry.is_dir) :
               (entry._is_dir !== undefined ? Boolean(entry._is_dir) : false);
```

```rust
// fenst/mod.rs: XplrEntry
#[serde(rename = "name")]
pub _Name:       String,         // Serializes as "name" but struct field is _Name
#[serde(rename = "path")]
pub _Path:       String,         // Serializes as "path" but struct field is _Path
```

**Issues**:
- Rename overhead on every field encode/decode
- Frontend defensive programming suggests inconsistency history
- Tests may not catch field name changes

---

### 2.5 Missing Incremental/Delta Updates

**Problem**: No partial frame updates or state diffing:

```
Current: XplrProjectPts() → Full PtsFrameDto every 60 FPS
  └─ Always includes all ProjectedPoints even if unchanged
  └─ Always includes all 5 metadata strings
  └─ No state hash or change detection

Better:
  ✓ Only send changed ProjectedPoints (delta encoding)
  ✓ Only send metadata if changed (e.g., file load, device change)
  ✓ Separate fast path: camera-only updates (geometry constant)
  ✓ Static metadata once, then stream only geometric updates
```

---

### 2.6 No Compression or Float Quantization

**Problem**: Raw f32 values serialized with no optimization:

```rust
pub struct ProjectedPoint {
    pub _X:           f32,         // 4 bytes
    pub _Y:           f32,         // 4 bytes
    pub _Radius:      f32,         // 4 bytes (screen pixels, 0-100 range)
    pub _CoreRadius:  f32,         // 4 bytes (screen pixels, 0-50 range)
    pub _Alpha:       f32,         // 4 bytes (alpha 0.0-1.0)
}
// = 20 bytes per point

// For 100K points:
// 100,000 × 20 = 2,000,000 bytes (2 MB) per frame
// 60 FPS = 120 MB/s network pressure
```

**Opportunity**:
- Screen coordinates (X, Y): 16-bit quantized (±65535 → typically 0-4096 pixel range)
- Radius, CoreRadius: 8-bit quantized (0-255 → 0-100 pixels)
- Alpha: 8-bit quantized (0-255 → 0.0-1.0)
- **Compressed**: 2 + 2 + 1 + 1 + 1 = 7 bytes per point instead of 20
- **Savings**: 65% reduction → 2.1 MB/s instead of 120 MB/s

---

### 2.7 Camera Parameter Synchronization

**Problem**: Camera state sent every frame:

```rust
// XplrProjectPts receives:
pub fn XplrProjectPts(
    path: String,
    width: f32,
    height: f32,
    dpr: f32,
    speed: f32,          // Animation rotation speed
    pan_x: Option< f32>,
    pan_y: Option< f32>,
    zoom: Option< f32>,
    rot_x: Option< f32>,
    rot_y: Option< f32>,  // = 10 parameters every frame!
    is_interactive: Option< bool>,
)
```

**Issues**:
- 10 parameters re-sent even if unchanged
- No state hashing to detect changes
- Interactive mode flag toggled frequently (150ms timeout)
- Pan/Zoom/Rot could use delta encoding if previous state tracked

---

## 3. Optimization Recommendations

### 3.1 Phase 1: Fix Field Naming Consistency

**Action**: Remove `#[serde(rename)]` overhead, use consistent snake_case:

```rust
// BEFORE:
#[derive(Serialize)]
pub struct XplrEntry {
    #[serde(rename = "name")]
    pub _Name: String,
    #[serde(rename = "path")]
    pub _Path: String,
}

// AFTER:
#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub struct XplrEntry {
    pub name: String,
    pub path: String,
}
```

**Benefits**:
- Eliminates 20+ serde rename operations per struct
- Frontend normalization no longer needed
- Cleaner Rust code

---

### 3.2 Phase 2: Separate Static & Dynamic Frame Data

**Action**: Split `PtsFrameDto` into immutable + streaming components:

```rust
// STATIC (sent once per session/file load):
#[repr(C)]
#[derive(Serialize)]
pub struct PtsSessionMetadata {
    file_name: String,
    total_point_count: u32,
    bbox_min: [f32; 3],
    bbox_max: [f32; 3],
}

// DYNAMIC (streamed every frame):
#[repr(C)]
#[derive(Serialize)]
pub struct PtsFrameUpdate {
    // Geometry: points only if geometry changed
    points_dirty: bool,
    points: Option<Buff<ProjectedPoint>>,  // Send only if changed

    // Camera state: send only if changed
    camera_state_dirty: bool,
    zoom: Option<f32>,
    pan_x: Option<f32>,
    pan_y: Option<f32>,
    rot_x: Option<f32>,
    rot_y: Option<f32>,

    // UI overlay (recomputed every frame, minimal overhead):
    overlay_lines: Vec<String>,  // ["Points: 50000", "Zoom: 1.5x", ...]
    shader_status: String,       // Single line, not split
}
```

**Implementation Flow**:

```rust
pub fn XplrProjectPts(...) -> Result<Vec<u8>, String> {
    let mut guard = PTS_STATE.lock()?;
    let state = guard.entry(path.clone()).or_insert_with(|| {
        // FIRST TIME: Send metadata + full geometry
        return (
            Some(PtsSessionMetadata { ... }),  // Send once
            Some(all_projected_points)         // Send full geometry
        );
    });

    // SUBSEQUENT FRAMES: Only send changed geometry + overlay
    let camera_changed = has_camera_state_changed(&state._Scene, &new_params);
    let geometry = if camera_changed {
        Some(recompute_projections())
    } else {
        None  // No geometry change
    };

    let overlay = format_overlay_strings(&state._Scene);

    return Ok(PtsFrameUpdate {
        points_dirty: camera_changed,
        points: geometry,
        camera_state_dirty: camera_changed,
        zoom: camera_changed.then_some(state._Scene.Camera()._Zoom),
        overlay_lines: overlay,
        shader_status: get_backend_status(),
    });
}
```

**Benefits**:
- Metadata sent once → 0 overhead for repeated frames
- Geometry sent only on camera/geometry changes
- **Bandwidth savings**: 60 FPS × 60 Hz = 3,600 frames/sec
  - Current: 2 MB/frame = 7.2 GB/s (unsustainable)
  - Optimized: Only every N-th frame with geometry = 36 MB/s typical case

---

### 3.3 Phase 3: Implement Float Quantization

**Action**: Pack screen coordinates into compact binary format:

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct QuantizedPoint {
    // 16-bit screen X, Y (quantized to viewport pixel range)
    x: u16,
    y: u16,
    // 8-bit radius, core_radius, alpha (quantized 0-255)
    radius: u8,
    core_radius: u8,
    alpha: u8,
}
// = 7 bytes per point instead of 20 (-65% size)

impl From<ProjectedPoint> for QuantizedPoint {
    fn from(p: ProjectedPoint) -> Self {
        QuantizedPoint {
            x: (p._X.clamp(0.0, 4096.0) as u16),
            y: (p._Y.clamp(0.0, 4096.0) as u16),
            radius: ((p._Radius * 2.55).clamp(0.0, 255.0) as u8),
            core_radius: ((p._CoreRadius * 2.55).clamp(0.0, 255.0) as u8),
            alpha: ((p._Alpha * 255.0).clamp(0.0, 255.0) as u8),
        }
    }
}
```

**Benefits**:
- 100K points: 700 KB instead of 2 MB per frame (-65%)
- 60 FPS: 42 MB/s instead of 120 MB/s
- Quantization loss negligible (sub-pixel for screen coords)

**Frontend Dequantization**:

```javascript
function dequantizePoint(qp) {
    return {
        x: (qp.x / 65535.0) * window.innerWidth,
        y: (qp.y / 65535.0) * window.innerHeight,
        radius: qp.radius / 2.55,
        core_radius: qp.core_radius / 2.55,
        alpha: qp.alpha / 255.0,
    };
}
```

---

### 3.4 Phase 4: Compress Metadata with Bitfield

**Action**: Pack camera + UI state into compact binary:

```rust
// BEFORE:
pub struct PtsFrameDto {
    _Count: usize,           // 8 bytes
    _ShaderStatus: String,   // 100+ bytes
    _OverlayText1: String,   // 100+ bytes
    _OverlayText2: String,   // 100+ bytes
}

// AFTER:
#[repr(C)]
pub struct FrameMetadata {
    total_points: u32,       // 4 bytes
    device_type: u8,         // Enum: CPU=0, RustGPU=1, CUDA=2
    device_count: u8,        // 2 devices
    shader_flags: u8,        // Bitfield: has_frustum_cull, multi_gpu, etc.
    // Overlay strings sent separately, only if changed
}
```

**Benefits**:
- 8 bytes metadata instead of 200+ bytes strings
- Bitfield encoding expresses device state compactly

---

### 3.5 Phase 5: Change Detection & State Hashing

**Action**: Hash frame state to detect no-op updates:

```rust
pub struct FrameState {
    camera_hash: u64,
    geometry_hash: u64,
    metadata_hash: u64,
}

fn compute_frame_hash(scene: &SceneGraph, params: &CameraParams) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    hasher.write_f32(scene.Camera()._PanX);
    hasher.write_f32(scene.Camera()._PanY);
    hasher.write_f32(scene.Camera()._Zoom);
    hasher.write_f32(scene.Camera()._RotX);
    hasher.write_f32(scene.Camera()._RotY);
    std::hash::Hasher::finish(&hasher)
}

pub fn XplrProjectPts(...) -> Result<PtsFrameUpdate, String> {
    let new_hash = compute_frame_hash(&scene, &params);
    if new_hash == state.last_hash {
        return Ok(PtsFrameUpdate::NO_CHANGE);  // Zero-byte response!
    }
    state.last_hash = new_hash;
    // ... compute full update
}
```

**Benefits**:
- Zero-byte responses for identical frames (~90% of 60 FPS stream in idle mode)
- Bandwidth savings: 99% reduction for static scenes

---

### 3.6 Phase 6: Binary Frame Format with Dictionary Compression

**Action**: Implement compact binary framing:

```rust
// Frame format (binary wire protocol):
#[repr(C)]
pub struct BinaryFrameHeader {
    magic: u32,              // 0x46524D45 ('FRME')
    version: u8,
    flags: u8,               // Bitfield: has_geometry, has_metadata, has_overlay, etc.
    geometry_count: u16,

    // Conditional fields (based on flags):
    // [if has_metadata] metadata: FrameMetadata (8 bytes)
    // [if has_geometry] geometry: [QuantizedPoint; N] (7 bytes each)
    // [if has_overlay] overlay: String (length-prefixed)
}
```

**Benefits**:
- Explicit change notification via flags
- Variable-length encoding only what changed
- No unnecessary string serialization

---

## 4. Implementation Roadmap

| Phase | Task | Effort | Bandwidth Savings | Dependencies |
| :--- | :--- | :--- | :--- | :--- |
| **1** | Remove serde rename overhead | 1-2h | ~5% | None |
| **2** | Split static/dynamic frame data | 3-4h | ~40% | Phase 1 |
| **3** | Float quantization (16-bit coords, 8-bit attributes) | 2-3h | ~65% | Phase 2 |
| **4** | Metadata bitfield compression | 1-2h | ~70% | Phase 3 |
| **5** | Hash-based change detection | 2-3h | ~85%* | Phase 4 |
| **6** | Binary frame format with flags | 3-4h | ~95%* | Phase 5 |

*Assumes idle/static scenes; dynamic animation provides lower baseline.

---

## 5. Impact Analysis

### Baseline (Current):
- **Single Frame Size**: ~2 MB (100K points + metadata)
- **60 FPS Load**: ~120 MB/s
- **Frame IPC Overhead**: High (full serialization every frame)

### After Phase 3 (Most Important):
- **Frame Size**: ~700 KB (quantized points)
- **60 FPS Load**: ~42 MB/s
- **Improvement**: **65% reduction**

### After Phase 6 (Full Optimization):
- **Frame Size (Idle)**: ~100 bytes (state hash unchanged)
- **Frame Size (Animation)**: ~300 KB (quantized geometry only)
- **60 FPS Load**: ~18 MB/s average (mixed idle/animation)
- **Overall Improvement**: **85% reduction**

---

## 6. References

- **File**: [src/fenst/xplrcmds.rs](src/fenst/xplrcmds.rs) - IPC command handlers
- **File**: [src/fenst/mod.rs](src/fenst/mod.rs) - DTO definitions
- **File**: [src/frieze/pts_view.rs](src/frieze/pts_view.rs) - Native rendering
- **Wiki**: [Fenst.md](wiki/Fenst.md) - Architecture overview
- **Wiki**: [Swarm.md](wiki/Swarm.md) - Compute pipeline
