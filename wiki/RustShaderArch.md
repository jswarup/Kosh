# Rust-GPU Shader Architecture Reference

This developer reference details the internals of the **Rust-GPU** compiler, runtime, and hardware execution model in `Kosh`. It serves as a guide for developers working on the GPU-side shader crate ([compute-shader](file:///c:/Work/Taregna/Kosh/compute-shader/)) or modifying host pipeline execution in [src/swarm/_tests.rs](file:///c:/Work/Taregna/Kosh/src/swarm/_tests.rs).

---

## 1. Compiler Toolchain & Driving Mechanism

Rust-GPU operates as a pluggable backend to the standard Rust compiler (`rustc`). Rather than using LLVM to output CPU machine code, it translates Rust's Mid-level Intermediate Representation (MIR) directly to SPIR-V intermediate representation.

```
                  +-------------------------+
                  |  Rust Source (no_std)   |
                  +------------+------------+
                               |
                               v (rustc)
                  +------------+------------+
                  |    Rust Compiler MIR    |
                  +------------+------------+
                               |
                               v (rustc_codegen_spirv)
                  +------------+------------+
                  |     SPIR-V IR Builder   |
                  +------------+------------+
                               |
                               v
                  +------------+------------+
                  |     SPIR-V Bytecode     |
                  +-------------------------+
```

### The Driver: `spirv-builder`
The host code leverages the `spirv-builder` build script helper crate:
- **Toolchain Channel**: Dynamically resolves and pulls the specific nightly compiler toolchain channel required to match internal compiler APIs used by `rustc_codegen_spirv`.
- **RUSTFLAGS Injection**: Configures compiler flags such as `-Zcodegen-backend=rustc_codegen_spirv` and targets `spirv-unknown-vulkan1.1` under the hood.
- **Cargo Invocation**: Invokes a cargo sub-process targeting the `#![no_std]` shader crate, generating `.spv` binary outputs.

---

## 2. GPU Runtime Constraints (no_std)

Executing Rust directly on GPU hardware requires strict adherence to physical execution models. The code compiled for the SPIR-V target must operate within the following boundaries:

### A. Zero Dynamic Allocation
- **Constraint**: There is no heap, no allocator, and no `alloc` crate.
- **Implementation**: The shader crate uses `#![no_std]`. All data structures must be stack-allocated, compile-time sized, or bound to static arrays.

### B. No Recursion or Dynamic Dispatch
- **Constraint**: The GPU hardware execution pipeline does not support recursive function call stacks or dynamic function routing.
- **Implementation**:
  - Functions are aggressively inlined by the compiler.
  - Trait objects (`dyn Trait`) are compilation errors. Polymorphism must be resolved at compile-time using monomorphized generic arguments or static traits.
  - Recursion leads to static compilation failure.

### C. Stack and Value Optimization
- **Constraint**: Deep call stacks consume expensive register memory space (thread registers).
- **Implementation**: Local structures like `Option<T>` or `Result<T, E>` are compiled down to memory-efficient unions or values. Simple calculations (like Collatz loop steps) should use local primitive registers rather than complex state trees.

---

## 3. SPIR-V Binding & Attribute Semantics

Rust-GPU uses custom attribute annotations (provided by `spirv-std` or the compiler backend) to map Rust syntax to SPIR-V execution configurations:

### A. Entry Point Declaration
```rust
#[spirv(compute(threads(64)))]
pub fn main_cs(...)
```
- **`compute(threads(x, y, z))`**: Declares this function as a compute shader kernel. The host pipeline must invoke it using this exact function name (`"main_cs"`).
- **Workgroup Size**: Sets the number of threads per local workgroup (x=64, y=1, z=1).

### B. Built-in Variables
```rust
#[spirv(global_invocation_id)] id: UVec3
```
- Maps the parameter to Vulkan's `gl_GlobalInvocationID`, providing a 3D index representing the current thread's global position within the compute grid dispatch.

### C. Storage Buffers
```rust
#[spirv(storage_buffer, descriptor_set = 0, binding = 0)] input: &[u32]
```
- **`storage_buffer`**: Maps the Rust reference/slice to a read/write storage buffer.
- **`descriptor_set` and `binding`**: Explicitly maps to descriptor layouts in the host `wgpu` pipeline (`BindGroupLayout`).
- **Read-Only / Read-Write**:
  - `&[T]` translates to a read-only buffer (`var<storage, read>`).
  - `&mut [T]` translates to a read-write buffer (`var<storage, read_write>`).

---

## 4. Host Binding Mechanics (wgpu)

Once compiled to SPIR-V, host integration in `wgpu` bridges raw bytes back into pipelines.

### Raw SPIR-V Loading
WebGPU's WGSL compilation pipeline is bypassed in favor of raw SPIR-V injection:
```rust
let  	spirv = std::borrow::Cow::Owned( wgpu::util::make_spirv_raw( &spirvData).into_owned());
let  	shader = device.create_shader_module( wgpu::ShaderModuleDescriptor {
    label: Some( "rust_gpu_shader"),
    source: wgpu::ShaderSource::SpirV( spirv),
});
```
- **Alignment Warning**: The byte slice loaded from disk must be 4-byte (32-bit word) aligned. `wgpu::util::make_spirv_raw` checks alignment and converts the `&[u8]` slice into a `Cow<[u32]>`.

### Descriptor Layout Alignment
Host-side binding structures must mirror the bindings declared in `lib.rs`:

| Rust-GPU Attribute Declaration | Host-side `wgpu::BindGroupLayoutEntry` Layout |
|:---|:---|
| `descriptor_set = 0, binding = 0` / `&[u32]` | `binding: 0`, `visibility: COMPUTE`, `ty: Buffer { read_only: true }` |
| `descriptor_set = 0, binding = 1` / `&mut [u32]` | `binding: 1`, `visibility: COMPUTE`, `ty: Buffer { read_only: false }` |
