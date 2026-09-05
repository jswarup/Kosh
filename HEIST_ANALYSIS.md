# Heist Module Analysis - Work-Stealing Scheduler Framework

## 1. Overview: Purpose in Kosh Architecture

**Heist** is a **multi-threaded work-stealing scheduler** that manages computational work across multiple worker threads (Maestros). Its primary purposes are:

- **Heterogeneous compute execution**: Coordinate CPU and GPU work (via SwarmEngine) in a unified framework
- **Dynamic task scheduling**: Support both CPU and GPU computation targets with automatic load balancing
- **Work-stealing parallelism**: Enable efficient multi-threaded execution with work-stealing load balancing
- **Compositional task graphs**: Support hierarchical task composition via ChoreTree structures
- **Coroutine support**: Integrate with the Stalks coroutine framework for suspendable work units

The framework is designed for **scalable computational engines** (like SimEngine) that need to balance work across heterogeneous accelerators.

---

## 2. Atelier: The Work-Stealing Scheduler

### Architecture

**Atelier** (French for "workshop") is the **global scheduler** that manages the entire execution ecosystem:

```rust
pub struct Atelier<'a> {
    _SzSchedJob: Atm<U32>,           // Count of jobs in flight (atomic)
    _Maestros: Buff<Maestro<'a>>,    // Array of worker threads
    _SzPreds: Buff<Atm<U16>>,        // Predecessor count per job
    _SuccIds: Buff<U16>,             // Successor job IDs
    _FreeJobLock: Spinlock,          // Lock for job allocation
    _FreeJobStash: Stash<U16>,       // Pool of free job IDs
    _JobBuff: Buff<WorkPtr<'a>>,     // Job function pointers
    _JobDocBuff: Buff<&'static str>, // Job documentation strings
    _Terminal: U16,                  // Sentinel job ID
    _FusionThres: U32,               // Threshold for task fusion
    _Swarm: Option<Arc<SwarmEngine>>,// GPU acceleration runtime
    _WorkerThreads: Buff<OnceLock<thread::Thread>>,
}
```

### Key Features

1. **Global Static Singleton**: Uses `OnceLock` for thread-safe global access
   ```rust
   static GLOBAL_ATELIER: OnceLock<Atelier<'static>> = OnceLock::new();
   ```

2. **Initialization**: Must be explicitly initialized with worker count
   ```rust
   Atelier::Init(szMaestro)  // Initialize with N worker threads
   ```

3. **Job ID Allocation**: Manages a pool of 4096 pre-allocated job slots (U32::_16Sz = 4096)
   - Uses spinlock-protected freelists for thread-safe allocation
   - Each Maestro caches free job IDs locally to minimize lock contention

4. **Heterogeneous Compute Integration**: Optional SwarmEngine for GPU execution
   ```rust
   atelier.SetSwarm(SwarmEngine::Auto())  // Attach GPU runtime
   ```

### Public API

```rust
pub trait IAtelier<'a> {
    fn MainMaestro(&self) -> &Maestro<'a>;
    fn Maestros(&self) -> Arr<'a, Maestro<'a>>;
    fn FusionThres(&self) -> U32;
    fn SetFusionThres<T: Into<U32>>(&mut self, val: T);
    fn SetSwarm(&mut self, swarm: SwarmEngine);
    fn Swarm(&self) -> Option<&SwarmEngine>;
    fn SetSucc<J: Into<U16>, S: Into<U16>>(&self, jobId: J, succId: S);
    fn ConstructJob<M: Into<U32>, S: Into<U16>>(&self, maestroIdx: M, succId: S,
                                                 job: WorkPtr<'a>, docStr: &'static str) -> U16;
    fn DoLaunch(&self);
}
```

### Execution Model

- **Synchronous**: `Atelier::Post()` or `Atelier::PostChoreTree()` - Submit work
- **Blocking Wait**: `Atelier::Wait()` or `Atelier::Pump()` - Execute until all work done
- **Parallel Launch**: `DoLaunch()` - Coordinate all workers and wait for completion

---

## 3. Maestro: Worker Thread Coordinator

### Purpose

**Maestro** (Spanish for "master/conductor") represents a **single worker thread** in the work-stealing scheduler. Each Maestro:
- Maintains a local run queue for jobs
- Steals work from other Maestros when idle
- Participates in successor tracking and dependency management
- Communicates with the Atelier coordinator

### Structure

```rust
pub struct Maestro<'a> {
    _Index: U32,                              // Worker thread ID (0 = main)
    _Atelier: *const Atelier<'a>,             // Reference to scheduler
    _SzProcessed: U32,                        // Processed job count
    _JobCache: Stash<U16>,                    // Local free job ID cache (256 slots)
    _RunQueue: Stash<U16>,                    // Local job queue (1024 slots)
    _RunQlock: Spinlock,                      // Lock for run queue
    _CurSuccId: Atm<U16>,                     // Current successor job ID
    _TempQueue: Stash<U16>,                   // Temp queue for flushing (64 slots)
}
```

### Thread-Local State

Uses Rust's `thread_local!` for current thread's Maestro index:
```rust
thread_local! {
    static CURRENT_MAESTRO_INDEX: Cell<U32> = Cell::new(U32::_0);
}
```

### Key Operations

```rust
pub trait IMaestro<'a>: IWorker {
    fn Atelier(&self) -> &Atelier<'a>;
    fn MaestroIndex(&self) -> U32;
    fn Swarm(&self) -> Option<&SwarmEngine>;
    fn CurSuccId(&self) -> U16;
    fn SetCurSuccId<K: Into<U16>>(&self, val: K);
    fn ConstructJob<S: Into<U16>>(&self, succId: S, job: impl IntoWorkPtr<'a>,
                                   docStr: &'static str) -> U16;
    fn EnqueueJob<J: Into<U16>>(&self, jobId: J);
    fn FlushTempQueue(&self);
    fn PostChoreTree<T: IChoreNode + 'a>(&self, node: &T);
}
```

### Job Queue Management

1. **TempQueue**: Intermediate queue (avoids lock contention)
   - Push jobs into temp queue
   - Flush temp queue to run queue under lock

2. **RunQueue**: Protected by spinlock, used for work stealing

3. **JobCache**: Local free job ID pool
   - Fast path: allocate from cache without locks
   - Slow path: import free jobs from global pool

---

## 4. Choretree: Hierarchical Task Composition

### Core Components

#### **Chore**: Atomic Work Unit
```rust
pub struct Chore {
    pub _DocStr: &'static str,
    pub _Target: ChoreTarget,      // Cpu, Gpu, GpuAuto
    pub _Weight: U32,               // Estimated work amount
    _Closure: fn(&DynIWorker<'_>), // Function to execute
}
```

#### **ChoreTarget**: Execution Affinity
```rust
pub enum ChoreTarget {
    Cpu,                           // Execute on CPU via Maestro
    Gpu(BackendKind),              // Execute on specific GPU backend
    GpuAuto,                        // Let SwarmEngine choose
}
```

#### **IChoreNode**: Composable Task Interface
```rust
pub trait IChoreNode: Copy + Send + Sync {
    fn Weight(&self) -> U32 { U32(1) }
    fn Post<'a, M: IMaestro<'a>>(&self, maestro: &M, tails: &mut Stash<U16>) -> U16;
    fn Exec(&self, worker: &DynIWorker<'_>);
}
```

### Task Composition Patterns

#### **Parallel Execution** (`|` operator)
Multiple independent tasks run in parallel:
```rust
ChoreTree!(choreA | choreB | choreC)
```
- Creates an "EnqueueArr" job that enqueues all heads
- All tasks execute concurrently
- Tails register as terminal successors

#### **Sequential Execution** (`<` operator)
Tasks run one after another:
```rust
ChoreTree!(choreA < choreB < choreC)
```
- A's successors are set to B's head
- B's successors are set to C's head
- Ensures strict ordering

#### **Mixed Composition**
Complex DAGs using both operators:
```rust
ChoreTree!(
    (choreA < choreB) | (choreC | choreD) < choreE
)
```

### Binary Tree Representation

Uses `BinNode<L, R>` with `BinOp` to represent task DAGs:
```rust
pub struct BinNode<L, R> {
    pub _Left: L,
    pub _Right: R,
    pub _Op: BinOp,  // Bor (parallel) or Less (sequential)
}
```

### Task Fusion Optimization

When total weight ≤ `FusionThres`:
```rust
if self.Weight() <= maestro.Atelier().FusionThres() {
    // Execute entire subtree synchronously in one job
    let fused = FusedChore { _Node: *self };
    return maestro.ConstructJob(U16(0), fused, "FusedBinNode");
}
```

**Benefits**:
- Reduces job allocation overhead
- Improves cache locality
- Minimizes lock contention
- Default threshold: 2, configurable via `WithFusionThres()`

---

## 5. SpawnQuell Pattern: Distributed Data Processing

### Architecture

**SpawnQuell** enables efficient processing of arrays across workers:

```rust
pub struct SpawnQuellNode<'a, T> {
    pub _Data: Arr<'a, T>,
    pub _Target: ChoreTarget,
    pub _DocStr: &'static str,
    pub _ItemWeight: U32,
    pub _SpawnFn: fn(Arr<'a, T>, &DynIWorker<'_>),  // Process chunk
    pub _QuellFn: fn(Arr<'a, T>, &DynIWorker<'_>),  // Finalization
}
```

### Execution Flow

1. **Quell Setup**: Create finalization job (registered as tail)
2. **Spawn Phase**: Distribute data chunks to workers
   - If total weight ≤ fusionThres: single job
   - Else: adaptive chunking based on worker count
3. **ChunkWork Execution**: Workers process disjoint data ranges
4. **QuellWork**: Final reduction/aggregation

### Example: Data Parallelism
```rust
let spawnQuell = CpuSpawnQuell!(
    buff.Arr(),
    |chunk, _w| {  // Spawn: process each chunk
        chunk.USeg().Traverse(|i| {
            let val = *chunk.At(i);
            chunk.SetAt(i, &(val * 2));
        });
    },
    |all, _w| {    // Quell: finalization
        println!("Processed {} items!", all.Size());
    }
);
```

---

## 6. Work-Stealing Implementation

### Algorithm Overview

The scheduler implements **work-stealing load balancing**:

```rust
fn ExecuteLoop<M: Into<U32>>(&self, maestroIdx: M) {
    let mIdx = maestroIdx.into();
    let maestro = self._Maestros.Arr().MutAt(mIdx);
    let mut jobId = U16(0);
    let mut stealSeed = mIdx.AsU32();

    while self._SzSchedJob.Load(Ordering::Acquire) != 0 {
        while jobId != 0 {
            // Execute current job
            let job = *self._JobBuff.Arr().At(jobId);
            (job._Func)(job._Data, maestro);

            // Check successor availability
            let succId = maestro.CurSuccId();
            if succId != 0 {
                let szPred: U16 = self.SzPred(succId).Add(-U16(1));
                if szPred == U16(1) {
                    // All predecessors done, execute successor
                    jobId = succId;
                    self._SzSchedJob.Add(U32(1));
                } else {
                    jobId = U16::_0;
                }
            } else {
                jobId = U16::_0;
            }

            self._SzSchedJob.Add(-U32(1));
        }

        // Pop from local queue
        jobId = maestro.PopJob();

        // If empty, steal from another Maestro
        if jobId == 0 {
            jobId = self.GrabJob(mIdx, &mut stealSeed);
        }

        if jobId == 0 {
            spin_loop();  // Brief spin before yielding
            yield_now();
        }
    }
}
```

### Key Characteristics

1. **Local-First**: Each worker checks its own queue first
2. **Randomized Stealing**: Use Knuth hash to randomize victim selection
   ```rust
   stealSeed = stealSeed.wrapping_mul(2654435761u32).wrapping_add(1u32);
   ```

3. **Dependency Tracking**: Predecessor counters prevent early execution
   - Each job tracks how many predecessors must complete
   - When counter reaches 1, successor is eligible
   - Atomic compare-and-swap ensures race-free updates

4. **Efficient Memory**: Pre-allocated 4096 job slots
   - Avoids dynamic allocation overhead
   - Enables fast ID-based lookups
   - Thread-local caching reduces lock contention

### Contention Reduction

- **TempQueue**: Separate staging area for job queueing
- **JobCache**: Local free job ID pools per worker
- **Spinlock + Yield**: Brief spinning before thread park/yield

---

## 7. Coroutine Integration: CoroChore

### Suspension Points

**CoroChore** wraps Stalks coroutines for suspendable work:

```rust
pub struct CoroChore {
    pub _DocStr: &'static str,
    pub _Target: ChoreTarget,
    pub _Weight: U32,
    pub _Closure: fn(CoroYielder<'_, WorkerFatPtr, ()>, WorkerFatPtr),
}
```

### WorkerFatPtr: Coroutine Context

```rust
#[derive(Copy, Clone)]
pub struct WorkerFatPtr {
    pub _Ptr: *const DynIWorker<'static>,
}
```

Transmutes worker lifetime to `'static` for passing through coroutine channels.

### Execution Model

```rust
fn CoroJobFunc(dataPtr: *mut (), worker: &DynIWorker<'_>) {
    unsafe {
        let mut owned = Box::from_raw(dataPtr as *mut CoroWork);
        let workerPtr = WorkerFatPtr {
            _Ptr: transmute::<&DynIWorker<'_>, *const DynIWorker<'static>>(worker)
        };

        match owned._Coro.Resume(workerPtr) {
            CoroRes::Yield(_) => {
                // Coroutine suspended - resubmit as new job
                let newData = Box::into_raw(owned) as *mut ();
                let job = WorkPtr::New(newData, CoroJobFunc);
                worker.PostJob(job);
            }
            CoroRes::Done(_) => {
                // Coroutine finished
            }
        }
    }
}
```

### Usage Pattern

```rust
let coroChore = CoroChore::New(|yielder, worker| {
    // Perform initial work

    yielder.Yield(());  // Suspend, requeue as new job

    // Resume after all other work done
});
```

---

## 8. Dependency Management

### Predecessor Counting

Each job tracks dependencies through atomic counters:

```rust
_SzPreds: Buff<Atm<U16>>  // One counter per job

// When establishing dependency:
fn SetSucc<J: Into<U16>, S: Into<U16>>(&self, jobId: J, succId: S) {
    let j = jobId.into();
    let s = succId.into();
    self._SuccIds.Arr().SetAt(j, &s);
    self.SzPred(s).Add(1);  // Increment successor's predecessor count
}

// When job completes:
let szPred: U16 = self.SzPred(succId).Add(-U16(1));
if szPred == U16(1) {
    // All predecessors satisfied, successor ready to run
    jobId = succId;
}
```

### DAG Execution Guarantees

- **Topological Ordering**: Successors never execute before all predecessors
- **No Spurious Execution**: Counter-based readiness check
- **Atomic Updates**: Lock-free dependency management

---

## 9. Public API and Key Traits

### Atelier API

| Method | Purpose |
|--------|---------|
| `Init(szMaestro)` | Initialize global scheduler with N workers |
| `Get()` | Get global Atelier instance |
| `Post(job)` | Submit single job |
| `PostChoreTree(tree)` | Submit hierarchical task |
| `Wait()` / `Pump()` | Execute until completion |
| `DoLaunch()` | Parallel launch with thread coordination |

### Maestro API

| Method | Purpose |
|--------|---------|
| `ConstructJob()` | Create job from work closure |
| `EnqueueJob()` | Queue job for execution |
| `PostChoreTree()` | Submit task composition |
| `PostJob()` | Quick job submission (IWorker impl) |
| `CurSuccId()` / `SetCurSuccId()` | Manage successor context |

### Core Traits

```rust
pub trait IChore: IWork + IChoreNode {
    fn Target(&self) -> ChoreTarget;
    fn DocStr(&self) -> &'static str;
}

pub trait IChoreNode: Copy + Send + Sync {
    fn Weight(&self) -> U32;
    fn Post<'a, M: IMaestro<'a>>(&self, maestro: &M, tails: &mut Stash<U16>) -> U16;
    fn Exec(&self, worker: &DynIWorker<'_>);
}

pub trait IWorker {
    fn PostJob(&self, job: WorkPtr<'_>);
}
```

### Macros for Convenience

```rust
// Create chores
Chore!(closure)                    // Default chore
CpuChore!("doc", closure)          // CPU-targeted
GpuChore!("doc", backend, closure) // GPU-specific
GpuAutoChore!("doc", closure)      // Auto GPU selection

// Compose trees
ChoreTree!(a < b | c)              // Syntax sugar for BinNode

// Data parallel
CpuSpawnQuell!(arr, spawn_fn, quell_fn)
```

---

## 10. Integration with SimEngine: Key Patterns

### Pattern 1: Direct Job Submission
```rust
Atelier::Post(|worker| {
    // Synchronous computation
});
Atelier::Wait();  // Block until done
```

### Pattern 2: Structured Task Graphs
```rust
let engine = SwarmEngine::Auto();
let mut atelier = Atelier::new(4);
atelier.SetSwarm(engine);

let pipeline = ChoreTree!(
    (cpuPrep | dataLoad) < gpuCompute < cpuFinalize
);

atelier.MainMaestro().PostChoreTree(&pipeline);
atelier.DoLaunch();
```

### Pattern 3: Data-Parallel Workloads
```rust
let chunk_work = CpuSpawnQuell!(
    data.Arr(),
    |chunk, worker| {
        // Process chunk across workers
    },
    |result, worker| {
        // Aggregate results
    }
);
```

### Pattern 4: Heterogeneous Pipelines
```rust
let cpuStage1 = CpuChore!("prep", prep_fn);
let gpuStage = GpuAutoChore!("compute", gpu_fn);
let cpuStage3 = CpuChore!("finalize", final_fn);

ChoreTree!(cpuStage1 < gpuStage < cpuStage3)
```

---

## 11. Performance Characteristics

### Advantages

1. **Work-Stealing Load Balancing**
   - No central bottleneck
   - Automatic load distribution
   - O(1) local operations

2. **Task Fusion**
   - Reduces job allocation overhead
   - Improves CPU cache locality
   - Configurable via `FusionThres`

3. **Heterogeneous Support**
   - Unified API for CPU/GPU work
   - Target selection via `ChoreTarget`
   - Transparent SwarmEngine integration

4. **Coroutine Support**
   - Suspension/resumption without blocking threads
   - Efficient I/O patterns
   - Non-blocking synchronization

### Scalability Considerations

1. **Memory**: 4096 pre-allocated job slots (fixed overhead)
2. **Contention**: Job queues protected by spinlocks
   - TempQueue reduces lock frequency
   - JobCache minimizes global allocations
   - Randomized stealing spreads stealing attempts

3. **Work-Stealing**: Knuth hash provides good distribution
   - Seed per-worker to avoid collisions
   - Wrap-around arithmetic for fast hashing

### Configuration Tuning

| Parameter | Effect |
|-----------|--------|
| `FusionThres` | ↓ Increases parallelism, ↑ Reduces task overhead |
| Worker count | Scales parallelism; choose based on core count |
| `SwarmEngine` | Enables GPU acceleration for heterogeneous work |

---

## 12. Key Design Insights

### 1. **Global Singleton Pattern**
- Thread-safe initialization via `OnceLock`
- Eliminates lifetime complexity
- Simplifies API for callers

### 2. **ID-Based Job Representation**
- Jobs represented by 16-bit IDs, not pointers
- Enables static allocation without fragmentation
- Fast array-based lookups

### 3. **Dependency Tracking Without Explicit Barriers**
- Predecessor counters implicit in job graph
- No separate synchronization primitives needed
- Compiler-friendly (no `await`, `join`, etc.)

### 4. **Composable Task Trees**
- Binary tree representation supports arbitrary DAGs
- Recursive `Post()` method naturally builds job graph
- Weight calculation enables adaptive strategies

### 5. **Coroutine-Aware Execution**
- Suspendable work units via `CoroChore`
- Fat pointer pattern for context passing
- Automatic requeuing on resumption

---

## 13. Example: Complete Heterogeneous Pipeline

```rust
#[test]
fn TestHeistSwarmHeterogeneousPipeline() {
    // Initialize scheduler with GPU support
    let engine = SwarmEngine::Auto();
    let mut atelier = Atelier::New(4);
    atelier.SetSwarm(engine);

    // Define stages
    let cpuPrep = CpuChore!("PrepData", |worker| {
        println!("CPU prep stage");
    });

    let gpuCompute = GpuAutoChore!("Compute", |worker| {
        let maestro = Maestro::FromWorker(worker);
        if let Some(swarm) = maestro.Swarm() {
            let input = Buff![1.0f32, 2.0, 3.0];
            let result = swarm.RunDouble(&input).unwrap();
            println!("GPU result: {:?}", result);
        }
    });

    let cpuFinalize = CpuChore!("Finalize", |worker| {
        println!("CPU finalize stage");
    });

    // Compose pipeline: prep -> compute -> finalize
    let pipeline = ChoreTree!(cpuPrep < gpuCompute < cpuFinalize);

    // Execute with all 4 workers
    atelier.MainMaestro().PostChoreTree(&pipeline);
    atelier.DoLaunch();

    // Result: work automatically distributed across 4 workers
}
```

---

## 14. Metadata and Tracking: AtelierInfo

**AtelierInfo** provides runtime diagnostics:

```rust
pub struct AtelierInfo {
    pub _HookedStash: Stash<JobInfo>,  // In-flight jobs
    pub _JobRefBuff: Buff<U16>,        // Free job references
}

#[derive(Clone, Copy)]
pub struct JobInfo {
    pub _JobId: U16,
    pub _SuccId: U16,
    pub _SzPred: U16,
    pub _DocStr: &'static str,
}
```

### Introspection API

```rust
AtelierInfo::TraceJobs(atelier)       // Capture current state
AtelierInfo::FetchConnectedJobs()     // Get job graph slice
```

Useful for debugging, profiling, and understanding work distribution.

---

## Summary Table

| Component | Purpose | Key Type |
|-----------|---------|----------|
| **Atelier** | Global work-stealing scheduler | `struct Atelier<'a>` |
| **Maestro** | Worker thread coordinator | `struct Maestro<'a>` |
| **Chore** | Atomic work unit | `struct Chore` |
| **ChoreTree** | Hierarchical task composition | `impl IChoreNode` |
| **SpawnQuell** | Data-parallel distributed work | `struct SpawnQuellNode<'a, T>` |
| **CoroChore** | Suspendable work with coroutines | `struct CoroChore` |
| **AtelierInfo** | Runtime diagnostics & metadata | `struct AtelierInfo` |

---

## Conclusion

**Heist** is a sophisticated, production-grade work-stealing scheduler designed for heterogeneous computational workloads. Its key innovations include:

1. **Unified API** for CPU and GPU work
2. **Efficient task composition** via binary tree DSL
3. **Lock-free dependency management** using atomic counters
4. **Coroutine integration** for advanced control flow
5. **Work-stealing load balancing** with cache-friendly optimizations

For SimEngine integration, Heist provides the infrastructure for:
- Distributing simulation workloads across cores
- GPU acceleration of compute-intensive stages
- Structured async/suspendable computational kernels
- Efficient load balancing in heterogeneous systems
