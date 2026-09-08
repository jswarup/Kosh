# Rube Performance Implementation Plan

## Purpose

Turn Rube's performance-oriented architecture into measured, repeatable improvements. The plan prioritizes evidence, preserves serial behavior, and treats CPU parallelism and GPU offload as separate execution paths.

The current repository documents a 100M+ gate-operations/second target, but does not yet provide an apples-to-apples benchmark suite. No optimization estimate in this document should be treated as a measured result until the benchmark gates are complete.

## Scope

In scope:

- Serial `SimEngine::Drive()` performance
- Opt-in Heist execution through `Drive_Parallel()`
- Trigger ownership and serial/parallel equivalence
- CPU SIMD candidates for `FastWarp`
- Future `swarm`/`symph` GPU dispatch for large homogeneous batches
- Memory traffic and allocation measurement

Out of scope for the first iteration:

- Replacing the simulation semantics with a general delta-cycle scheduler
- Broad SystemC feature parity
- Automatic GPU execution for custom callbacks or coroutine kernels
- Unmeasured changes to public simulation semantics

## Guiding Rules

1. Preserve the current serial result as the reference behavior.
2. Benchmark representative circuits before changing hot-path data structures.
3. Keep small workloads on the serial path when scheduler overhead exceeds useful parallel work.
4. Treat `_CurrentVals` as read-only during warp evaluation and make all `_FutureVals` ownership assumptions explicit.
5. Require deterministic serial/parallel equivalence tests before enabling a parallel optimization by default.
6. Do not compare Rube and SystemC using different timing semantics or circuit workloads.

## Baseline Workloads

Create deterministic workloads covering different bottlenecks:

| Workload | Purpose |
|---|---|
| Homogeneous AND/XOR network | Measures `FastWarp` throughput and warp batching |
| Heterogeneous primitive network | Measures warp switching and readiness overhead |
| Wide arithmetic network | Measures `Reg` operations and SIMD potential |
| High-fanout netlist | Measures subscriber traversal and trigger traffic |
| Sparse activity netlist | Measures `_ReadyWords` effectiveness |
| Deep sequential pipeline | Measures temporal advance and edge handling |
| Coroutine-heavy design | Measures `CoroWarp` scheduling overhead |
| Large partitionable netlist | Measures Heist scaling |

Every workload should have fixed topology, fixed initial values, and a fixed number of cycles. Record circuit size, trigger count, warp counts, active-lane ratio, and fan-out distribution.

## Metrics and Harness

Add a repeatable benchmark harness using the repository's existing Rust test/build setup. A Criterion dependency is optional; a dedicated benchmark binary or ignored integration test is sufficient for the first baseline.

Collect:

- Gate operations per second
- Simulation cycles per second
- Nanoseconds per `Drive()` cycle
- Serial versus parallel speedup
- Scaling efficiency by worker count
- First-cycle and steady-state timing separately
- Allocation count and allocated bytes per cycle
- Peak resident memory
- Cache misses and branch misses when platform tooling is available
- GPU dispatch, transfer, synchronization, and kernel time when GPU support is added

Use release builds with a documented toolchain and repeat each measurement enough times to report median and percentile values. Record CPU model, core count, GPU model, driver, operating system, and compiler configuration.

## Execution Phases

### Phase 0: Baseline and Correctness Gate

Deliverables:

- Deterministic workload generators
- Serial benchmark harness
- Baseline report checked into a non-generated results location
- Serial repeatability tests
- Allocation and cycle-count instrumentation

Exit criteria:

- All baseline workloads produce stable outputs across repeated runs.
- The benchmark can distinguish first-cycle cost from steady-state cost.
- The current 100M+ target is labeled as measured, missed, or not applicable for each workload.

### Phase 1: Serial Hot-Path Profiling

Profile `ResolveReadyModules`, readiness-word scanning, each warp evaluator, `TriggerWad::AdvanceAll`, and subscriber traversal.

Candidate improvements:

- Remove redundant bounds or conversion work in inner loops.
- Confirm that warp arrays and trigger arrays have favorable access order.
- Precompute stable indices or masks during `Layout::Freeze()`.
- Avoid changing triple-buffer semantics until profiling shows temporal storage is dominant.

Exit criteria:

- One or more measured bottlenecks are identified.
- Each serial optimization has a before/after benchmark and identical output tests.
- No optimization is accepted solely on cache or branch-prediction assumptions.

### Phase 2: Adaptive Heist CPU Parallelism

Harden the existing opt-in `Drive_Parallel()` path before tuning it.

Tasks:

- Document and isolate the shared engine-pointer callback mechanism.
- Audit every worker callback for read/write access to triggers and warp metadata.
- Add serial-versus-parallel equivalence tests across all warp kinds.
- Add race detection and stress runs with varying worker counts.
- Tune `CpuSpawnQuell!` chunk size and Heist fusion thresholds.
- Add a workload-size threshold that selects serial execution for small circuits.
- Measure scaling at 2, 4, 8, and available worker counts.

Exit criteria:

- No race or equivalence failures across repeated stress runs.
- Parallel results match serial results for all supported workloads.
- Parallel mode improves the large-netlist benchmark and does not regress small-netlist latency beyond an agreed threshold.
- Any remaining unsafe assumptions are documented at the API boundary.

### Phase 3: Trigger Ownership and Memory Traffic

Only proceed if profiling shows trigger access or subscriber storage is a material bottleneck.

Tasks:

- Generate explicit ownership metadata during `Layout::Freeze()`.
- Validate that each parallel worker's writes are disjoint or define a safe reduction rule.
- Measure subscriber spans, trigger buffers, and readiness words separately.
- Evaluate compressed subscriber indices only for sparse/high-fanout workloads.
- Compare triple-buffering with alternatives using memory bandwidth and correctness measurements.

Exit criteria:

- Ownership validation rejects unsafe layouts before execution.
- Memory reduction does not increase cycle time or complicate temporal semantics without a measured benefit.

### Phase 4: CPU SIMD Fast Kernels

Prototype SIMD only for operations with simple, regular `Reg` layouts, initially bitwise primitives and wide arithmetic.

Tasks:

- Define scalar reference kernels and SIMD implementations behind the same test vectors.
- Check alignment, unknown-mask propagation, and tail handling.
- Provide architecture-gated implementations with scalar fallback.
- Benchmark AVX2/AVX-512 where available and verify behavior on non-x86 targets.

Exit criteria:

- SIMD output is bit-identical to scalar output, including X-mask behavior.
- SIMD produces a repeatable gain on workloads where the operation is dominant.
- Dispatch overhead does not regress small warps.

### Phase 5: GPU Prototype Through Swarm/Symph

Treat GPU support as a separate backend, not as an automatic replacement for CPU execution.

Candidate workloads:

- Large homogeneous `FastWarp` batches
- Wide bitwise operations
- Wide arithmetic with regular memory access

Avoid initially:

- Small warps
- Custom closures
- Coroutine kernels
- Designs requiring frequent CPU/GPU synchronization

Tasks:

- Define a GPU-safe kernel representation for a restricted subset of `FastWarp`.
- Map trigger buffers to `swarm` buffers and define ownership of future values.
- Compile a minimal `symph` kernel for CPU and SPIR-V/GPU targets.
- Measure transfer, dispatch, synchronization, and kernel time separately.
- Add an explicit backend selector and CPU fallback.

Exit criteria:

- GPU results match the scalar CPU reference.
- GPU execution wins after transfer and synchronization costs are included.
- Unsupported kernels remain on CPU without changing simulation results.
- GPU dispatch is not enabled by default until workload thresholds are validated.

### Phase 6: SystemC Comparison

Run a fair comparison only after Rube's baseline harness is stable.

Rules:

- Use equivalent synchronous circuits and equivalent cycle/delta-cycle observation points.
- Separate gate-level workloads from TLM workloads; do not compare them as the same category.
- Report throughput, memory, startup cost, and scaling independently.
- Compare serial Rube with the SystemC reference model first, then compare opt-in Heist Rube separately.
- Clearly distinguish measured results from architectural expectations.

## Proposed Results Table

Maintain one table per benchmark revision:

| Workload | Engine | Workers | Backend | Cycles/s | ns/cycle | Allocations/cycle | Peak memory | Notes |
|---|---|---:|---|---:|---:|---:|---:|---|
|  | Rube serial | 1 | CPU |  |  |  |  |  |
|  | Rube parallel | 4 | CPU |  |  |  |  |  |
|  | Rube parallel | 8 | CPU |  |  |  |  |  |
|  | Rube GPU | N/A | Swarm/Symph |  |  |  |  |  |
|  | SystemC | N/A | CPU |  |  |  |  |  |

## Priority Order

1. Benchmark harness and deterministic reference tests
2. Serial profiling and low-risk hot-path cleanup
3. Heist safety validation and adaptive serial/parallel selection
4. Heist chunk and fusion tuning
5. Trigger ownership and memory optimization if profiling justifies it
6. SIMD fast kernels
7. GPU prototype for large regular warps
8. Reproducible SystemC comparison

## Definition of Done

The performance effort is complete for an iteration when:

- Results are reproducible on documented hardware and toolchains.
- Serial behavior remains the reference and all optimized paths are equivalence-tested.
- Parallel safety assumptions are explicit and tested.
- Reported speedups include scheduler, transfer, and synchronization overhead.
- Rube documentation reports measured results separately from targets and architectural expectations.
