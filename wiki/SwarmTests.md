# Swarm GPU Compute Tests

The `swarm` module contains test demonstrations of GPU compute capabilities using the `wgpu` library and inline WebGPU Shading Language (WGSL) compute shaders. These tests showcase how to orchestrate parallel GPU execution, manage GPU buffer mapping, and perform zero-copy byte casting. All GPU computing dependencies are scoped strictly to `dev-dependencies`, resulting in zero runtime or binary size overhead for the production library.

---

## Core Architecture and Helpers

To interface between CPU-side Rust data structures and GPU-bound WebGPU buffers without pulling in heavy external dependencies like `bytemuck`, the tests utilize specific helper utilities.

### 1. Graceful GPU Initialization (`GpuInit`)
The GPU tests are designed to execute seamlessly in both local developer environments (with hardware acceleration) and headless CI/CD systems (which may lack a GPU adapter). 
```rust
fn	GpuInit() -> Option< ( wgpu::Device, wgpu::Queue)>
```
* **Hardware Selection**: Requests a high-performance adapter via `wgpu::Instance::request_adapter`.
* **Graceful Fallback**: If no compatible adapter is found, `GpuInit` returns `None`. The tests log a skip message and return early rather than panicking.

### 2. Zero-Copy Slice Casting (`ISliceExt`)
WebGPU buffers require raw bytes (`&[u8]`) for data transfer. To achieve zero-cost casting between typed slices (e.g., `&[f32]`, `&[u32]`) and byte slices, the project defines the `ISliceExt` trait in [silo/cast.rs](file:///mnt/c/Work/Taregna/Kosh/src/silo/cast.rs):
```rust
pub trait ISliceExt
{
    fn	CastSlice( &self) -> &[u8];
    fn	CastSliceFrom< U: Copy>( &self) -> &[U];
}
```
* **`CastSlice`**: Reinterprets a typed slice `&[T]` as a raw byte slice `&[u8]`.
* **`CastSliceFrom`**: Reinterprets a raw byte slice `&[u8]` back into `&[U]`, verifying that the input length is properly aligned to the target type size at runtime.

---

## Test Demonstrations

Three GPU tests are implemented in [swarm/_tests.rs](file:///mnt/c/Work/Taregna/Kosh/src/swarm/_tests.rs), showcasing progressive complexity.

### 1. Double Values (`TestGpuDoubleValues`)
Demonstrates the "Hello World" of GPU compute: uploading a single buffer, running a shader that modifies it in place, and reading it back.
* **CPU Data**: Built using `Buff::Create` with 256 `f32` elements initialized to `1.0..=256.0`.
* **WGSL Shader**:
  ```wgsl
  @group(0) @binding(0)
  var<storage, read_write> data: array<f32>;

  @compute @workgroup_size(64)
  fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
      let idx = gid.x;
      if idx < arrayLength(&data) {
          data[idx] = data[idx] * 2.0;
      }
  }
  ```

### 2. Vector Addition (`TestGpuVectorAdd`)
Demonstrates binding multiple buffers in a single bind group (two read-only input buffers and one read-write output buffer).
* **CPU Data**: Two input buffers (`buffA` and `buffB`) of 512 `f32` elements.
* **WGSL Shader**:
  ```wgsl
  @group(0) @binding(0) var<storage, read> a: array<f32>;
  @group(0) @binding(1) var<storage, read> b: array<f32>;
  @group(0) @binding(2) var<storage, read_write> result: array<f32>;

  @compute @workgroup_size(64)
  fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
      let idx = gid.x;
      if idx < arrayLength(&result) {
          result[idx] = a[idx] + b[idx];
      }
  }
  ```

### 3. Collatz Conjecture Steps (`TestGpuCollatz`)
Demonstrates integer arithmetic and branching loops inside a compute shader.
* **Algorithm**: For a given positive integer $n$, computes the number of steps to reach 1:
  - If $n$ is even: $n \to n / 2$
  - If $n$ is odd: $n \to 3n + 1$
* **CPU Data**: Input sequence `1..=128`.
* **WGSL Shader**:
  ```wgsl
  @group(0) @binding(0) var<storage, read> input: array<u32>;
  @group(0) @binding(1) var<storage, read_write> output: array<u32>;

  fn collatz_steps(n_in: u32) -> u32 {
      var n: u32 = n_in;
      var steps: u32 = 0u;
      while n != 1u {
          if (n % 2u) == 0u {
              n = n / 2u;
          } else {
              n = 3u * n + 1u;
          }
          steps = steps + 1u;
      }
      return steps;
  }

  @compute @workgroup_size(64)
  fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
      let idx = gid.x;
      if idx < arrayLength(&output) {
          output[idx] = collatz_steps(input[idx]);
      }
  }
  ```

---

## How to Run the Tests

To run the GPU compute tests, run:
```bash
cargo test -- --test-threads=1 swarm
```

* `--test-threads=1` is recommended to prevent multiple tests from requesting GPU resources concurrently.
* If no GPU or software Vulkan renderer is found, the console output will indicate that tests were skipped successfully.
