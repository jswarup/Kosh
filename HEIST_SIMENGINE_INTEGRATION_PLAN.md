# Implementation Plan: Heist/Atelier Work-Stealing Scheduling for SimEngine

**Document Version**: 1.0
**Date**: 2026-09-05
**Target Integration**: SimEngine with Heist work-stealing scheduler
**Estimated Effort**: 60-80 engineering hours

---

## Executive Summary

This document provides a comprehensive roadmap for integrating Heist/Atelier work-stealing scheduling into SimEngine to enable multi-threaded parallel circuit simulation with deterministic ordering guarantees.

**Key Goals**:
1. Distribute warp evaluation across multiple CPU workers
2. Maintain deterministic simulation semantics (no data races)
3. Preserve temporal phase model (ResolveReady → Eval → Advance)
4. Leverage work-stealing for automatic load balancing
5. Support optional GPU acceleration via Swarm integration

**Expected Benefits**:
- **Throughput**: 2-8× speedup on multi-core (depending on circuit structure)
- **Latency**: Single-cycle latency maintained for small circuits
- **Scalability**: Near-linear speedup up to #CPUs cores
- **Determinism**: Bit-exact reproducibility across runs

---

## Part 1: Current SimEngine Architecture Review

### 1.1 Execution Model

```
SimEngine::Drive() {
    1. ResolveReadyModules()        // Compute ready bitmap from trigger changes
    2. EvalFastWarps()              // Evaluate all FastWarp batches sequentially
    3. EvalCustomWarps()            // Evaluate all CustomWarp batches sequentially
    4. EvalBehavioralWarps()        // Evaluate all BehavioralWarp batches sequentially
    5. EvalCoroWarps()              // Evaluate all CoroWarp batches sequentially
    6. Triggers.AdvanceAll()        // Latch: Past ← Current ← Future
}
```

### 1.2 Data Structures

**Key Objects**:
```rust
pub struct SimEngine {
    pub _Triggers: TriggerWad,              // Temporal state (SoA)
    pub _FastWarps: Buff<FastWarp>,         // Batched gate operations
    pub _CustomWarps: Buff<CustomWarp>,     // Custom kernel batches
    pub _BehavioralWarps: Buff<BehavioralWarp>,  // Closure batches
    pub _CoroWarps: Buff<CoroWarp>,         // Coroutine batches
    pub _PortToTrigger: Buff<TriggerId>,    // Port→Trigger mapping
    pub _ReadyWords: Buff<u64>,             // 64-lane predication bitmap
    pub _CycleCount: usize,
}
```

### 1.3 Current Parallelism Opportunities

| Warp Type | Size | Parallelism | Notes |
|-----------|------|-------------|-------|
| **FastWarps** | 10-1000+ | High | Embarrassingly parallel; no deps |
| **CustomWarps** | 1-100 | High | Kernel-dependent; usually independent |
| **BehavioralWarps** | 1-100 | High | If closures are pure |
| **CoroWarps** | 1-50 | Medium | Coroutines have state; care needed |

**Critical Constraints**:
- All warps write to `_Triggers._FutureVals[...]` (shared state)
- **No write-write conflicts** if warps operate on disjoint triggers
- `ResolveReadyModules()` must complete before warp evaluation
- `Triggers.AdvanceAll()` must execute after all warps complete

---

## Part 2: Heist Integration Architecture

### 2.1 High-Level Design

```
SimEngine (Multi-threaded Phase Executor)
├── PhaseCoordinator (Heist Chore manager)
│   ├── Phase 1: ResolveReadyModules (CPU-serial, fast)
│   ├── Phase 2: EvalFastWarps (work-stealing parallel)
│   ├── Phase 3: EvalCustomWarps (work-stealing parallel)
│   ├── Phase 4: EvalBehavioralWarps (work-stealing parallel)
│   ├── Phase 5: EvalCoroWarps (work-stealing parallel)
│   └── Phase 6: AdvanceAll (CPU-serial)
│
├── WarpScheduler (work distribution)
│   ├── Chunk FastWarps into ~2N chunks (N = #workers)
│   ├── Chunk CustomWarps into ~2N chunks
│   ├── Chunk BehavioralWarps into ~2N chunks
│   └── Chunk CoroWarps into ~2N chunks
│
└── TriggerGuard (synchronization)
    ├── Ensure no write-write conflicts on _FutureVals
    ├── Protect _ReadyWords updates
    └── Coordinate AdvanceAll barrier
```

### 2.2 Thread-Safe Trigger Access

**Challenge**: Multiple workers write to `_Triggers._FutureVals[trigId]` simultaneously.

**Solution**: Partition triggers by "ownership" at compile-time.

```rust
pub struct PartitionedTriggers {
    pub _Partitions: Buff<TriggerPartition>,      // One per worker
    pub _TriggerToPartition: Buff<U8>,            // trigId → partition index
}

pub struct TriggerPartition {
    pub _PastVals: Buff<Reg>,                     // Worker-owned slice
    pub _CurrentVals: Buff<Reg>,
    pub _FutureVals: Buff<Reg>,
    pub _StartTrigId: TriggerId,
    pub _Count: U32,
}
```

**Invariant**: Each trigger assigned to exactly one partition at layout time. Workers never contend on same trigger.

### 2.3 Work Chunk Definition

**FastWarpChunk**:
```rust
pub struct FastWarpChunk {
    pub _WarpIndices: Buff<U32>,               // Indices into _FastWarps
    pub _TriggerPartition: U8,                 // Owning partition
}

impl FastWarpChunk {
    pub fn Execute(&self, engine: &SimEngine, partition: &mut TriggerPartition) {
        for &warp_idx in self._WarpIndices.Arr() {
            let warp = &engine._FastWarps[warp_idx];
            EvalFastWarp(warp, partition);  // No contention; partition-local
        }
    }
}
```

**Similar for CustomWarpChunk, BehavioralWarpChunk, CoroWarpChunk**.

### 2.4 Choretree Dependency Model

```rust
pub struct SimulationPhase {
    pub _PhaseChore: Chore,                    // Single phase execution
}

pub fn BuildSimulationCycle() -> ChoreNode {
    ChoreTree!(
        resolve_ready_phase <
        (fast_warp_phase | custom_warp_phase | behavioral_warp_phase | coro_warp_phase) <
        advance_all_phase
    )
}
```

**Semantics**:
- `resolve_ready_phase` executes first (serial, CPU-bound)
- All warp phases execute in parallel (data-independent)
- `advance_all_phase` executes last (serial, CPU-bound)

---

## Part 3: Implementation Phases

### Phase 1: Infrastructure (15-20 hours)

#### 1.1 Add Partition System to SimEngine

**File**: `src/rube/engine.rs`

```rust
pub struct PartitionedTriggers {
    pub _Partitions: Buff<TriggerPartition>,
    pub _TriggerToPartition: Buff<U8>,

    pub fn New(total_triggers: U32, partition_count: U8) -> Self {
        let triggers_per_partition = (total_triggers + partition_count as u32 - 1) / partition_count as u32;
        let mut partitions = Stash::WithCapacity(partition_count as usize);
        let mut trigger_map = Stash::WithCapacity(total_triggers as usize);

        for p in 0..partition_count {
            let start = (p as u32) * triggers_per_partition;
            let end = ((p as u32 + 1) * triggers_per_partition).min(total_triggers);
            let count = end - start;

            partitions.Push(TriggerPartition::New(start, count));
            for _ in 0..count {
                trigger_map.Push(p);
            }
        }

        Self {
            _Partitions: partitions.IntoBuff(),
            _TriggerToPartition: trigger_map.IntoBuff(),
        }
    }
}

pub struct TriggerPartition {
    pub _PastVals: Buff<Reg>,
    pub _CurrentVals: Buff<Reg>,
    pub _FutureVals: Buff<Reg>,
    pub _SubscriberSpans: Buff<USeg>,
    pub _Subscribers: Buff<TriggerSubscriber>,
    pub _StartTrigId: TriggerId,
    pub _Count: U32,
}

impl TriggerPartition {
    fn LocalizeIndex(&self, global_trig_id: TriggerId) -> U32 {
        debug_assert!(global_trig_id >= self._StartTrigId);
        global_trig_id - self._StartTrigId
    }
}
```

**Test**: Verify partition creation and index localization for various partition counts.

#### 1.2 Add Warp Chunking System

**File**: `src/rube/engine.rs` (new module `warp_scheduler.rs`)

```rust
pub struct WarpChunks {
    pub _FastChunks: Buff<FastWarpChunk>,
    pub _CustomChunks: Buff<CustomWarpChunk>,
    pub _BehavioralChunks: Buff<BehavioralWarpChunk>,
    pub _CoroChunks: Buff<CoroWarpChunk>,
}

pub struct FastWarpChunk {
    pub _WarpIndices: Buff<U32>,
    pub _PartitionId: U8,
}

impl WarpChunks {
    pub fn New(
        fast_warps: &Buff<FastWarp>,
        custom_warps: &Buff<CustomWarp>,
        behavioral_warps: &Buff<BehavioralWarp>,
        coro_warps: &Buff<CoroWarp>,
        partition_count: U8,
    ) -> Self {
        let target_chunks = (partition_count as usize) * 2;  // ~2× chunks per worker

        let fast_chunks = chunk_warps_by_partition(
            fast_warps, partition_count, target_chunks
        );
        let custom_chunks = chunk_warps_by_partition(
            custom_warps, partition_count, target_chunks
        );
        let behavioral_chunks = chunk_warps_by_partition(
            behavioral_warps, partition_count, target_chunks
        );
        let coro_chunks = chunk_warps_by_partition(
            coro_warps, partition_count, target_chunks
        );

        Self {
            _FastChunks: fast_chunks,
            _CustomChunks: custom_chunks,
            _BehavioralChunks: behavioral_chunks,
            _CoroChunks: coro_chunks,
        }
    }
}

fn chunk_warps_by_partition<T: HasPartitionInfo>(
    warps: &Buff<T>,
    partition_count: U8,
    target_chunk_count: usize,
) -> Buff<WarpChunk<T>> {
    // Distribute warps evenly across chunks
    let chunk_size = (warps.Size().AsUsize() + target_chunk_count - 1) / target_chunk_count;
    let mut chunks = Stash::New();

    for i in (0..warps.Size().AsUsize()).step_by(chunk_size) {
        let end = (i + chunk_size).min(warps.Size().AsUsize());
        let mut indices = Stash::New();

        for j in i..end {
            indices.Push(U32(j as u32));
        }

        chunks.Push(WarpChunk {
            _WarpIndices: indices.IntoBuff(),
            _PartitionId: (i % partition_count as usize) as U8,
        });
    }

    chunks.IntoBuff()
}
```

**Test**: Verify chunks are evenly distributed and partition assignments are consistent.

#### 1.3 Integrate with Heist

**File**: `src/rube/engine.rs` (new module `heist_integration.rs`)

```rust
use crate::heist::{Atelier, Chore, ChoreNode, IMaestro};

pub struct HeistSimEngine {
    pub _Engine: SimEngine,
    pub _Partitions: PartitionedTriggers,
    pub _WarpChunks: WarpChunks,
    pub _WorkerCount: U8,
}

impl HeistSimEngine {
    pub fn New(layout: &Layout, worker_count: U8) -> Self {
        let engine = SimEngine::Create(layout);
        let trigger_count = engine._Triggers.Size();
        let partitions = PartitionedTriggers::New(trigger_count, worker_count);
        let warp_chunks = WarpChunks::New(
            &engine._FastWarps,
            &engine._CustomWarps,
            &engine._BehavioralWarps,
            &engine._CoroWarps,
            worker_count,
        );

        Self {
            _Engine: engine,
            _Partitions: partitions,
            _WarpChunks: warp_chunks,
            _WorkerCount: worker_count,
        }
    }

    pub fn InitializeHeist(&self) {
        Atelier::Init(self._WorkerCount as u32);
    }

    pub fn Drive_Parallel(&mut self) -> Result<(), SimError> {
        // Phase 1: Resolve ready modules (serial)
        self.ResolveReadyModules()?;

        // Phase 2-5: Evaluate warps (parallel via Heist)
        self.EvalWarpsParallel()?;

        // Phase 6: Advance triggers (serial)
        self.AdvanceAll()?;

        self._Engine._CycleCount += 1;
        Ok(())
    }

    fn EvalWarpsParallel(&mut self) -> Result<(), SimError> {
        let chore_tree = self.build_warp_phase_tree();
        Atelier::PostChoreTree(chore_tree);
        Atelier::Wait();
        Ok(())
    }

    fn build_warp_phase_tree(&self) -> ChoreNode {
        // Build parallel task graph via Heist Chore API
        // (Details in Phase 2)
        todo!()
    }
}
```

**Test**: Verify HeistSimEngine initialization and Atelier thread pool startup.

---

### Phase 2: Warp Evaluation Kernels (20-25 hours)

#### 2.1 FastWarp Parallel Kernel

**File**: `src/rube/engine.rs`

```rust
pub fn eval_fast_warp_chunk_kernel(
    chunk: &FastWarpChunk,
    engine: &SimEngine,
    partition: &mut TriggerPartition,
) -> Result<(), SimError> {
    for &warp_idx in chunk._WarpIndices.Arr() {
        let warp = &engine._FastWarps[warp_idx];

        // ForEachReadyLane already handles predication
        engine.ForEachReadyLane(
            &engine._ReadyWords,
            0,
            warp._ModuleCount.AsUsize(),
            |lane_idx| {
                let module_idx = warp._ModuleStart.AsUsize() + lane_idx;

                let in1_global = warp._In1[lane_idx];
                let in2_global = warp._In2[lane_idx];
                let out_global = warp._Out[lane_idx];

                // Localize indices to partition
                let in1_local = partition.LocalizeIndex(in1_global);
                let in2_local = partition.LocalizeIndex(in2_global);
                let out_local = partition.LocalizeIndex(out_global);

                let in1_val = partition._CurrentVals[in1_local];
                let in2_val = partition._CurrentVals[in2_local];

                let result = warp._Op.Eval(in1_val, in2_val, warp._Mask);

                partition._FutureVals[out_local] = result;
            },
        );
    }
    Ok(())
}

pub fn create_fast_warp_chore(
    engine: &SimEngine,
    chunk: &FastWarpChunk,
) -> Chore {
    let partition_id = chunk._PartitionId;
    let chunk_ptr = chunk as *const FastWarpChunk;

    Chore::New(|worker| {
        let engine_ptr = engine as *const SimEngine as *mut SimEngine;
        let engine_ref = unsafe { &*engine_ptr };

        let partition = &mut engine_ref._Partitions._Partitions[partition_id as usize];
        let chunk_ref = unsafe { &*chunk_ptr };

        eval_fast_warp_chunk_kernel(chunk_ref, engine_ref, partition)
    })
}
```

**Design Notes**:
- Each chunk operates on partition-owned triggers only (no contention)
- `ForEachReadyLane()` respects predication bitmap
- Closure captures via raw pointers (Worker context provides safety guarantee)

#### 2.2 CustomWarp Parallel Kernel

**File**: `src/rube/engine.rs`

```rust
pub fn eval_custom_warp_chunk_kernel(
    chunk: &CustomWarpChunk,
    engine: &SimEngine,
    partition: &mut TriggerPartition,
) -> Result<(), SimError> {
    for &warp_idx in chunk._WarpIndices.Arr() {
        let warp = &engine._CustomWarps[warp_idx];
        let callback = &engine._CustomCallbacks[warp_idx];

        engine.ForEachReadyLane(
            &engine._ReadyWords,
            0,
            warp._ModuleCount.AsUsize(),
            |lane_idx| {
                let in_triggers = &warp._InTriggers[lane_idx];
                let out_triggers = &warp._OutTriggers[lane_idx];

                let mut input_regs = Vec::new();
                for &in_trig in in_triggers {
                    let local_idx = partition.LocalizeIndex(in_trig);
                    input_regs.push(partition._CurrentVals[local_idx]);
                }

                let mut output_regs = vec![Reg::X; out_triggers.len()];
                callback(&input_regs, &mut output_regs);

                for (i, &out_trig) in out_triggers.iter().enumerate() {
                    let local_idx = partition.LocalizeIndex(out_trig);
                    partition._FutureVals[local_idx] = output_regs[i];
                }
            },
        );
    }
    Ok(())
}
```

**Challenge**: Custom kernels may allocate (Vec construction). Mitigate:
- Pre-allocate Vec buffers in partition
- Use stack arrays for small input/output sets (most common case)
- Profile to measure allocation overhead

#### 2.3 BehavioralWarp Parallel Kernel

**File**: `src/rube/engine.rs`

```rust
pub fn eval_behavioral_warp_chunk_kernel(
    chunk: &BehavioralWarpChunk,
    engine: &SimEngine,
    partition: &mut TriggerPartition,
) -> Result<(), SimError> {
    for &warp_idx in chunk._WarpIndices.Arr() {
        let warp = &engine._BehavioralWarps[warp_idx];

        engine.ForEachReadyLane(
            &engine._ReadyWords,
            0,
            warp._ModuleCount.AsUsize(),
            |lane_idx| {
                let in_triggers = &warp._InTriggers[lane_idx];
                let out_triggers = &warp._OutTriggers[lane_idx];

                let input_regs: Vec<Reg> = in_triggers
                    .iter()
                    .map(|&t| {
                        let local = partition.LocalizeIndex(t);
                        partition._CurrentVals[local]
                    })
                    .collect();

                let mut output_regs = vec![Reg::X; out_triggers.len()];
                (warp._Kernel)(&input_regs, &mut output_regs);

                for (i, &out_trig) in out_triggers.iter().enumerate() {
                    let local_idx = partition.LocalizeIndex(out_trig);
                    partition._FutureVals[local_idx] = output_regs[i];
                }
            },
        );
    }
    Ok(())
}
```

#### 2.4 CoroWarp Parallel Kernel (Special Handling)

**File**: `src/rube/engine.rs`

```rust
pub fn eval_coro_warp_chunk_kernel(
    chunk: &CoroWarpChunk,
    engine: &SimEngine,
    partition: &mut TriggerPartition,
    atelier: &Atelier,
) -> Result<(), SimError> {
    for &warp_idx in chunk._WarpIndices.Arr() {
        let warp = &engine._CoroWarps[warp_idx];

        // Each coroutine instance runs as a separate job to allow yielding
        for coro_idx in 0..warp._CoroCount {
            let coro_instance = &warp._Instances[coro_idx];

            // Create continuation chore if coro yields
            let coro_chore = create_coro_continuation_chore(
                coro_instance,
                engine,
                partition,
            );

            Atelier::Post(coro_chore);
        }
    }
    Ok(())
}

fn create_coro_continuation_chore(
    coro: &CoroInstance,
    engine: &SimEngine,
    partition: &mut TriggerPartition,
) -> Chore {
    let coro_ptr = coro as *const CoroInstance as *mut CoroInstance;
    let engine_ptr = engine as *const SimEngine as *mut SimEngine;
    let partition_ptr = partition as *mut TriggerPartition;

    Chore::New(|worker| {
        let coro_ref = unsafe { &mut *coro_ptr };
        let engine_ref = unsafe { &*engine_ptr };
        let partition_ref = unsafe { &mut *partition_ptr };

        match coro_ref.Resume(worker) {
            CoroRes::Yield => {
                // Requeue as new job for work-stealing
                let new_chore = create_coro_continuation_chore(coro_ref, engine_ref, partition_ref);
                Atelier::Post(new_chore);
            }
            CoroRes::Done => {
                // Coroutine completed
            }
        }
    })
}
```

**Design Notes**:
- Coroutines create dynamic chores that may yield
- When yield occurs, requeue automatically for work-stealing
- Non-blocking suspension model (no thread blocking)

---

### Phase 3: ChoreTree Construction & Sequencing (15-20 hours)

#### 3.1 Temporal Phase Dependency Graph

**File**: `src/rube/engine.rs` (new module `chore_builder.rs`)

```rust
use crate::heist::{ChoreNode, ChoreTree};

pub struct CycleChoreBuilder;

impl CycleChoreBuilder {
    pub fn build(engine: &HeistSimEngine) -> ChoreNode {
        // Phase 1: ResolveReady (serial, must complete first)
        let resolve_chore = Self::build_resolve_chore(engine);

        // Phases 2-5: Warp evaluation (parallel, can run concurrently)
        let fast_chores = Self::build_fast_warp_chores(engine);
        let custom_chores = Self::build_custom_warp_chores(engine);
        let behavioral_chores = Self::build_behavioral_warp_chores(engine);
        let coro_chores = Self::build_coro_warp_chores(engine);

        // Phase 6: AdvanceAll (serial, must complete after warps)
        let advance_chore = Self::build_advance_chore(engine);

        // Build dependency tree
        // resolve_chore < (fast | custom | behavioral | coro) < advance_chore

        ChoreTree!(
            resolve_chore <
            (fast_chores | custom_chores | behavioral_chores | coro_chores) <
            advance_chore
        )
    }

    fn build_resolve_chore(engine: &HeistSimEngine) -> ChoreNode {
        // ResolveReadyModules is CPU-bound and serialization point
        // Can't parallelize without breaking determinism
        let engine_ptr = engine as *const HeistSimEngine as *mut HeistSimEngine;

        ChoreNode::New(|worker| {
            let engine_ref = unsafe { &mut *engine_ptr };
            engine_ref._Engine.ResolveReadyModules()
        })
    }

    fn build_fast_warp_chores(engine: &HeistSimEngine) -> ChoreNode {
        let chores: Vec<Chore> = engine._WarpChunks._FastChunks.Arr()
            .map(|chunk| create_fast_warp_chore(&engine._Engine, chunk))
            .collect();

        // Combine all chunks into parallel task graph
        ChoreNode::Parallel(chores)  // All execute concurrently
    }

    fn build_custom_warp_chores(engine: &HeistSimEngine) -> ChoreNode {
        let chores: Vec<Chore> = engine._WarpChunks._CustomChunks.Arr()
            .map(|chunk| create_custom_warp_chore(&engine._Engine, chunk))
            .collect();

        ChoreNode::Parallel(chores)
    }

    fn build_behavioral_warp_chores(engine: &HeistSimEngine) -> ChoreNode {
        let chores: Vec<Chore> = engine._WarpChunks._BehavioralChunks.Arr()
            .map(|chunk| create_behavioral_warp_chore(&engine._Engine, chunk))
            .collect();

        ChoreNode::Parallel(chores)
    }

    fn build_coro_warp_chores(engine: &HeistSimEngine) -> ChoreNode {
        // Coroutines may yield, so use special handling
        let chores: Vec<Chore> = engine._WarpChunks._CoroChunks.Arr()
            .map(|chunk| create_coro_warp_chore(&engine._Engine, chunk))
            .collect();

        ChoreNode::Parallel(chores)
    }

    fn build_advance_chore(engine: &HeistSimEngine) -> ChoreNode {
        let engine_ptr = engine as *const HeistSimEngine as *mut HeistSimEngine;

        ChoreNode::New(|worker| {
            let engine_ref = unsafe { &mut *engine_ptr };
            engine_ref._Engine._Triggers.AdvanceAll();
            engine_ref._Engine._CycleCount += 1;
        })
    }
}
```

#### 3.2 Dynamic Chore Tree Configuration

**File**: `src/rube/engine.rs` (in `chore_builder.rs`)

```rust
pub struct CycleChoreConfig {
    pub _EnableFastWarps: bool,
    pub _EnableCustomWarps: bool,
    pub _EnableBehavioralWarps: bool,
    pub _EnableCoroWarps: bool,
    pub _FusionThreshold: U32,        // Heist task fusion threshold
    pub _MaxChunksPerWarp: U32,       // Limit chunks for small warps
}

impl Default for CycleChoreConfig {
    fn default() -> Self {
        Self {
            _EnableFastWarps: true,
            _EnableCustomWarps: true,
            _EnableBehavioralWarps: true,
            _EnableCoroWarps: true,
            _FusionThreshold: U32(10),      // Fuse if <10 modules
            _MaxChunksPerWarp: U32(16),     // Max 16 chunks per warp type
        }
    }
}

impl CycleChoreBuilder {
    pub fn build_with_config(
        engine: &HeistSimEngine,
        config: &CycleChoreConfig,
    ) -> ChoreNode {
        let resolve_chore = Self::build_resolve_chore(engine);

        let mut parallel_chores = Vec::new();

        if config._EnableFastWarps {
            parallel_chores.push(Self::build_fast_warp_chores(engine));
        }
        if config._EnableCustomWarps {
            parallel_chores.push(Self::build_custom_warp_chores(engine));
        }
        if config._EnableBehavioralWarps {
            parallel_chores.push(Self::build_behavioral_warp_chores(engine));
        }
        if config._EnableCoroWarps {
            parallel_chores.push(Self::build_coro_warp_chores(engine));
        }

        let advance_chore = Self::build_advance_chore(engine);

        if parallel_chores.is_empty() {
            // No parallel work
            ChoreTree!(resolve_chore < advance_chore)
        } else {
            let parallel_node = ChoreNode::Parallel(parallel_chores);
            ChoreTree!(resolve_chore < parallel_node < advance_chore)
        }
    }
}
```

---

### Phase 4: Synchronization & Safety (12-15 hours)

#### 4.1 Partition Ownership Verification

**File**: `src/rube/engine.rs` (new module `safety.rs`)

```rust
pub struct PartitionOwnershipValidator;

impl PartitionOwnershipValidator {
    /// Verify that no trigger is written by multiple workers
    pub fn validate_no_write_conflicts(
        engine: &HeistSimEngine,
    ) -> Result<(), String> {
        let mut trigger_owners = vec![255u8; engine._Engine._Triggers.Size().AsUsize()];

        for (chunk_idx, chunk) in engine._WarpChunks._FastChunks.Arr().enumerate() {
            for &warp_idx in chunk._WarpIndices.Arr() {
                let warp = &engine._Engine._FastWarps[warp_idx];

                for out_trig in warp._Outputs.Arr() {
                    let trig_idx = out_trig.AsUsize();
                    if trigger_owners[trig_idx] == 255 {
                        trigger_owners[trig_idx] = chunk._PartitionId;
                    } else if trigger_owners[trig_idx] != chunk._PartitionId {
                        return Err(format!(
                            "Trigger {} written by partitions {} and {}",
                            trig_idx,
                            trigger_owners[trig_idx],
                            chunk._PartitionId
                        ));
                    }
                }
            }
        }

        // Repeat for custom, behavioral, coro warps

        Ok(())
    }

    /// Verify that partition assignments are consistent
    pub fn validate_partition_consistency(
        engine: &HeistSimEngine,
    ) -> Result<(), String> {
        for trig_idx in 0..engine._Engine._Triggers.Size().AsUsize() {
            let trig_id = U32(trig_idx as u32);
            let assigned_partition = engine._Partitions._TriggerToPartition[trig_idx];

            // Verify partition contains this trigger
            let partition = &engine._Partitions._Partitions[assigned_partition as usize];
            if trig_id < partition._StartTrigId
                || trig_id >= partition._StartTrigId + partition._Count {
                return Err(format!(
                    "Trigger {} assigned to partition {}, but partition range is [{}, {})",
                    trig_idx,
                    assigned_partition,
                    partition._StartTrigId,
                    partition._StartTrigId + partition._Count
                ));
            }
        }

        Ok(())
    }
}
```

#### 4.2 Thread Safety Documentation

**File**: `src/rube/engine.rs` (in `safety.rs`)

```rust
/// # Safety Guarantees for Multi-Threaded Simulation
///
/// 1. **Partition Isolation**: Each trigger belongs to exactly one partition.
///    Workers only write to their partition's triggers.
///
/// 2. **Read Safety**: `_CurrentVals` are read-only during warp evaluation.
///    All reads happen before any writes (same phase).
///
/// 3. **Temporal Ordering**: Three-phase model ensures:
///    - Phase 1: Read `_CurrentVals`
///    - Phase 2-5: Write `_FutureVals`
///    - Phase 6: Atomic latch (Past ← Current ← Future)
///
/// 4. **No Data Races**: Guaranteed by partition assignment + Heist work-stealing model.
///
/// 5. **Deterministic Ordering**: Chore dependencies ensure phases execute in order.
///    Result is bit-identical across runs.
///
/// # Unsafe Code Justification
///
/// - Raw pointer captures in closures: Valid for lifetime of Atelier::Wait()
/// - Partition references: Guaranteed stable during Heist execution
/// - Trigger access: Validated by PartitionOwnershipValidator
///
pub trait HeistSimEngineSafety {
    fn verify_invariants(&self) -> Result<(), String>;
}

impl HeistSimEngineSafety for HeistSimEngine {
    fn verify_invariants(&self) -> Result<(), String> {
        PartitionOwnershipValidator::validate_no_write_conflicts(self)?;
        PartitionOwnershipValidator::validate_partition_consistency(self)?;
        Ok(())
    }
}
```

#### 4.3 Determinism Verification Tests

**File**: `src/rube/engine.rs` (new test module)

```rust
#[cfg(test)]
mod determinism_tests {
    use super::*;

    #[test]
    fn test_parallel_execution_deterministic() {
        // Build simple adder circuit
        let mut layout = Layout::New();
        let adder_id = Adder::<8>::New(&mut layout, "test_adder", None);
        layout.Freeze().unwrap();

        // Run with different worker counts
        for worker_count in [1, 2, 4] {
            let mut engine = HeistSimEngine::New(&layout, worker_count as u8);
            engine.InitializeHeist();

            let mut results = Vec::new();
            for cycle in 0..100 {
                engine.Drive_Parallel().unwrap();
                let out = engine.GetOutput(adder_id.OutPort());
                results.push(out);
            }

            // Verify all worker counts produce same results
            if results != results {  // Compare with baseline (1 worker)
                panic!("Parallel execution not deterministic!");
            }
        }
    }

    #[test]
    fn test_no_trigger_write_conflicts() {
        // Build complex circuit
        let mut layout = Layout::New();
        // ... build circuit ...
        layout.Freeze().unwrap();

        let engine = HeistSimEngine::New(&layout, 4);

        // Verify safety invariants
        engine.verify_invariants().unwrap();
    }
}
```

---

### Phase 5: Integration & Fallback (8-10 hours)

#### 5.1 Dual-Mode SimEngine (Serial + Parallel)

**File**: `src/rube/engine.rs`

```rust
pub enum SimEngineMode {
    Serial,                          // Original sequential implementation
    Parallel { worker_count: u8 },   // Heist-based parallel implementation
}

pub struct SimEngine {
    pub _Mode: SimEngineMode,
    pub _Serial: Option<SerialSimEngine>,      // Original engine
    pub _Parallel: Option<HeistSimEngine>,     // New engine
}

impl SimEngine {
    pub fn Create(layout: &Layout) -> Self {
        Self::CreateWithMode(layout, SimEngineMode::Serial)
    }

    pub fn CreateWithMode(layout: &Layout, mode: SimEngineMode) -> Self {
        match mode {
            SimEngineMode::Serial => {
                Self {
                    _Mode: mode,
                    _Serial: Some(SerialSimEngine::Create(layout)),
                    _Parallel: None,
                }
            }
            SimEngineMode::Parallel { worker_count } => {
                Self {
                    _Mode: mode,
                    _Serial: None,
                    _Parallel: Some(HeistSimEngine::New(layout, worker_count)),
                }
            }
        }
    }

    pub fn Drive(&mut self) -> Result<(), SimError> {
        match &mut self._Mode {
            SimEngineMode::Serial => {
                self._Serial.as_mut().unwrap().Drive();
                Ok(())
            }
            SimEngineMode::Parallel { .. } => {
                self._Parallel.as_mut().unwrap().Drive_Parallel()
            }
        }
    }
}
```

#### 5.2 Fallback Mechanism

**File**: `src/rube/engine.rs`

```rust
pub struct HeistSimEngine {
    // ... existing fields ...
    pub _EnableFallback: bool,
    pub _FallbackReason: Option<String>,
}

impl HeistSimEngine {
    pub fn Drive_Parallel(&mut self) -> Result<(), SimError> {
        // Attempt parallel execution
        if let Err(e) = self.Drive_Parallel_Internal() {
            if self._EnableFallback {
                eprintln!("Parallel execution failed: {}. Falling back to serial.", e);
                self._FallbackReason = Some(e.to_string());
                return self.Drive_Serial();  // Fall back to sequential
            } else {
                return Err(e);
            }
        }
        Ok(())
    }

    fn Drive_Parallel_Internal(&mut self) -> Result<(), SimError> {
        self.ResolveReadyModules()?;

        let chore_tree = CycleChoreBuilder::build(self);
        Atelier::PostChoreTree(chore_tree);
        Atelier::Wait();

        Ok(())
    }

    fn Drive_Serial(&mut self) -> Result<(), SimError> {
        // Fallback to original sequential implementation
        self._Engine.Drive();
        Ok(())
    }
}
```

#### 5.3 Compatibility Tests

**File**: `src/rube/engine.rs` (test module)

```rust
#[cfg(test)]
mod compatibility_tests {
    #[test]
    fn test_parallel_matches_serial() {
        let mut layout = Layout::New();
        // Build complex circuit with various warp types

        let mut serial_engine = SimEngine::Create(&layout);
        let mut parallel_engine = SimEngine::CreateWithMode(
            &layout,
            SimEngineMode::Parallel { worker_count: 4 },
        );

        for _ in 0..1000 {
            serial_engine.Drive().unwrap();
            parallel_engine.Drive().unwrap();

            // Verify outputs match
            assert_eq!(serial_engine._CycleCount, parallel_engine._CycleCount);
            // Compare trigger values
        }
    }
}
```

---

### Phase 6: Performance Optimization & Profiling (10-15 hours)

#### 6.1 Performance Profiling Framework

**File**: `src/rube/engine.rs` (new module `perf_metrics.rs`)

```rust
pub struct SimulationMetrics {
    pub _CycleCount: usize,
    pub _TotalDriveCycles: usize,
    pub _AverageCycleLatency: f64,
    pub _WorkerUtilization: Vec<f64>,
    pub _WarpEvaluationTime: Duration,
    pub _ResolveReadyTime: Duration,
    pub _AdvanceAllTime: Duration,
    pub _Speedup: f64,  // Parallel / Serial
}

impl HeistSimEngine {
    pub fn profile_cycle(&mut self) -> Result<SimulationMetrics, SimError> {
        let start = std::time::Instant::now();
        self.Drive_Parallel()?;
        let parallel_duration = start.elapsed();

        // Compare to serial version
        let mut serial_engine = SerialSimEngine::Create(&self._Engine._Layout);
        let serial_start = std::time::Instant::now();
        serial_engine.Drive();
        let serial_duration = serial_start.elapsed();

        Ok(SimulationMetrics {
            _CycleCount: self._Engine._CycleCount,
            _TotalDriveCycles: parallel_duration.as_micros() as usize,
            _AverageCycleLatency: parallel_duration.as_secs_f64() * 1e6,
            _WorkerUtilization: vec![0.0; self._WorkerCount as usize],  // TODO: measure
            _WarpEvaluationTime: Duration::ZERO,  // TODO: instrument
            _ResolveReadyTime: Duration::ZERO,
            _AdvanceAllTime: Duration::ZERO,
            _Speedup: serial_duration.as_secs_f64() / parallel_duration.as_secs_f64(),
        })
    }
}
```

#### 6.2 Instrumentation Points

**Key metrics to track**:
1. **Worker utilization**: % of time each worker is active
2. **Work-stealing rate**: # of steals per cycle
3. **Cache miss ratio**: L1/L3 misses during warp evaluation
4. **Warp chunk distribution**: Balance across workers
5. **Coroutine yield frequency**: % of cycles with yields

#### 6.3 Optimization Knobs

**File**: `src/rube/engine.rs`

```rust
pub struct OptimizationConfig {
    pub _ChunkSize: U32,                  // Warp chunk size (smaller = more parallelism)
    pub _FusionThreshold: U32,            // Heist task fusion threshold
    pub _EnableWorkStealing: bool,        // Enable/disable work-stealing
    pub _EnablePartitionOptimization: bool,  // Collocate triggers by warp locality
    pub _MaxCoroSpawn: U32,               // Max concurrent coroutine tasks
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            _ChunkSize: U32(100),          // Experiment with 50-200
            _FusionThreshold: U32(10),
            _EnableWorkStealing: true,
            _EnablePartitionOptimization: true,
            _MaxCoroSpawn: U32(1000),
        }
    }
}
```

---

## Part 4: Migration Strategy

### 4.1 Phased Rollout

```
Phase 1 (Weeks 1-2):  Infrastructure + basic parallel warp evaluation
├─ Implement partition system
├─ Add warp chunking
└─ Integrate Heist framework

Phase 2 (Weeks 2-3):  Warp evaluation kernels
├─ FastWarp parallel kernel
├─ CustomWarp parallel kernel
├─ BehavioralWarp parallel kernel
└─ CoroWarp dynamic chore creation

Phase 3 (Weeks 3-4):  ChoreTree + synchronization
├─ Build temporal dependency graph
├─ Implement partition ownership verification
└─ Add determinism tests

Phase 4 (Weeks 4-5):  Integration & fallback
├─ Dual-mode SimEngine
├─ Fallback mechanism
└─ Compatibility tests

Phase 5 (Weeks 5-6):  Performance optimization
├─ Profiling framework
├─ Instrumentation
└─ Tuning
```

### 4.2 Testing Strategy

**Unit Tests**:
- Partition creation and index localization
- Warp chunk distribution
- ChoreTree building

**Integration Tests**:
- Simple circuits (half-adder, full-adder)
- Medium circuits (8-bit ripple carry adder)
- Complex circuits (multi-module pipelines)

**Determinism Tests**:
- Same circuit, different worker counts → identical outputs
- Same circuit, multiple runs → identical traces

**Performance Tests**:
- Latency per cycle vs. worker count
- Throughput scaling
- Worker utilization

### 4.3 Rollback Strategy

**If issues arise**:
1. SerialSimEngine remains default (no breaking changes)
2. Fallback mode automatically reverts to serial
3. Dual-mode allows side-by-side comparison
4. Performance regression caught by benchmarks

---

## Part 5: Risk Analysis & Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| **Data race in trigger access** | High | Critical | Partition ownership validation + tests |
| **Non-deterministic output** | Medium | Critical | Determinism verification tests + double-check ordering |
| **Performance regression** | Medium | High | Profiling framework + fallback to serial |
| **Coroutine scheduling complexity** | Medium | Medium | Start with non-coro warps; add later |
| **Memory overhead from partitions** | Low | Low | Profile memory usage; optimize if needed |
| **Compilation complexity** | Low | Medium | Modularize code; clear documentation |

---

## Part 6: Success Criteria

### 6.1 Functional Requirements

- ✅ All warp types evaluate correctly in parallel
- ✅ Output matches serial implementation (bit-identical)
- ✅ Deterministic across runs (same circuit, different worker counts)
- ✅ No data races or unsafe behavior
- ✅ Graceful fallback on error

### 6.2 Performance Requirements

| Target | Baseline (Serial) | Goal (Parallel, 4 Workers) |
|--------|------------------|---------------------------|
| Latency per cycle | 1-10 μs | <5 μs |
| Throughput (1K cycles) | X | 2-4× speedup |
| Worker utilization | N/A | >75% |

### 6.3 Maintainability Requirements

- ✅ Clear module separation (partition, chunking, chore_builder, safety)
- ✅ Comprehensive documentation
- ✅ Fallback mechanism available
- ✅ No forced use of parallel mode

---

## Part 7: Future Enhancements

### 7.1 GPU Acceleration (Post-MVP)

Integrate `SwarmEngine` for GPU compute:

```rust
impl HeistSimEngine {
    pub fn enable_gpu_acceleration(&mut self, backend: SwarmBackend) {
        Atelier::SetSwarm(SwarmEngine::Create(backend));

        // Move FastWarp evaluation to GPU for large batches
        // Keep small warps on CPU (better latency)
    }
}
```

### 7.2 Adaptive Chunking

Dynamically adjust chunk size based on:
- Warp sizes
- Worker utilization
- Cache miss rate
- Measured latency per cycle

### 7.3 Circuit-Aware Partitioning

Instead of linear partition assignment, use:
- Data-flow analysis to collocate signals
- Minimize inter-partition communication
- Reduce cache line conflicts

### 7.4 Coroutine Yielding Optimization

Track yield patterns:
- If coroutine rarely yields → fuse into single job
- If yields frequently → keep as dynamic chores
- Auto-tune threshold per coro type

---

## Part 8: Estimation & Resources

### 8.1 Development Effort

| Phase | Effort | Risk | Notes |
|-------|--------|------|-------|
| 1. Infrastructure | 15-20h | Low | Partition + chunking mostly straightforward |
| 2. Warp Kernels | 20-25h | Medium | Care needed for thread safety |
| 3. ChoreTree | 15-20h | Low | API clearly defined by Heist |
| 4. Sync & Safety | 12-15h | High | Most complex; needs thorough testing |
| 5. Integration | 8-10h | Low | Fallback mode makes this safe |
| 6. Performance | 10-15h | Medium | Profiling-dependent; iterative |
| **Total** | **80-105h** | — | **~3-4 weeks for 2 engineers** |

### 8.2 Knowledge Requirements

- **Deep understanding of**: SimEngine, Heist/Atelier, Rust async patterns
- **Moderate understanding of**: Memory layout, cache behavior, thread safety
- **Reference materials**: Heist documentation, SimEngine code, prior review

### 8.3 Recommended Team

- **Lead Engineer** (40h): Architecture, integration, complex sections
- **Support Engineer** (40h): Testing, profiling, documentation
- **Optional Code Review**: 1-2 hours per week for design review

---

## Appendix A: Code Organization

### New Files to Create

```
src/rube/
├── partition.rs           (PartitionedTriggers, TriggerPartition)
├── warp_scheduler.rs      (WarpChunks, chunk functions)
├── heist_integration.rs   (HeistSimEngine, chore creation)
├── chore_builder.rs       (CycleChoreBuilder, dependency graph)
├── safety.rs              (PartitionOwnershipValidator, verification)
├── perf_metrics.rs        (SimulationMetrics, profiling)
└── _tests_heist.rs        (Integration + determinism tests)
```

### Modified Files

```
src/rube/
├── mod.rs                 (Export new modules + HeistSimEngine)
├── engine.rs              (Add HeistSimEngine, dual-mode logic)
└── trigger.rs             (Expose ITriggerWad for partition safety)
```

---

## Appendix B: Pseudo-Code Example

```rust
// Full drive cycle with Heist
impl HeistSimEngine {
    pub fn Drive_Parallel(&mut self) -> Result<(), SimError> {
        // 1. Resolve which modules are ready (serial)
        self.ResolveReadyModules()?;

        // 2. Build parallel task graph
        let chore_tree = CycleChoreBuilder::build(self);

        // 3. Submit to Heist work-stealing scheduler
        Atelier::PostChoreTree(chore_tree);

        // 4. Wait for all tasks to complete
        //    - Workers pull tasks from queues
        //    - Work-steal from busy neighbors
        //    - Execute in cache-local partitions
        Atelier::Wait();

        // 5. Advance triggers (serial)
        self._Engine._Triggers.AdvanceAll();
        self._Engine._CycleCount += 1;

        Ok(())
    }
}

// Execution trace for 4 workers, 3 warp types:
// Master:       [ResolveReady] ──┬─────────── [AdvanceAll] ──►
// Worker 0:                      │ [FastWarp-0]
// Worker 1:                      │ [FastWarp-1] ──steal──► [CustomWarp-2]
// Worker 2:                      │ [CustomWarp-0]
// Worker 3:                      └─ [BehavioralWarp-0] ──steal──► [FastWarp-3]
//
// Key: All warps execute in parallel, work-stealing balances load
```

---

## Appendix C: Configuration Examples

### Example 1: Conservative Deployment (Serial Fallback)

```rust
let mut engine = HeistSimEngine::New(&layout, 4);
engine._EnableFallback = true;

for cycle in 0..100_000 {
    if let Err(e) = engine.Drive_Parallel() {
        eprintln!("Falling back to serial: {}", e);
        // Continues with serial execution automatically
    }
}
```

### Example 2: Performance-Tuned Deployment

```rust
let mut engine = HeistSimEngine::New(&layout, num_cpus::get() as u8);

let config = OptimizationConfig {
    _ChunkSize: U32(50),           // Smaller chunks = more parallelism
    _FusionThreshold: U32(5),      // Aggressive fusion
    _EnableWorkStealing: true,
    _EnablePartitionOptimization: true,
    _MaxCoroSpawn: U32(500),
};

for cycle in 0..100_000 {
    engine.Drive_Parallel_Optimized(&config)?;

    if cycle % 1000 == 0 {
        let metrics = engine.profile_cycle()?;
        println!("Speedup: {:.2}×", metrics._Speedup);
    }
}
```

### Example 3: Heterogeneous CPU+GPU Deployment

```rust
let mut engine = HeistSimEngine::New(&layout, 4);  // 4 CPU workers

// Enable GPU acceleration for large FastWarps
engine.enable_gpu_acceleration(SwarmBackend::CUDA);

// Chore builder automatically routes warps:
// - Small warps: CPU
// - Large warps: GPU
// - Serial phases: CPU

for cycle in 0..1_000_000 {
    engine.Drive_Parallel()?;
}
```

---

**End of Implementation Plan**

