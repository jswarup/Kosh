# Optimization Guide: Real-Time Frame Serialization Bottleneck

## Fenst / Swarm+Symph Pipeline

---

## 1. Problem Statement

During 3D point cloud visualization, the system projects point clouds in real-time at 60 FPS. Every frame, the full dataset (`PtsFrameDto`) is serialized from backend to frontend, creating a significant serialization and memory bandwidth bottleneck.

### Identified Bottlenecks:

1. **Full Frame Serialization**: Every 60 FPS frame sends the entire `PtsFrameDto`, including unchanged geometry, 5 static metadata strings, and array headers.
2. **Serde Overhead**: Field renaming attributes (`#[serde(rename = "...")]`) introduce serialization overhead on hot paths.
3. **No Delta Compression**: If only camera angle changes (or nothing changes during idle), all point coordinates are fully re-serialized.
4. **Redundant Metadata**: Strings like `"Swarm CUDA [2 dev] | Rot: (45°, 60°)"` and `"Source: block.pts"` are re-serialized 60 times per second.
5. **Memory Copy Overhead**: Points are repeatedly cloned across pipelines:
   ```
   .pts File -> ParsePtsStream -> PtsCloud { points: [x, y, z]* } -> Buff<[f32; 3]>
   |
   v
   SceneGraph._Points -> ProjectSceneCluster -> Camera Transform (GPU or CPU)
   |
   v
   ProjectedPoint -> Buff<ProjectedPoint> -> PtsFrameDto
   |
   v
   PtsFrameDto._Points -> ToBytes() -> Frame IPC -> Frontend
   ```

---

## 2. Bandwidth Analysis

### Baseline (Current Implementation):
- **100K Points Dataset**:
  - `ProjectedPoint`: 5 `f32` fields (`_X`, `_Y`, `_Radius`, `_CoreRadius`, `_Alpha`) = 20 bytes per point.
  - 100K points * 20 bytes = **2.0 MB per frame**.
- **Frame Rate**: 60 FPS.
- **Bandwidth Consumption**: 2.0 MB * 60 = **120 MB/s continuous throughput**.

---

## 3. Recommended Multi-Phase Optimization Strategy

### 3.1 Phase 1: Direct Binary Struct Serialization (Zero-Copy)
Eliminate JSON / intermediate map serialization for point arrays by adopting contiguous flat memory slices:
```rust
// Contiguous binary frame layout
// [Header: 40 bytes] [Points: N * 20 bytes] [Lines: M * 16 bytes] [Strings metadata]
```

### 3.2 Phase 2: Separate Static & Dynamic Frame Data
Split `PtsFrameDto` into immutable session metadata and streaming delta updates:
```rust
// Static metadata sent once on load
pub struct PtsSessionMetadata {
    pub file_name: String,
    pub total_point_count: u32,
    pub bbox_min: [f32; 3],
    pub bbox_max: [f32; 3],
}

// Dynamic updates streamed per frame
pub struct PtsFrameUpdate {
    pub points_dirty: bool,
    pub points: Option<Buff<ProjectedPoint>>,
    pub camera_state_dirty: bool,
    pub zoom: Option<f32>,
    pub pan_x: Option<f32>,
    pub pan_y: Option<f32>,
    pub rot_x: Option<f32>,
    pub rot_y: Option<f32>,
}
```

### 3.3 Phase 3: Implement Float Quantization
Pack 2D screen coordinates into compact fixed-point integer representations:
- Screen coordinates (`X`, `Y`): 16-bit unsigned ints (`u16`) for 0..4096 viewport resolution.
- `Radius`, `CoreRadius`, `Alpha`: 8-bit unsigned ints (`u8`).
- **Result**: 7 bytes per point instead of 20 bytes (**65% size reduction**).

### 3.4 Phase 4: State Hashing & Change Detection
Compute a fast hash over camera parameters and scene version. If the hash matches the previous frame, return an empty `NO_CHANGE` token (**99% reduction during idle view**).

---

## 4. Optimization Roadmap & Impact Summary

| Phase | Optimization | Estimated Frame Size | 60 FPS Bandwidth | Improvement |
| :--- | :--- | :--- | :--- | :--- |
| **Baseline** | Full DTO JSON / Serde | 2.0 MB | 120 MB/s | Baseline |
| **Phase 1** | Binary flat layout | 2.0 MB | 120 MB/s (Low CPU) | 3x lower CPU |
| **Phase 2** | Static / Dynamic split | 2.0 MB (dirty only) | ~60 MB/s | ~50% savings |
| **Phase 3** | Float quantization (7B) | 700 KB | 42 MB/s | 65% savings |
| **Phase 4** | State hashing (Idle) | ~100 B | ~6 KB/s | >99% savings |

---

## 5. Module References

- **File**: `src/fenst/xplrcmds.rs` - IPC command handlers
- **File**: `src/fenst/mod.rs` - DTO definitions
- **File**: `src/wxfrieze/pts_view.rs` - Presentation viewport
- **Wiki**: [Fenst.md](Fenst.md) - Architecture overview
- **Wiki**: [Swarm.md](Swarm.md) - Compute pipeline

