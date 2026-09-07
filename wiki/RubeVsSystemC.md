# Rube vs. SystemC: Comparative Analysis

This document compares Kosh's `rube` simulation engine against SystemC across architecture, functionality, and performance.

---

## Architecture

| Aspect | Rube | SystemC |
|---|---|---|
| **Language/paradigm** | Rust, const-generics + type-state (`Module<IN,OUT,SUBS,State>`), compile-time sealing (`Construction`→`Sealed`) | C++ class library, virtual `sc_module`/`sc_interface` inheritance, runtime polymorphism |
| **Simulation model** | Synchronous 3-phase temporal model (Past/Current/Future) with `AdvanceAll()`; no iterative convergence within a phase — O(#modules), single-pass | Discrete-event simulation with **delta cycles**: processes (`SC_METHOD`/`SC_THREAD`) re-evaluate until signals stabilize each timestep, supporting arbitrary combinational feedback and multiple delays |
| **Scheduling granularity** | Compile-time-sorted "warps" (`FastWarp`/`CustomWarp`) grouping identical `KernelOp`s for SIMT-style batch dispatch | Runtime event queue + sensitivity lists; scheduler wakes processes based on dynamic event notifications (`sc_event`) |
| **Dispatch mechanism** | Enum + `match` (`KernelKind::Fast/Behavioral/Custom/Coro`) → static dispatch, no vtables in hot path | Virtual function calls (`process_call_back`), heavier per-process overhead |
| **Memory layout** | Structure-of-Arrays (`TriggerWad._PastVals/_CurrentVals/_FutureVals`), 64-lane bitset predication, zero heap allocation in `Drive()` | Object-oriented, each module/signal is a heap-allocated C++ object; AoS layout, no cache-locality guarantees |
| **Concurrency model** | Coroutine kernels (`CoroKernelFactory`) for explicit stateful FSMs; potential Rayon parallel warps (not yet default) | `SC_THREAD` processes use cooperative coroutines (via `ucontext`/fibers) — conceptually similar but per-object, not batched |
| **Timing model** | Currently cycle/edge-based (posedge/negedge via `IsEdge`); no native sub-cycle delta-delay chains yet | Full delta-cycle + real-time delay semantics (`wait(10, SC_NS)`), required for accurate RTL/TLM timing |

**Key architectural difference**: SystemC is built for *generality* — arbitrary event-driven concurrency, multiple abstraction levels (RTL, TLM), delays — at the cost of per-object virtual dispatch and heap-heavy `sc_signal`/`sc_module` objects. Rube trades that generality for a **restricted, statically-analyzable synchronous model** that can be compiled into flat, batched, branch-predictable warps.

---

## Functionality

| Feature | Rube | SystemC |
|---|---|---|
| **Abstraction levels** | RTL-only currently (gates, latches, adders, FIFOs) | RTL + TLM-2.0 (transaction-level modeling), loosely/approximately-timed abstractions |
| **4-state logic** | Yes, IEEE-1364-style 0/1/X via bit-packed `Reg` (`_Val`/`_X`) | Yes, via `sc_logic`/`sc_lv` (4-state logic vectors) |
| **VCD waveform export** | Yes (`vcd`, `vcdio` — zero-heap parser/writer) | Yes (`sc_trace`), mature and widely tooled |
| **HDL interop (DPI/VPI)** | Planned (Phase 4 of EDA roadmap), not yet implemented | Mature — SystemC has IEEE 1666 standardization, VPI/DPI bridges, and is the basis of many commercial co-simulation flows |
| **Verification ecosystem** | None yet (no UVM equivalent) | UVM built on SystemC/SystemVerilog is the industry-standard verification methodology |
| **Introspection/reflection** | In progress (`IModuleInterface`, SV/VHDL export) — Phase 1 | `sc_object` hierarchy provides runtime name/kind introspection, plus mature debugger/GUI support (e.g. in Questa, VCS) |
| **Simulation control** | In progress (`ISimulationController`: pause/resume/breakpoints) — Phase 3 | Native support via scheduler control APIs, well-integrated with commercial debuggers |
| **Ecosystem/tooling maturity** | New, single-project, no external tool support | 20+ years, IEEE 1666 standard, supported by every major EDA vendor (Synopsys, Cadence, Siemens) |

Functionally, SystemC is a **superset** today — it supports TLM, arbitrary timing, delta-cycle semantics, and has a full verification/tooling ecosystem. Rube's EDA roadmap (Phases 0–5, see [Rube.md](Rube.md#4-eda-compatibility-roadmap)) is explicitly working toward closing this gap (SV/VHDL export, DPI/VPI, package management) but most of it is still "in development" or "pending."

---

## Performance

This is where Rube is designed to significantly outperform SystemC for its target domain (large synchronous gate/RTL networks):

- **Dispatch overhead**: SystemC pays virtual-call + event-queue overhead per process invocation; Rube uses static `match` dispatch inside pre-sorted warps, eliminating vtable indirection for the fast/primitive path.
- **Cache behavior**: Rube's SoA `TriggerWad` gives ~99% L1 hit rates for sequential trigger scans; SystemC's per-object `sc_signal`s are heap-scattered, causing much more pointer-chasing.
- **Allocation**: Rube's `Drive()` is zero-heap-allocation; SystemC's delta-cycle event scheduling frequently allocates/deallocates event and process-handle bookkeeping at runtime.
- **Batch execution**: Rube's 64-lane word predication skips 64 inactive gates per instruction; SystemC's sensitivity-list model evaluates processes individually as events fire, without SIMD-style batching.
- **Claimed throughput**: Rube targets ~100M+ gate ops/sec on modern CPUs (see [Rube.md §7](Rube.md#7-performance-characteristics)) with near-O(1) per-cycle cost independent of circuit size for homogeneous circuits — SystemC's delta-cycle event overhead generally scales worse for gate-level netlists (it's optimized more for TLM-level abstraction where fewer, coarser-grained events dominate).
- **Trade-off**: SystemC's flexibility (arbitrary delays, feedback loops, TLM) requires the more general (and slower) event-driven scheduler; Rube's speed comes from restricting to a synchronous, feedback-free-per-phase model that can be resolved in one linear pass.

---

## Parallel Execution: Multi-Core/GPU (`heist` + `swarm`/`symph`) vs. SystemC

SystemC's reference kernel is fundamentally **single-threaded**: the scheduler runs one process at a time on one core, and although some commercial simulators (e.g. Questa, Xcelium) offer proprietary multi-threaded/parallel kernels, the open IEEE 1666 SystemC kernel itself has no standardized multi-core or GPU execution model — dependency management via arbitrary delta-cycle events makes safe automatic parallelization hard.

Rube, in contrast, has two dedicated subsystems built specifically to parallelize the synchronous warp-based model across cores and GPUs:

### `heist` — work-stealing CPU parallelism
- **Atelier/Maestro model**: a global work-stealing scheduler (`Atelier`) coordinates per-thread workers (`Maestro`), each with a local run queue; idle workers steal from random victims (Knuth-hash-randomized) — no central lock contention.
- **ChoreTree DAG composition**: Rube's synchronous phases map directly onto `ChoreTree!(a < b)` (sequential) and `ChoreTree!(a | b)` (parallel) combinators, e.g. `resolve_chore < (fast_chores | custom_chores | behavioral_chores | coro_chores) < advance_chore` — preserving the required `ResolveReady → Eval → AdvanceAll` ordering while evaluating independent warp types concurrently.
- **Partitioned triggers, lock-free by construction**: triggers are statically partitioned across workers at layout/compile time so each worker only ever writes to its own partition's `_FutureVals`, eliminating data races **without runtime locks** — a direct extension of Rube's existing SoA/warp compile-time sorting rather than a bolted-on threading layer.
- **Determinism preserved**: because each `FastWarp`/`CustomWarp` chunk only reads `_CurrentVals` and writes disjoint `_FutureVals` ranges, parallel execution is bit-identical to serial execution (proven by construction, not just tested) — important for reproducible RTL simulation, something ad hoc thread-pooling on top of SystemC would need to re-derive per design.
- **Expected speedup**: ~2–8× on multi-core (near-linear for homogeneous circuits, 3–4× typical for heterogeneous/mixed gate types), scaling with worker count and warp-chunk load balance.
- **Coroutine-aware**: `CoroChore` lets suspended/yielding kernels (Rube's `CoroKernelFactory`-based stateful modules, Rube's `stalks` coroutines) requeue themselves on yield without blocking a worker thread — analogous in spirit to SystemC's `SC_THREAD` fiber-style processes, but scheduled by a work-stealing pool instead of a single-threaded cooperative scheduler.

### `swarm`/`symph` — heterogeneous CPU/GPU compute
- **Backend-agnostic dispatch**: `SwarmEngine`/`SwarmDevice` enum-dispatch (not vtables) unifies CPU multithreading, Rust-GPU (WebGPU/Vulkan via SPIR-V), and CUDA (via `cuda-oxide`/PTX) behind one `IComputeDevice`/`IComputeBuffer`/`IComputeKernel` trait surface.
- **Portable kernels via `symph`**: kernel code is written once in `#![no_std]` Rust and compiled both natively (CPU) and to SPIR-V (GPU) via `rust-gpu` — meaning warp evaluation kernels could in principle be offloaded to GPU without maintaining a separate shader-language implementation.
- **Routing hook already anticipated**: Rube's own integration notes call out GPU acceleration as future work, with chore routing via a backend selector (`ChoreTree!` targets can be `CpuChore!()`, `GpuChore!()`, or `GpuAutoChore!()`), so the same dependency graph that drives CPU work-stealing can eventually dispatch large homogeneous `FastWarp` batches (e.g. massive bitwise/arithmetic gate arrays) to the GPU for SIMT execution at a much larger batch width than a CPU core's 64-lane predication.
- **No SystemC equivalent**: SystemC has no standardized GPU offload path at all; any GPU acceleration of a SystemC/TLM model requires ad hoc, model-specific co-simulation bridges (typically via DPI/VPI into custom CUDA/OpenCL code), not a first-class simulator-kernel capability.

### Net comparison
| Aspect | Rube (heist + swarm/symph) | SystemC |
|---|---|---|
| Multi-core scheduling | Native, work-stealing, DAG-composed (`Atelier`/`ChoreTree`) | Not part of the standard; single-threaded reference kernel |
| Data-race safety | Compile-time trigger partitioning, lock-free hot path | N/A — no built-in multi-threaded execution model |
| Determinism under parallelism | Guaranteed by construction (disjoint writes + phase ordering) | N/A |
| GPU offload | First-class hook via `swarm`/`symph` (CPU/WebGPU/CUDA behind one trait), same kernel source for CPU and SPIR-V | None; requires custom co-simulation bridges per project |
| Maturity | Design/integration-plan stage for Rube (parallel SimEngine not yet default), GPU path anticipated but not wired to `rube` yet | Mature but stuck at single-thread by standard; parallel kernels only via proprietary commercial extensions |

In short: SystemC's concurrency model (delta-cycle events, `SC_THREAD`/`SC_METHOD`) is inherently hard to parallelize safely and has no standard multi-core or GPU story. Rube's warp-based architecture was designed from the start to decompose cleanly into independent, partition-owned units of work, making it a much more natural fit for `heist`'s work-stealing multi-core scheduler and `swarm`/`symph`'s CPU/GPU heterogeneous backend — though as of this writing that CPU-parallel and GPU-offload integration for `SimEngine` is a planned/in-progress capability, not yet the default execution path.

---

## Summary

- **Architecture**: Rube is a compile-time-optimized, data-oriented (SoA, warp-batched) synchronous simulator in Rust; SystemC is a runtime, event-driven, object-oriented C++ framework supporting arbitrary timing/concurrency.
- **Functionality**: SystemC is far more mature and complete (TLM, delta-cycles, DPI/VPI, UVM ecosystem); Rube covers RTL-level gates/latches/adders/FIFOs today, with EDA-compatibility features (SV export, DPI, introspection) still in progress.
- **Performance**: Rube is architected for substantially higher raw gate-simulation throughput and lower memory overhead within its restricted synchronous model; SystemC sacrifices some raw speed for generality and standardization.
- **Parallelism**: Rube's warp/partition architecture is a natural fit for `heist`'s work-stealing multi-core scheduler and `swarm`/`symph`'s CPU/GPU heterogeneous compute layer, with deterministic lock-free parallel execution planned/in-progress; SystemC's standard kernel has no built-in multi-core or GPU execution model.

In short, Rube is currently a narrower, performance-first synchronous logic simulator, while SystemC is a broader, standardized, general-purpose HW/SW co-design and verification framework — Rube's own roadmap explicitly frames closing the functionality gap with SystemC/EDA tooling as its next major direction.
