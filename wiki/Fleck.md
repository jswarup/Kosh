# Module Reference: `fleck`

## 1. Overview & Purpose

The `fleck` module provides high-throughput **3D geometric representations, vector arithmetic, and geometry file I/O**. It encompasses:
1. **Multidimensional Vector Math (`Vex<N, T>`)**: Generic N-dimensional vector algebra with compile-time dimension checking, vector space operations, scalar multiplication, inner products, normalization, and coordinate projections.
2. **Specialized 3D & 2D Types (`Pt3f`, `WPt3f`, `WPt2f`, `Dir3f`, `Point32`, `RGB`)**: High-performance representations for Euclidean positions, homogeneous coordinates, direction vectors, and colors.
3. **3D Point Cloud Processing (`PtsCloud`, `PtsPoint`, `PtsShard`)**: Parsing, bounding box calculation, intensity mapping, and streaming parsing for `.pts` point cloud files.
4. **Wavefront Mesh Processing (`WaveObjModel`, `WaveObjFace`, `WaveObjMaterial`, `WaveObjShard`, `WaveObjParserCtx`)**: Streaming parser for 3D polygonal `.obj` meshes with support for negative indexing, normals, texture coordinates, and DTO conversion.

---

## 2. Architecture & Class Diagram

```mermaid
classDiagram
    class Vex~N, T~ {
        -_Data: [T; N]
        +New(data) Vex
        +Dot(other) T
        +NormSq() T
        +Norm() f32
        +Normalize() Vex
        +Cross(other) Vex
    }

    class Pt3f {
        +f32 _X
        +f32 _Y
        +f32 _Z
    }

    class PtsPoint {
        +Point32 _Coord
        +f32 _Intensity
        +RGB _Color
        +New(coord, intensity, color) PtsPoint
    }

    class PtsCloud {
        +Buff~PtsPoint~ _Points
        +Point32 _BBoxMin
        +Point32 _BBoxMax
        +New() PtsCloud
        +Push(point)
        +Points() Arr~PtsPoint~
        +ToDto() PtsPointsDto
    }

    class WaveObjModel {
        +Buff~WPt3f~ _Vertices
        +Buff~Dir3f~ _Normals
        +Buff~WPt2f~ _TexCoords
        +Buff~Face~ _Faces
        +New() WaveObjModel
        +ToDto() WaveObjMeshDto
        +Triangulate() Buff~[FaceVertex; 3]~
    }

    class Face {
        +Buff~FaceVertex~ _Vertices
        +Len() usize
    }

    class FaceVertex {
        +U32 _VertexIdx
        +Option~U32~ _TexCoordIdx
        +Option~U32~ _NormalIdx
    }


    class WaveObjParserCtx {
        +Stash~WPt3f~ _VStash
        +Stash~Face~ _FStash
        +Stash~FaceVertex~ _FaceVerts
        +New(streamSz) WaveObjParserCtx
    }

    class WaveObjParserCtxMM {
        +Get() &mut WaveObjParserCtx
        +PushVal(arr) bool
        +EndV() bool
        +EndFace() bool
    }
    WaveObjParserCtxMM --> WaveObjParserCtx : mutates

    PtsPoint *-- Point32 : coordinates
    PtsCloud o-- PtsPoint : stores in Buff
    WaveObjModel o-- Face : stores in Buff
    Face *-- FaceVertex : coordinates
```

---

## 3. Geometric Subsystems

### 3.1 Vector Space & Arithmetic Engine (`Vex`)
The `fleck::vex` module implements formal mathematical vector spaces over arbitrary dimensions:
- **Scalar Operations**: Add, subtract, multiply, divide across scalar values.
- **Inner Product & Norms**: Euclidean dot product (`Dot`), squared norm (`NormSq`), L2 norm (`Norm`), and unit normalization (`Normalize`).
- **3D Specialization**: Cross product (`Cross`) for 3-element vectors.
- **Type Aliases**:
  - `Pt3f`: 3D Cartesian position vector `(x, y, z)`.
  - `WPt3f`: 3D Homogeneous coordinate vector `(x, y, z, w)`.
  - `WPt2f`: 2D Homogeneous coordinate vector `(u, v, w)`.
  - `Dir3f`: 3D Direction vector with normalized ray semantics.

### 3.2 Point Cloud Parser (`PtsCloud`, `PtsShard`)
Zero-allocation streaming parser for `.pts` point cloud data:
- Streaming via `IStream` and `ShardTree!`.
- Extracts 3D coordinates, radar/lidar intensity floats, and 24-bit RGB color tuples.
- Dynamically maintains axis-aligned minimum and maximum bounding boxes (`_BBoxMin`, `_BBoxMax`).

### 3.3 Wavefront OBJ Parser (`WaveObjModel`, `WaveObjShard`)
Grammar-based parser for polygonal 3D models:
- **Geometry Tokens**: Geometric vertices (`v`), vertex normals (`vn`), texture coordinates (`vt`), parameter space vertices (`vp`).
- **Polygonal Faces (`f`)**: Supports variable-valence faces (triangles, quads, polygons), `v/vt/vn` vertex triplets, and relative/negative indexing.
- **Materials & Groups**: Handles group identifiers (`g`), smoothing groups (`s`), and material library bindings (`usemtl`, `mtllib`).
- **Triangulation**: Convex fan triangulation converts arbitrary N-sided polygons into GPU-ready triangle lists.
- **Zero-Overhead Context (WaveObjParserCtx)**: Uses a tightly packed WaveObjParserCtx struct to manage dynamic Stash allocations. A lightweight WaveObjParserCtxMM copy-wrapper provides interior mutability for the ShardTree! action blocks without requiring heavy refcells or boxing.

---

## 4. Free Functions Reference

### Point Cloud I/O (`ptsio.rs`)
- **`ParsePts(path: &str) -> Result<PtsCloud, String>`**: Parses `.pts` file from disk using `BuffStream`.
- **`ParsePtsBytes(bytes: &[u8]) -> Result<PtsCloud, String>`**: Parses in-memory byte slice using `FixedStream`.
- **`ParsePtsStream(stream: &mut dyn IStream) -> Result<PtsCloud, String>`**: Parses stream implementing `IStream`.

### Wavefront OBJ I/O (`waveobjio.rs`)
- **`ParseWaveObj(input: &str) -> Result<WaveObjModel, String>`**: Parses `.obj` file from disk using `BuffStream`.
- **`ParseWaveObjBytes(bytes: &[u8]) -> Result<WaveObjModel, String>`**: Parses in-memory byte slice using `FixedStream`.
- **`ParseWaveObjStream(stream: &mut dyn IStream) -> Result<WaveObjModel, String>`**: Parses stream implementing `IStream`.

