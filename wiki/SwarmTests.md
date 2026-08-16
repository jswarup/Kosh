# Swarm Compute & Hardware Abstraction Tests

The `swarm` module provides a unified compute abstraction layer across **CPU** (multi-threaded SIMT emulation), **Rust-GPU / WebGPU** (SPIR-V bytecode & WGSL shaders via `wgpu`), and **Cuda-Oxide** (NVIDIA CUDA / PTX compute kernels).

---

## 1. Core Architecture & Hardware Abstraction

```
                     +-----------------------------------+
                     |           SwarmEngine             |
                     | (Auto-selection & Unified Facade) |
                     +-----------------+-----------------+
                                       |
          +----------------------------+----------------------------+
          |                            |                            |
          v                            v                            v
+-------------------+        +--------------------+       +--------------------+
|     CpuDevice     |        |   RustGpuDevice    |       |  CudaOxideDevice   |
| (Multi-thread CPU)|        |  (wgpu / SPIR-V)   |       |   (CUDA / PTX)     |
+-------------------+        +--------------------+       +--------------------+
          |                            |                            |
          v                            v                            v
+-------------------+        +--------------------+       +--------------------+
|     CpuBuffer     |        |   RustGpuBuffer    |       |  CudaOxideBuffer   |
| (Host Memory/Buff)|        |   (wgpu::Buffer)   |       | (CUDA Dev Ptr/Sim) |
+-------------------+        +--------------------+       +--------------------+
```

### Key Components

1. **`BackendKind`**: Enum specifying execution target (`BackendKind::Cpu`, `BackendKind::RustGpu`, `BackendKind::CudaOxide`).
2. **`SwarmEngine`**: High-level execution engine featuring:
   - `SwarmEngine::Auto()`: Automatically discovers and selects the best available compute hardware (CudaOxide -> RustGpu -> Cpu fallback).
   - `SwarmEngine::New(BackendKind)`: Explicit backend binding and dynamic switching.
   - Standardized runners: `RunDouble`, `RunVectorAdd`, `RunCollatz`, `RunPointCloud`.
3. **Enum-Dispatched Device Wrappers**:
   - `SwarmDevice`, `SwarmBuffer`, `SwarmKernel` eliminate virtual dispatch and `Box<dyn ...>` overhead on hot execution paths.
4. **Zero-Copy Slice Casting (`ISliceExt`)**:
   - Postfix typed slice casting methods in [silo/cast.rs](../src/silo/cast.rs): `CastSlice(&self) -> &[u8]`, `CastSliceFrom<U: Copy>(&self) -> &[U]`, and `CastSliceMut<U: Copy>(&mut self) -> &mut [U]`.
5. **Backward Compatibility**:
   - Full backward compatibility for `IGpuOp` trait on `wgpu::Device` in [gpusop.rs](../src/swarm/gpusop.rs).

---

## 2. Test Demonstrations

The test suite in [swarm/_tests.rs](../src/swarm/_tests.rs) comprises 11 comprehensive tests:

### A. CPU SIMT Backend Tests
* **`TestCpuDoubleValues`**: Verifies in-place element doubling over host memory buffers using SIMT indexing.
* **`TestCpuVectorAdd`**: Verifies 2-input/1-output vector addition on CPU.
* **`TestCpuCollatz`**: Verifies integer branching and loop convergence across CPU threads.
* **`TestCpuPointCloud`**: Verifies Wang Hash PRNG 3D point cloud generation in `[-20.0, 20.0]³`.

### B. Rust-GPU / WebGPU Backend Tests
* **`TestGpuDoubleValues`**: WGSL compute shader execution for in-place buffer modification.
* **`TestGpuVectorAdd`**: WGSL vector addition with multiple storage buffer bindings.
* **`TestGpuCollatz`**: WGSL integer arithmetic and iterative Collatz sequence computation.
* **`TestRustGpuComputeExample`**: Build-time compilation of the `gcomp` Rust-GPU crate to SPIR-V and device execution.

### C. Cuda-Oxide (CUDA / PTX) Backend Tests
* **`TestCudaOxidePtxExecution`**: Compiles and launches PTX assembly compute kernels.

### D. Unified SwarmEngine & Backend Switching Tests
* **`TestSwarmEngineBackendSwitching`**: Executes identical compute workloads (`Double`, `VectorAdd`, `Collatz`, `PointCloud`) across CPU, Cuda-Oxide, and Rust-GPU backends and verifies that all backends produce equivalent results.
* **`TestSwarmEngineAuto`**: Verifies hardware auto-detection and execution fallback.

---

## 3. How to Run the Tests

To run all swarm compute tests:
```bash
cargo test -- --test-threads=1 swarm
```

To run the entire workspace test suite:
```bash
cargo test --workspace
```
