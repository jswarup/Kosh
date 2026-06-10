# Heist Framework

The `heist` module is a high-performance, work-stealing task scheduling and dependency resolution framework designed for parallel execution of structured jobs in Kosh. It manages a pool of worker threads, schedules jobs with predecessor/successor dependencies, and supports dynamic, graph-based execution.

## Purpose & Design Philosophy
The primary purpose of the Heist implementation is:
1. **Unthreaded Development**: Enable developers to design, develop, and debug complex recursive algorithms (such as QuickSort) in a simple, sequential, unthreaded environment using the ZST `Worker`.
2. **Seamless Scaling**: Effortlessly transition the exact same algorithm code to a multi-threaded parallel model (using `Atelier` and its `Meister` orchestrator) to match the machine's hardware capabilities and performance requirements.

---

## Architecture & Core Components

Heist is built around four central concepts:

```mermaid
graph TD
    Atelier[Atelier: Central Scheduler & Job Storage]
    Maven[Maven: Worker Threads & Local Queues]
    Maestro[Maestro: Thread-Local Orchestrator]
    Chore[Chore: High-Level Dependency Graph]

    Atelier -->|spawns & manages| Maven
    Maven -->|executes jobs using| Maestro
    Chore -->|compiled into| Atelier
```

### 1. Atelier (The Scheduler)
Defined in [atelier.rs](../src/heist/atelier.rs), `Atelier` is the central coordination engine. It:
* Manages a fixed-size thread pool running multiple `Maven` workers.
* Stores scheduled jobs in a cache-friendly, pre-allocated array of type-erased `WorkPtr`s.
* Tracks predecessors (`_SzPreds`) and successor job IDs (`_SuccIds`) to execute task DAGs (Directed Acyclic Graphs). When a job completes, it decrements the predecessor count of its successor; once that count hits 1 (meaning all dependencies are done), the successor job is automatically enqueued.

### 2. Maven (The Worker)
Defined in [maven.rs](../src/heist/maven.rs), a `Maven` represents a worker thread. It has:
* A thread-local job stash (`_JobCache`) to recycle job IDs without global lock contention.
* A synchronized run queue (`_RunQueue`) protected by a `Spinlock`.
* Work-stealing capabilities: if a worker's run queue is empty, it attempts to steal work from other Mavens using a randomized steal seed.

### 3. Maestro (The Orchestrator)
Defined in [maestro.rs](../src/heist/maestro.rs), `Maestro` is the thread-local context wrapper that implements the `IWorker` trait. 
* Every job is executed inside a `Maestro` context.
* Jobs can use `Maestro` to dynamically spawn sub-tasks, construct successor dependencies (`ConstructJob`), or enqueue bulk work (`ConstructEnqueueBulk`).

### 4. Chore & ChoreTree (Dependency Graph)
Defined in [chore.rs](../src/heist/chore.rs), `Chore` represents a unit of work that can be structured into a dependent tree (`dyn Bud<Chore>`) using the `ChoreTree!` macro.
* **Operators**:
  * `a | b`: Parallel execution (OR dependency).
  * `a < b`: Sequencing (a runs before b).
* When a chore tree is posted (`budTree.Post(&maestro)`), it compiles the tree into `WorkPtr`s with correct successor chains, and schedules them onto the `Atelier`.

---

## Example Usage

### 1. Basic Inline Job Construction
Jobs can be created directly by passing closures to `ConstructJob`:

```rust
let atelier = Atelier::New(U32(4)); // Create Atelier with 4 worker threads
let meister = atelier.Meister();
let mut jobId = U16(0);

// Define job 1
jobId = meister.ConstructJob(
    jobId, // Succesor dependency (0 means no successor)
    |_worker: &dyn IWorker| {
        println!("Job 1 executed!");
    },
);

// Define job 2 (Job 1 will only run after Job 2 completes)
jobId = meister.ConstructJob(
    jobId, // Job 1 is successor
    |_worker: &dyn IWorker| {
        println!("Job 2 executed!");
    },
);

// Enqueue starting job (Job 2)
meister.EnqueueJob(&mut jobId);
drop(meister);

// Launch execution
atelier.DoLaunch();
```

### 2. Graph-Based Dependency Resolution via `ChoreTree!`
You can construct complex tree-structured execution flows using the macro:

```rust
use crate::heist::chore::Chore;

let a = Chore::New(|_| { print!("A "); });
let b = Chore::New(|_| { print!("B "); });
let c = Chore::New(|_| { print!("C "); });

// c runs before both b and a
let budTree = crate::ChoreTree!(
    c < (b | a)
);

let atelier = Atelier::New(U32(4));
let meister = atelier.Meister();

budTree.Post(&meister);
drop(meister);

atelier.DoLaunch(); // Will print C A B (or C B A)
```

### 3. QuickSorter 2-Way Execution (Threaded vs. Unthreaded)

A concrete application of the framework's versatility is `QuickSorter` (defined on `Arr` in [arr.rs](../src/silo/arr.rs)). It produces a job closure that can be run either sequentially on a single thread or in parallel using the work-stealing thread pool:

#### A. Threaded Execution (Work-Stealing)
Using `Atelier` and a `Meister` context, the sorting tasks are scheduled dynamically across multiple worker threads to enable multi-threaded execution based on machine capability and performance:

```rust
let mut buff = Buff::Create(U32(100), |_| U32(rand::random::<u32>() % 128));
let quickSorter = buff.Arr().QuickSorter(|a, b| a > b);

let atelier = Atelier::New(U32(4)); // Spawns 4 worker threads
{
    let meister = atelier.Meister();
    let mut jobId = meister.ConstructJob(atelier.Terminal(), quickSorter);
    meister.EnqueueJob(&mut jobId);
}
atelier.DoLaunch(); // Runs quicksort in parallel
```

#### B. Unthreaded Execution (Sequential)
Using a synchronous ZST `Worker` instance, the same `quickSorter` job executes sequentially and immediately on the caller's main thread, enabling simple algorithm development and debugging in an unthreaded environment:

```rust
let mut buff = Buff::Create(U32(100), |_| U32(rand::random::<u32>() % 128));
let quickSorter = buff.Arr().QuickSorter(|a, b| a > b);

let worker = Worker::New();
quickSorter(&worker); // Runs quicksort sequentially on the current thread
```

