# Module Reference: `heist`

## 1. Overview & Purpose

The `heist` module is Kosh's **asynchronous workflow DAG orchestrator and work-stealing job runtime**. It provides:
1. **The `Atelier` Master Workspace**: Manages worker thread pools (`Maestro`), global job allocators, predecessor counter arrays (`_SzPreds: Buff<Atm<U16>>`), and successor routing tables (`_SuccIds: Buff<U16>`).
2. **The `Maestro` Worker**: Thread-local execution agent managing local job caches (`_JobCache`), run queues (`_RunQueue`), temporary enqueue queues (`_TempQueue`), and work-stealing from peer Maestros.
3. **Chore DAG DSL (`ChoreTree!`)**: Construct parallel (`|`) and sequential (`<`) execution graphs with automatic predecessor and successor resolution.
4. **Target Hardware Affinity (`ChoreTarget`)**: Directs individual chore tasks to host CPU worker threads, specific GPU backends (`Gpu(BackendKind)`), or auto-selected GPU compute (`GpuAuto`).

---

## 2. Architecture & Class Diagram

```mermaid
classDiagram
    class IAtelier~'a~ {
        <<trait>>
        +MainMaestro() &Maestro
        +Maestros() Arr~Maestro~
        +SetSwarm(swarm: SwarmEngine)
        +Swarm() Option~&SwarmEngine~
        +SetSucc(jobId: U16, succId: U16)
        +ConstructJob(maestroIdx, succId, job, docStr) U16
        +DoLaunch()
    }

    class Atelier~'a~ {
        +Atm~U32~ _SzSchedJob
        -_Maestros: Buff~Maestro~
        -_SzPreds: Buff~Atm~U16~~
        -_SuccIds: Buff~U16~
        -_FreeJobStash: Stash~U16~
        -_JobBuff: Buff~WorkPtr~
        -_Terminal: U16
        -_Swarm: Option~Arc~SwarmEngine~~
        +New(szMaestro: U32) Atelier
        +Terminal() U16
    }

    class IMaestro~'a~ {
        <<trait>>
        +Atelier() &Atelier
        +MaestroIndex() U32
        +Swarm() Option~&SwarmEngine~
        +CurSuccId() U16
        +SetCurSuccId(val)
        +ConstructJob(succId, job, docStr) U16
        +EnqueueJob(jobId)
        +ConstructEnqueArr(succId, buff, docStr) U16
        +FlushTempQueue()
        +PostChoreTree(node)
    }

    class Maestro~'a~ {
        -_Index: U32
        -_Atelier: *const Atelier
        +U32 _SzProcessed
        -_JobCache: Stash~U16~
        -_RunQueue: Stash~U16~
        -_RunQlock: Spinlock
        -_CurSuccId: Atm~U16~
        -_TempQueue: Stash~U16~
        +New(maestroInd) Maestro
        +FromWorker(worker) &Maestro
    }

    class IChore {
        <<trait>>
        +Target() ChoreTarget
        +DocStr() &'static str
    }

    class Chore {
        +&'static str _DocStr
        +ChoreTarget _Target
        -fn(&DynIWorker) _Closure
        +New(f) Chore
        +Cpu(doc, f) Chore
        +Gpu(doc, backend, f) Chore
        +GpuAuto(doc, f) Chore
        +DoWork(worker)
    }

    class ChoreTarget {
        <<enumeration>>
        Cpu
        Gpu(BackendKind)
        GpuAuto
    }

    class IChoreNode {
        <<trait>>
        +Post(maestro: &impl IMaestro, tails: &mut Stash~U16~) U16
    }

    class JobInfo {
        +U16 _JobId
        +U16 _SuccId
        +U16 _SzPred
        +&'static str _DocStr
        +New(atelier, jobId) JobInfo
    }

    class AtelierInfo {
        +Stash~JobInfo~ _HookedStash
        +Buff~U16~ _JobRefBuff
        +TraceJobs(atelier) AtelierInfo
    }

    IAtelier <|.. Atelier : implements
    IMaestro <|.. Maestro : implements
    IChore <|.. Chore : implements
    IChoreNode <|.. Chore : implements
    IChoreNode <|.. BinNode : implements
    Atelier *-- Maestro : owns
    Maestro o-- Atelier : references
    Atelier ..> AtelierInfo : generates trace
```

---

## 3. Work-Stealing & DAG Execution Flowchart

```mermaid
flowchart TD
    Start["Maestro::ExecuteLoop(maestroIdx)"] --> CheckPending{"_SzSchedJob > 0 ?"}
    CheckPending -- No --> Finish["Execution Loop Terminated"]
    CheckPending -- Yes --> HaveJob{"jobId != 0 ?"}

    HaveJob -- Yes --> ExecJob["Execute job: (job.func)(job.data, maestro)<br/>_SzProcessed += 1"]
    ExecJob --> FreeJob["atelier.FreeJob(maestroIdx, jobId)"]
    FreeJob --> CheckSucc{"succId != 0 ?"}

    CheckSucc -- Yes --> DecrPred["szPred = atelier.SzPred(succId).Add(-1)"]
    DecrPred --> Ready{"szPred == 1 ?<br/>(All precursors finished)"}
    Ready -- Yes --> SetNext["jobId = succId<br/>_SzSchedJob.Add(1)"]
    Ready -- No --> ClearJob["jobId = 0"]
    CheckSucc -- No --> ClearJob

    ClearJob --> DecrGlobal["_SzSchedJob.Add(-1)"]
    DecrGlobal --> CheckPending

    HaveJob -- No --> PopLocal["jobId = maestro.PopJob() (from local _RunQueue)"]
    PopLocal --> FoundLocal{"jobId != 0 ?"}
    FoundLocal -- Yes --> HaveJob
    FoundLocal -- No --> StealJob["jobId = atelier.GrabJob(maestroIdx, &mut stealSeed)<br/>(Knuth hash across peer maestros)"]
    StealJob --> FoundSteal{"jobId != 0 ?"}
    FoundSteal -- Yes --> HaveJob
    FoundSteal -- No --> YieldSpin["spin_loop() & yield_now()"]
    YieldSpin --> CheckPending
```

---

## 4. DAG Construction with `ChoreTree!`

The `ChoreTree!` macro translates pipeline expressions into DAG dependencies:
- **Sequence (`<`)**: `A < B` sets `A`'s successor to `B`. `B` will not execute until `A` decrements `B`'s predecessor counter to 0.
- **Parallel (`|`)**: `A | B` creates a parallel branch. Both `A` and `B` are posted to the run queue via a generated `EnqPar` distributor node.

```mermaid
flowchart LR
    subgraph ParallelPipeline ["Example: (A | B) < C < (D | E)"]
        Enq1["EnqPar (Distributor)"] --> JobA["Job A (CPU)"]
        Enq1 --> JobB["Job B (GPU)"]
        JobA --> JobC["Job C (Reducer)"]
        JobB --> JobC
        JobC --> Enq2["EnqPar (Distributor)"]
        Enq2 --> JobD["Job D (Output)"]
        Enq2 --> JobE["Job E (Telemetry)"]
    end
```

---

## 5. Struct Reference

### `Atelier<'a>` (implements `IAtelier<'a>`)
Workspace coordinator managing multi-threaded pipeline execution:
- `New<S: Into<U32>>(szMaestro: S) -> Self`: Initializes workspace with `szMaestro` worker threads and $2^{16}$ pre-allocated job descriptors.
- `MainMaestro(&self) -> &Maestro<'a>`: Returns reference to maestro 0 for DAG submission.
- `Maestros(&self) -> Arr<'a, Maestro<'a>>`: Returns array of all worker maestros.
- `SetSwarm(&mut self, swarm: SwarmEngine)`: Binds GPU/CPU compute engine to workspace.
- `Swarm(&self) -> Option<&SwarmEngine>`: Returns reference to bound Swarm compute engine if present.
- `SetSucc<J: Into<U16>, S: Into<U16>>(&self, jobId: J, succId: S)`: Sets `_SuccIds[jobId] = succId` and increments `_SzPreds[succId]` atomically.
- `ConstructJob<M: Into<U32>, S: Into<U16>>(&self, maestroIdx: M, succId: S, job: WorkPtr<'a>, docStr: &'static str) -> U16`: Allocates a unique `jobId` and registers its successor.
- `DoLaunch(&self)`: Spawns worker threads via `std::thread::scope` and runs the execution loop until all tasks complete.

### `Maestro<'a>` (implements `IMaestro<'a>`, `IWorker`)
Worker thread executor managing local queues and work-stealing:
- `New<I: Into<U32>>(maestroInd: I) -> Self`: Constructs worker instance.
- `FromWorker<'w>(worker: &'w DynIWorker<'_>) -> &'w Self`: Safe downcasting helper from dynamic worker.
- `Atelier(&self) -> &Atelier<'a>`: Returns reference to owning Atelier workspace.
- `MaestroIndex(&self) -> U32`: Returns assigned worker index.
- `Swarm(&self) -> Option<&SwarmEngine>`: Returns Swarm compute engine if bound.
- `CurSuccId(&self) -> U16`: Returns currently active successor ID.
- `SetCurSuccId<K: Into<U16>>(&self, val: K)`: Updates active successor ID.
- `ConstructJob<S: Into<U16>>(&self, succId: S, job: impl IntoWorkPtr<'a>, docStr: &'static str) -> U16`: Allocates job via Atelier.
- `EnqueueJob<J: Into<U16>>(&self, jobId: J)`: Pushes job to thread-local `_TempQueue`.
- `FlushTempQueue(&self)`: Flushes temporary queue into thread-safe `_RunQueue` and increments `_SzSchedJob`.
- `ConstructEnqueArr<S: Into<U16>>(&self, succId: S, buff: Buff<U16>, docStr: &'static str) -> U16`: Constructs parallel job distributor.
- `PostChoreTree<T: IChoreNode>(&self, node: &T)`: Recursively traverses and posts a `ChoreTree` into the scheduler.

### `Chore` (implements `IChore`, `IWork`, `IChoreNode`)
A runnable unit of work with documentation and hardware affinity:
- `New(f: fn(&DynIWorker<'_>)) -> Self`: Default CPU chore.
- `Cpu(docStr: &'static str, f: fn(&DynIWorker<'_>)) -> Self`: Explicit CPU chore.
- `Gpu(docStr: &'static str, backend: BackendKind, f: fn(&DynIWorker<'_>)) -> Self`: Chore bound to specific compute backend.
- `GpuAuto(docStr: &'static str, f: fn(&DynIWorker<'_>)) -> Self`: Chore dispatched to auto-selected compute device.
- `Target(&self) -> ChoreTarget`: Returns target execution affinity.
- `DocStr(&self) -> &'static str`: Returns documentation label.

---

## 6. The `ChoreTree!` Macro Syntax

```rust
use kosh::{ChoreTree, CpuChore, GpuAutoChore};
use kosh::heist::Atelier;
use kosh::silo::U32;

let atelier = Atelier::New(U32(4));
let maestro = atelier.MainMaestro();

// Define a DAG: Run Step1 and Step2 in parallel, then aggregate in Step3, then output in Step4
let pipeline = ChoreTree!(
    (
        CpuChore!("Step1", |w| { /* CPU processing */ })
        | GpuAutoChore!("Step2", |w| { /* GPU processing */ })
    )
    < CpuChore!("Step3", |w| { /* Aggregation */ })
    < CpuChore!("Step4", |w| { /* Finalize */ })
);

maestro.PostChoreTree(&pipeline);
atelier.DoLaunch();
```
