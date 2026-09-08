# Rube vs. SystemC: Comparative Analysis

This document compares Kosh's `rube` simulation engine against SystemC across architecture, functionality, and performance.

For the follow-up measurement and optimization work, see [RubePerformancePlan.md](RubePerformancePlan.md).

---

## Architecture

| Aspect | Rube | SystemC |
|---|---|---|
| **Language/paradigm** | Rust, layout-owned runtime `Module` records with `USeg` ranges, runtime sealing, and compiled warp data | C++ class library, virtual `sc_module`/`sc_interface` inheritance, runtime polymorphism |
| **Simulation model** | Synchronous 3-phase temporal model (Past/Current/Future) with `AdvanceAll()`; no iterative convergence within a phase — O(#modules), single-pass | Discrete-event simulation with **delta cycles**: processes (`SC_METHOD`/`SC_THREAD`) re-evaluate until signals stabilize each timestep, supporting arbitrary combinational feedback and multiple delays |
| **Scheduling granularity** | Freeze-time-sorted "warps" (`FastWarp`/`CustomWarp`) grouping compatible kernels for SIMT-style batch dispatch | Runtime event queue + sensitivity lists; scheduler wakes processes based on dynamic event notifications (`sc_event`) |
| **Dispatch mechanism** | Enum + `match` (`KernelKind::Fast/Behavioral/Custom/Coro`) → static dispatch, no vtables in hot path | Virtual function calls (`process_call_back`), heavier per-process overhead |
| **Memory layout** | Structure-of-Arrays (`TriggerWad._PastVals/_CurrentVals/_FutureVals`), 64-lane bitset predication, zero heap allocation in `Drive()` | Object-oriented, each module/signal is a heap-allocated C++ object; AoS layout, no cache-locality guarantees |
| **Concurrency model** | Coroutine kernels (`CoroKernelFactory`) for explicit stateful FSMs; opt-in Heist CPU warp execution via `Drive_Parallel()` | `SC_THREAD` processes use cooperative coroutines (via `ucontext`/fibers) — conceptually similar but per-object, not batched |
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
| **Verification ecosystem** | None yet (no UVM-equivalent framework) | Mature SystemC and SystemVerilog verification ecosystems; UVM itself is a SystemVerilog methodology commonly used alongside SystemC |
| **Introspection/reflection** | In progress (`IModuleInterface`, SV/VHDL export) — Phase 1 | `sc_object` hierarchy provides runtime name/kind introspection, plus mature debugger/GUI support (e.g. in Questa, VCS) |
| **Simulation control** | In progress (`ISimulationController`: pause/resume/breakpoints) — Phase 3 | Native support via scheduler control APIs, well-integrated with commercial debuggers |
| **Ecosystem/tooling maturity** | New, single-project, no external tool support | 20+ years, IEEE 1666 standard, supported by every major EDA vendor (Synopsys, Cadence, Siemens) |

Functionally, SystemC is a **superset** today — it supports TLM, arbitrary timing, delta-cycle semantics, and has a full verification/tooling ecosystem. Rube's EDA roadmap (Phases 0–5, see [Rube.md](Rube.md#4-eda-compatibility-roadmap)) is explicitly working toward closing this gap (SV/VHDL export, DPI/VPI, package management) but most of it is still "in development" or "pending."

---

## Performance

This is where Rube is designed to have an advantage for its target domain (large synchronous gate/RTL networks), but the repository currently documents targets rather than a completed apples-to-apples benchmark:

- **Dispatch overhead**: SystemC pays virtual-call + event-queue overhead per process invocation; Rube uses static `match` dispatch inside pre-sorted warps, eliminating vtable indirection for the fast/primitive path.
- **Cache behavior**: Rube's SoA `TriggerWad` is designed for sequential trigger scans; exact L1 hit rates and the comparison with SystemC remain benchmark questions.
- **Allocation**: Rube's `Drive()` is zero-heap-allocation; SystemC's delta-cycle event scheduling frequently allocates/deallocates event and process-handle bookkeeping at runtime.
- **Batch execution**: Rube's 64-lane readiness words let the engine skip inactive lane groups with compact bit operations; SystemC's sensitivity-list model evaluates processes individually as events fire, without this compiled warp batching.
- **Target throughput**: Rube documents a 100M+ gate-operations/second target (see [Rube.md §7](Rube.md#7-performance-characteristics)); no repository benchmark currently establishes that figure or a direct SystemC speedup.
- **Trade-off**: SystemC's flexibility (arbitrary delays, feedback loops, TLM) requires the more general (and slower) event-driven scheduler; Rube's speed comes from restricting to a synchronous, feedback-free-per-phase model that can be resolved in one linear pass.

---

## Parallel Execution: Multi-Core/GPU (`heist` + `swarm`/`symph`) vs. SystemC

SystemC's reference kernel is fundamentally **single-threaded**: the scheduler runs one process at a time on one core, and although some commercial simulators (e.g. Questa, Xcelium) offer proprietary multi-threaded/parallel kernels, the open IEEE 1666 SystemC kernel itself has no standardized multi-core or GPU execution model — dependency management via arbitrary delta-cycle events makes safe automatic parallelization hard.

Rube has an implemented opt-in CPU parallel path through Heist, while the separate Swarm/Symph subsystem provides building blocks for future heterogeneous offload. The current Rube simulation loop is not GPU-enabled:

### `heist` — work-stealing CPU parallelism
- **Atelier/Maestro model**: a global work-stealing scheduler (`Atelier`) coordinates per-thread workers (`Maestro`), each with a local run queue; idle workers steal from random victims (Knuth-hash-randomized) — no central lock contention.
- **ChoreTree DAG composition**: Rube's synchronous phases map directly onto `ChoreTree!(a < b)` (sequential) and `ChoreTree!(a | b)` (parallel) combinators, e.g. `resolve_chore < (fast_chores | custom_chores | behavioral_chores | coro_chores) < advance_chore` — preserving the required `ResolveReady → Eval → AdvanceAll` ordering while evaluating independent warp types concurrently.
- **Current safety boundary**: the implementation uses a shared engine pointer for worker callbacks; the documentation should treat partition ownership and determinism as validation requirements, not compile-time guarantees.
- **Determinism goal**: the temporal phase structure is intended to make serial and parallel results match, but this requires dedicated race and equivalence testing for the current opt-in path.
- **Expected speedup**: earlier design estimates were ~2–8× on multi-core; actual scaling is unmeasured and will depend on worker count, circuit size, and warp balance.
- **Coroutine-aware**: `CoroChore` lets suspended/yielding kernels (Rube's `CoroKernelFactory`-based stateful modules, Rube's `stalks` coroutines) requeue themselves on yield without blocking a worker thread — analogous in spirit to SystemC's `SC_THREAD` fiber-style processes, but scheduled by a work-stealing pool instead of a single-threaded cooperative scheduler.

### `swarm`/`symph` — heterogeneous CPU/GPU compute
- **Backend-agnostic dispatch**: `SwarmEngine`/`SwarmDevice` enum-dispatch (not vtables) unifies CPU multithreading, Rust-GPU (WebGPU/Vulkan via SPIR-V), and CUDA (via `cuda-oxide`/PTX) behind one `IComputeDevice`/`IComputeBuffer`/`IComputeKernel` trait surface.
- **Portable kernels via `symph`**: the subsystem contains `#![no_std]` Rust kernels that can target native CPU and SPIR-V/GPU builds via `rust-gpu`; adapting Rube warp evaluation to that interface remains future work.
- **Integration status**: `swarm`/`symph` expose CPU/GPU backends and portable kernels, but current `rube::SimEngine` warp execution uses Heist CPU spawning; GPU routing and Rube kernel offload are future integration work.
- **No SystemC equivalent**: SystemC has no standardized GPU offload path at all; any GPU acceleration of a SystemC/TLM model requires ad hoc, model-specific co-simulation bridges (typically via DPI/VPI into custom CUDA/OpenCL code), not a first-class simulator-kernel capability.

### Net comparison
| Aspect | Rube (heist + swarm/symph) | SystemC |
|---|---|---|
| Multi-core scheduling | Native, work-stealing, DAG-composed (`Atelier`/`ChoreTree`) | Not part of the standard; single-threaded reference kernel |
| Data-race safety | Opt-in parallel path; safety depends on current worker callback/ownership validation | N/A — no built-in multi-threaded execution model |
| Determinism under parallelism | Intended by phase ordering; requires equivalence and race testing | N/A |
| GPU offload | Independent `swarm`/`symph` capability; not wired into Rube simulation yet | None; requires custom co-simulation bridges per project |
| Maturity | Serial Rube path is implemented; Heist parallel path is opt-in; GPU integration is future work | Mature but single-threaded by the standard reference model; parallel kernels only via proprietary extensions |

In short: SystemC's concurrency model (delta-cycle events, `SC_THREAD`/`SC_METHOD`) is inherently difficult to parallelize safely and has no standard GPU execution story. Rube's compiled warp representation is a natural fit for Heist's work-stealing CPU scheduler, and its independent Swarm/Symph layer offers a plausible future GPU path. The current CPU-parallel path is opt-in, while GPU offload is not yet integrated with `SimEngine`.

---

## Summary

- **Architecture**: Rube is a data-oriented, freeze-compiled, warp-batched synchronous simulator in Rust; SystemC is a runtime, event-driven, object-oriented C++ framework supporting arbitrary timing/concurrency.
- **Functionality**: SystemC is far more mature and complete (TLM, delta-cycles, DPI/VPI, UVM ecosystem); Rube covers RTL-level gates/latches/adders/FIFOs today, with EDA-compatibility features (SV export, DPI, introspection) still in progress.
- **Performance**: Rube is architected for substantially higher raw gate-simulation throughput and lower memory overhead within its restricted synchronous model; SystemC sacrifices some raw speed for generality and standardization.
- **Parallelism**: Rube has an opt-in Heist work-stealing CPU path built around compiled warps; deterministic equivalence still needs validation, and `swarm`/`symph` GPU offload is not yet wired into Rube. SystemC's standard kernel has no built-in GPU execution model.

In short, Rube is currently a narrower, performance-first synchronous logic simulator, while SystemC is a broader, standardized, general-purpose HW/SW co-design and verification framework — Rube's own roadmap explicitly frames closing the functionality gap with SystemC/EDA tooling as its next major direction.
