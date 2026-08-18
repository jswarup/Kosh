# Kosh Platform Architecture Overview

## 1. System Philosophy & Objectives

The **Kosh** platform is an ultra-high-performance computational, symbolic, grammar-parsing, and workflow-orchestration runtime written in Rust. It is engineered with three uncompromising architectural goals:

1. **Zero Heap Allocation in Recursive AST Structures**: Elimination of fine-grained heap allocations (`Box<DynINode>`) for parser grammars, expression trees, and chore pipelines via macro-driven inline stack lifetime extension.
2. **Explicit Memory Control & Cache Locality**: Replaces standard dynamic collections with contiguous buffers (`Buff<T>`), zero-copy slice wrappers (`Arr<'a, T>`), lock-free atomic stacks (`Stk<'a, 'b, T>`), and contiguous interval segments (`USeg`).
3. **Cross-Platform Heterogeneous SIMT Computing**: Unifies compute dispatch across multithreaded CPU SIMT, WebGPU / Vulkan (Rust-GPU SPIR-V), and CUDA / PTX (Cuda-Oxide) with zero-cost data representations.

---

## 2. Layered Architecture

The Kosh codebase is structured into four distinct functional tiers:

```mermaid
graph TD
    subgraph Layer4 ["Tier 4: Presentation & Applications"]
        Frieze["frieze<br/>Desktop GUI App (Tauri v2, Multi-window, 3D Canvas)"]
        Fenst["fenst<br/>Virtual File Explorer & Data Provider Hub"]
    end

    subgraph Layer3 ["Tier 3: Domain Subsystems"]
        Fleck["fleck<br/>Point Cloud (.pts) Parser & Spatial Bounding"]
        Fresco["fresco<br/>Symbolic Mathematics & Expression Repositories"]
        Heist["heist<br/>Work-Stealing Chore DAG Orchestrator"]
    end

    subgraph Layer2 ["Tier 2: Compute & Core Engines"]
        Swarm["swarm<br/>Heterogeneous GPU/CPU Compute Device Engine"]
        Symph["symph<br/>no_std SPIR-V / SIMT Compute Kernels"]
        Shard["shard<br/>Zero-Box Recursive Descent Grammar Engine"]
        Flux["flux<br/>Stream Buffering & Reflection Serialization"]
    end

    subgraph Layer1 ["Tier 1: Primitives & Foundations"]
        Stalks["stalks<br/>AST Nodes (UniNode, BinNode), Atomics, Work Queue"]
        Silo["silo<br/>Memory Buffers, Slices, Custom Unsigned Numerics"]
    end

    Frieze --> Fenst
    Frieze --> Symph
    Fenst --> Shard
    Fenst --> Fresco
    Fenst --> Flux
    Fenst --> Swarm
    Fenst --> Silo

    Fleck --> Shard
    Fleck --> Flux
    Fleck --> Silo
    Fleck --> Fenst

    Fresco --> Flux
    Fresco --> Stalks
    Fresco --> Silo

    Heist --> Swarm
    Heist --> Stalks
    Heist --> Silo

    Swarm --> Symph
    Swarm --> Silo

    Shard --> Stalks
    Shard --> Flux
    Shard --> Silo

    Flux --> Silo
    Stalks --> Silo
```

---

## 3. Subsystem Breakdown

### Tier 1: Foundation
- **[`silo`](Silo.md)**: Defines project-wide numeric wrappers (`U8`, `U16`, `U32`, `U64`) that wrap primitive types with wrapping arithmetic, atomic bridges, and explicit conversions. Provides `Buff<T>` (owned slice buffer), `Arr<'a, T>` (borrowed slice wrapper), `Stk<'a, 'b, T>` (lock-free CAS stack backed by an array), `Stash<T>` (growable buffer stack), and `USeg` (half-open/closed unsigned index intervals).
- **[`stalks`](Stalks.md)**: Universal AST tree nodes (`UniNode<C, Op>`, `BinNode<L, R, Op>`, `BinOp`), atomic synchronization (`Atm<T>`, `Spinlock`), type-erased work pointers (`WorkPtr<'a>`, `IWork`), and worker scheduling interfaces (`IWorker`, `DynIWorker`). Defines the core recursive macro engine `NodeTree!`.

### Tier 2: Core Engines & Compute
- **[`shard`](Shard.md)**: High-performance recursive-descent grammar combinator parsing engine. Built on `IGrammar` and `Parser<'p>`, it parses binary/text inputs with `ShardTree!` DSL without heap allocations. Features a 256-bit bitset character filter (`Charset`), numeric parsers (`UInt`, `Int`, `Hex`, `Real`), repetition/action combinators, and a streaming `JSon` parser.
- **[`flux`](Flux.md)**: Universal streaming and data conversion layer. Provides input streams (`IStream`, `FixedStream`, `BuffStream`), output streams (`OutStream`, `JsonOutStream`), and a reflection-style visitor import/export framework (`IFluxExportSource`, `IFluxImportSource`, `FieldExp`, `FieldImp`, `ImplFluxSource!`).
- **[`swarm`](Swarm.md)**: Hardware compute engine abstracting CPU SIMT thread pools, Rust-GPU (`wgpu` executing compiled SPIR-V bytecode), and Cuda-Oxide (CUDA driver / PTX execution). Unified through `SwarmEngine`, `SwarmDevice`, `SwarmBuffer`, and `SwarmKernel`.
- **[`symph`](Swarm.md)**: `#![no_std]` portable compute library containing Wang Hash PRNG, Collatz sequence step calculators, element doubling, vector addition, and 3D point cloud generation. Compiles to standard host CPU functions and SPIR-V compute shaders (`pts_pointcloud_cs`, `camera_transform_cs`, `frustum_cull_cs`, `scene_vs`, `scene_fs`).

### Tier 3: Domain Subsystems
- **[`heist`](Heist.md)**: Asynchronous Directed Acyclic Graph (DAG) chore orchestration system. Features `Atelier` (master workspace managing thread execution and job tracking) and `Maestro` (thread worker executing jobs and stealing work). Uses `ChoreTree!` to construct and schedule complex dependency graphs across CPU and GPU backends.
- **[`fresco`](Fresco.md)**: Symbolic algebra engine. Models mathematical terms (`ITermNode`, `Term`, `TermTree!`) and compiles them into flat indices within `ExprRepos`. Implements polynomial expressions (`PolyExpr`, `SumExpr`, `ProdExpr`, `PowExpr`), real constants (`RealExpr`), and variables (`VarExpr`, `VarAttrib`).
- **[`fleck`](Fleck.md)**: High-throughput 3D point cloud processing. Parses ASCII and binary `.pts` files into `PtsCloud` using custom `Shard` grammars (`PtsShard`), computing bounding boxes, intensity, and RGB color channels.

### Tier 4: Applications & Desktop Explorer
- **[`fenst`](Fenst.md)**: Extensible virtual explorer framework. Defines unified tree interfaces (`Xplr`, `LeafXplr`, `BranchXplr`) and pluggable provider registries (`XplrRegistry`) supporting filesystem (`FsProvider`), AST grammars (`ShardProvider`), and symbolic algebra (`FrescoProvider`).
- **[`frieze`](Fenst.md)**: Embedded desktop GUI subsystem built on Tauri v2. Merged directly into the root `kosh` binary, it renders interactive 3D point clouds, provides multi-window document views, and handles IPC commands (`xplrcmds`). Launching `cargo run` opens Frieze by default, while `--test` dispatches test execution.

---

## 4. Key Architectural Patterns

### Pattern A: Stack-Allocated AST Lifetime Extension
Rather than wrapping child nodes in `Box<dyn INode>`, AST nodes are declared as concrete generic structs (`UniNode<C, Op>`, `BinNode<L, R, Op>`) via declarative macros (`NodeTree!`, `ShardTree!`, `TermTree!`, `ChoreTree!`).
Because expressions evaluate directly in the caller's stack frame, Rust's temporary lifetime extension preserves all child node references without incurring allocator traffic.

```
Caller Stack Frame
┌─────────────────────────────────────────────────────────────┐
│ BinNode {                                                   │
│   _Left: UniNode { _Child: Charset, _Op: USeg },           │
│   _Right: BinNode { _Left: Str, _Right: Real, _Op: Sum },  │
│   _Op: Bor                                                  │
│ }                                                           │
└─────────────────────────────────────────────────────────────┘
  ▲ Zero heap allocation, fully inlined, L1-cache friendly
```

### Pattern B: Lock-Free Stack Exchange
Inter-thread coordination in `heist` and memory reclamation in `silo` use atomic Compare-And-Swap (CAS) loops:
- `Stk` coordinates push/pop/import/export operations between private thread caches and global stashes via atomic index adjustments (`_Size: Atm<U32>`).
- If an allocation cache runs dry, workers export or import batches of indices with a single lock-free atomic CAS.

### Pattern C: Unified Visitor Flux Serialization
Instead of large proc-macro serialization dependencies, `flux` implements an efficient visitor pattern:
- Exporting traverses data via `FetchFieldExp` yielding `FieldExp::Obj` or `FieldExp::Arr`.
- Importing dynamically binds values into destination memory via `FetchFieldImp` and `FieldImp`.
- Macros (`ImplFluxSource!`, `ImplFluxSourceTyped!`) generate boilerplate-free import/export implementations for arbitrary structs.
