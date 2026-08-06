# Rust-GPU Architecture

The Kosh project integrates **Rust-GPU** ([`rust-gpu`](https://github.com/Rust-GPU/rust-gpu)) to compile native Rust code directly into SPIR-V compute shaders. This allows GPU compute kernels to be written in idiomatic, strongly-typed Rust rather than GLSL or HLSL, sharing numeric algorithms and data types between host CPU code and GPU device kernels.

---

## 1. Toolchain & Compilation Pipeline

Rust-GPU shaders are compiled into SPIR-V binaries at build time using custom compiler backends and target specifications.

```
                   +-------------------------+
                   |  Shader Source (gcomp)  |
                   |      #![no_std]         |
                   +------------+------------+
                                |
                                v (spirv-builder)
                   +------------+------------+
                   |  rustc_codegen_spirv    |
                   | Target: vulkan1.1       |
                   +------------+------------+
                                |
                                v
                   +------------+------------+
                   |    SPIR-V Bytecode      |
                   |    (.spv output)        |
                   +------------+------------+
                                |
                                v (include_bytes!)
                   +------------+------------+
                   |   Host Executable/App   |
                   +-------------------------+
```

### A. The Build Script (`build.rs`)

The desktop application ([src/fenst-app/build.rs](../src/fenst-app/build.rs)) leverages `spirv_builder` to orchestrate shader compilation:

```rust
let compileResult = spirv_builder::SpirvBuilder::new( "../../src/gcomp", "spirv-unknown-vulkan1.1")
    .build()
    .expect( "Failed to compile gcomp SPIR-V shader");
```

- **Target Architecture**: `spirv-unknown-vulkan1.1`.
- **Compiler Backend**: Injects `-Zcodegen-backend=rustc_codegen_spirv` into the compiler invocation.
- **Environment Emission**: Emits `cargo::rustc-env=GCOMP_SPV_PATH=<path_to_spv>` so the compiled bytecode path is passed to the host crate at compile time.

### B. Static Bytecode Embedding

The host code embeds the generated SPIR-V module directly into the binary's data segment using a single module-level static constant:

```rust
static GCOMP_SPV: &[u8] = include_bytes!( env!( "GCOMP_SPV_PATH"));
```

---

## 2. Shader Crate Architecture (`gcomp`)

The `gcomp` crate ([src/gcomp/src/lib.rs](../src/gcomp/src/lib.rs)) contains the GPU compute kernels. It is configured for dual-compilation: it compiles as `#![no_std]` for the `spirv` target and as a standard Rust library for unit testing.

### A. Attributes & Imports

```rust
#![cfg_attr(target_arch = "spirv", no_std)]
use spirv_std::{glam::UVec3, spirv};
```

### B. Compute Kernel Entry Point (`pts_pointcloud_cs`)

The point cloud generator kernel runs across 64-thread workgroups:

```rust
#[spirv(compute(threads(64)))]
pub fn pts_pointcloud_cs(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] output: &mut [spirv_std::glam::Vec4],
) {
    let idx = id.x as usize;
    if idx >= output.len() {
        return;
    }

    let hx = wang_hash( idx as u32 * 3 + 0);
    let hy = wang_hash( idx as u32 * 3 + 1);
    let hz = wang_hash( idx as u32 * 3 + 2);

    let x = hash_to_float( hx) * 40.0 - 20.0;
    let y = hash_to_float( hy) * 40.0 - 20.0;
    let z = hash_to_float( hz) * 40.0 - 20.0;

    output[idx] = spirv_std::glam::Vec4::new( x, y, z, 1.0);
}
```

#### Kernel Mechanics
- **`#[spirv(global_invocation_id)]`**: Injects the global 3D thread index. `id.x` represents the 1D offset of the current point.
- **`#[spirv(storage_buffer)]`**: Binds descriptor set 0, binding 0 as a read/write storage buffer containing 16-byte `Vec4` structures.
- **Wang Hash PRNG**: Generates pseudo-random deterministic numbers on the GPU:
  ```rust
  fn wang_hash(mut seed: u32) -> u32 {
      seed = (seed ^ 61) ^ (seed >> 16);
      seed = seed.wrapping_mul(9);
      seed = seed ^ (seed >> 4);
      seed = seed.wrapping_mul(0x27d4eb2d);
      seed = seed ^ (seed >> 15);
      seed
  }
  ```
- **Float Normalization**: Maps lower 24 bits of hash to floating-point coordinates in `[-20.0, 20.0]³`.

---

## 3. Host GPU Operator Trait (`IGpuOp`)

Kosh standardizes WebGPU / `wgpu` hardware interaction via the `IGpuOp` trait ([src/swarm/gpusop.rs](../src/swarm/gpusop.rs)), implemented directly on `wgpu::Device`.

```rust
pub trait IGpuOp {
    fn Init() -> Option<(Device, Queue)>;
    fn BufferInit(&self, label: &str, data: &[u8], usage: BufferUsages) -> Buffer;
    fn ReadBuffer(&self, queue: &Queue, buf: &Buffer, size: u64) -> Vec<u8>;
}
```

### A. Device Initialization (`Init`)
Asynchronously queries WebGPU hardware instances, selecting a high-performance adapter (`PowerPreference::HighPerformance`) via `pollster::block_on`.

### B. Storage Buffer Creation (`BufferInit`)
Encapsulates `create_buffer_init` to instantiate GPU storage buffers prepopulated with initial byte arrays.

### C. Staging Buffer Readback (`ReadBuffer`)
Reads storage buffer contents back to CPU memory synchronously:
1. Allocates a temporary staging buffer with `BufferUsages::MAP_READ | BufferUsages::COPY_DST`.
2. Encodes a GPU command (`copy_buffer_to_buffer`) to copy device data to staging memory.
3. Submits the command buffer to the `wgpu::Queue`.
4. Invokes `slice.map_async(MapMode::Read, ...)` and polls the device until readback completes.
5. Returns a unmapped, owned `Vec<u8>` CPU slice.

---

## 4. Execution & Pipeline Dispatch (`XplrFetchPtsPoints`)

The host library function `XplrFetchPtsPoints` ([src/fenst/mod.rs](../src/fenst/mod.rs)) coordinates the complete GPU execution lifecycle:

```
 [Host CPU]                              [GPU Hardware]
     |                                          |
     |--- 1. Initialize Device & Queue -------->|
     |--- 2. Create Storage Buffer (1600 B) --->|
     |--- 3. Load SPIR-V Module --------------->|
     |--- 4. Bind Pipeline & Storage Group ---->|
     |--- 5. Dispatch Workgroups (ceil(100/64)->|-- Execute pts_pointcloud_cs
     |--- 6. Submit Encoder Command Queue ----->|
     |                                          |
     |<-- 7. Copy & Staging Buffer Readback ----|
     |
 [Extract x, y, z -> Return PtsPointsDto]
```

### Dispatch Calculation
For 100 3D points, workgroup dispatch counts are computed dynamically:
$$\text{workgroups} = \left\lceil \frac{\text{numPoints}}{64} \right\rceil = \left\lceil \frac{100}{64} \right\rceil = 2$$

Each `Vec4(x, y, z, 1.0)` is mapped into 3D point tuples `[f32; 3]`, bounding box limits are assigned (`[-20, -20, -20]` to `[20, 20, 20]`), and returned as a `PtsPointsDto`.

---

## 5. GPU Execution Constraints (`#![no_std]`)

Writing shaders in Rust-GPU requires adhering to strict execution models:

1. **Zero Dynamic Allocation**: Shaders execute without a heap or `alloc` crate. Memory must be stack-allocated, passed via storage buffers, or statically sized.
2. **Alignment & Padding**: `glam::Vec4` vectors require 16-byte alignment matching SPIR-V std430 layout specifications.
3. **No Unbounded Loops**: Loops must have statically determinable boundaries or explicit loop caps to satisfy SPIR-V verification rules.
