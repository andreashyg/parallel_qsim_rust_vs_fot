//! Benchmark for the exchange of sync messages between partitions in every time step. Each
//! partition runs on its own thread and only exchanges empty messages with its neighbors, which is
//! what a simulation without agents does. Thus, the time per step is the lower bound that the
//! communication puts on a simulation step.
//!
//! The partitions are placed on a grid with `rows * cols = num_parts` cells, where `rows` is the
//! largest divisor of `num_parts` that is not larger than its square root. Each partition is a
//! neighbor of the partitions to its left, right, top, bottom, top right, and bottom left, i.e.,
//! the neighbor graph is a triangulated grid with at most 6 neighbors per partition, similar to a
//! planar partitioning of a road network.
//!
//! Run the benchmark in release mode:
//!
//! ```shell
//! cargo test -p matsim-rust --release --test messaging_benchmark -- --ignored --nocapture --test-threads=1
//! ```
//!
//! The partition counts can be set as a comma-separated list with `MATSIM_RUST_MSG_BENCH_PARTS`.
//! By default, the benchmark runs 2, 8, 32 and as many partitions as there are cores. The number
//! of measured steps can be set with `MATSIM_RUST_MSG_BENCH_STEPS`. Context switches and CPU time
//! are not measured here; run the test binary with `/usr/bin/time -l` (macOS) or
//! `/usr/bin/time -v` (Linux) for that.

use macros::deterministic_id_test;
use matsim_rust::simulation::messaging::messages::InternalSyncMessage;
use matsim_rust::simulation::messaging::sim_communication::SimCommunicator;
use matsim_rust::simulation::messaging::sim_communication::local_communicator::ChannelSimCommunicator;
use matsim_rust::simulation::time::Tick;
use nohash_hasher::{IntMap, IntSet};
use std::time::{Duration, Instant};

const PARTS_ENV: &str = "MATSIM_RUST_MSG_BENCH_PARTS";
const STEPS_ENV: &str = "MATSIM_RUST_MSG_BENCH_STEPS";
const DEFAULT_STEPS: u64 = 20_000;
const WARMUP_STEPS: u64 = 1_000;
const REPETITIONS: usize = 3;

#[deterministic_id_test(matsim_rust)]
#[ignore = "benchmark"]
fn exchange_empty_sync_messages() {
    let cores = std::thread::available_parallelism().unwrap().get() as u32;
    let parts: Vec<u32> = match std::env::var(PARTS_ENV) {
        Ok(parts) => parts
            .split(',')
            .map(|p| p.trim().parse().expect("Invalid partition count"))
            .collect(),
        Err(_) => {
            let mut parts = vec![2, 8, 32, cores];
            parts.sort_unstable();
            parts.dedup();
            parts
        }
    };
    let steps = std::env::var(STEPS_ENV)
        .map(|s| s.parse().expect("Invalid number of steps"))
        .unwrap_or(DEFAULT_STEPS);

    println!("cores: {cores}, steps: {steps}, repetitions: {REPETITIONS}");
    println!("| partitions | max neighbors | µs/step (min) | µs/step (median) | steps/s (min) |");
    println!("|---:|---:|---:|---:|---:|");
    for num_parts in parts {
        let neighbors = grid_neighbors(num_parts);
        let max_neighbors = neighbors.iter().map(Vec::len).max().unwrap();

        let mut durations: Vec<Duration> =
            (0..REPETITIONS).map(|_| run(&neighbors, steps)).collect();
        durations.sort_unstable();
        let min_us = durations[0].as_secs_f64() * 1e6 / steps as f64;
        let median_us = durations[REPETITIONS / 2].as_secs_f64() * 1e6 / steps as f64;
        println!(
            "| {num_parts} | {max_neighbors} | {min_us:.2} | {median_us:.2} | {:.0} |",
            1e6 / min_us
        );
    }
}

/// Runs `WARMUP_STEPS` and then `steps` steps on one thread per partition and returns the duration
/// of the measured steps.
fn run(neighbors: &[Vec<u32>], steps: u64) -> Duration {
    let comms = ChannelSimCommunicator::create_n_2_n(neighbors.len() as u32);

    std::thread::scope(|scope| {
        let handles: Vec<_> = comms
            .into_iter()
            .map(|comm| {
                let neighbors = &neighbors[comm.rank() as usize];
                scope.spawn(move || {
                    let mut early = Vec::new();
                    for step in 0..WARMUP_STEPS {
                        exchange(&comm, neighbors, Tick::new(step), &mut early);
                    }
                    comm.barrier();
                    let start = Instant::now();
                    for step in WARMUP_STEPS..WARMUP_STEPS + steps {
                        exchange(&comm, neighbors, Tick::new(step), &mut early);
                    }
                    comm.barrier();
                    let duration = start.elapsed();
                    assert!(comm.receive_pending().is_empty());
                    duration
                })
            })
            .collect();

        handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .max()
            .unwrap()
    })
}

/// Sends an empty message to every neighbor and waits for the messages of all neighbors, like
/// `NetMessageBroker::send_recv` does. `early` holds the senders of messages that arrived one step
/// early, because a neighbor was already done with the step. They are cached like in
/// `NetMessageBroker::in_messages`.
fn exchange(comm: &ChannelSimCommunicator, neighbors: &[u32], now: Tick, early: &mut Vec<u32>) {
    let rank = comm.rank();
    let messages: IntMap<u32, InternalSyncMessage> = neighbors
        .iter()
        .map(|&n| (n, InternalSyncMessage::new(now, rank, n)))
        .collect();
    let mut expected: IntSet<u32> = neighbors.iter().copied().collect();

    for from in early.drain(..) {
        expected.remove(&from);
    }
    comm.send_receive_vehicles(messages, &mut expected, now, |msg| {
        if msg.time() == now.next() {
            early.push(msg.from_process());
        } else {
            assert_eq!(now, msg.time());
        }
    });
}

/// Returns the neighbors of each partition on a triangulated grid, see the module documentation.
fn grid_neighbors(num_parts: u32) -> Vec<Vec<u32>> {
    let rows = (1..=num_parts)
        .take_while(|r| r * r <= num_parts)
        .filter(|r| num_parts.is_multiple_of(*r))
        .max()
        .unwrap();
    let cols = num_parts / rows;
    let offsets = [(0, -1), (0, 1), (-1, 0), (1, 0), (-1, 1), (1, -1)];

    (0..num_parts)
        .map(|rank| {
            let (row, col) = ((rank / cols) as i64, (rank % cols) as i64);
            offsets
                .iter()
                .map(|(dr, dc)| (row + dr, col + dc))
                .filter(|&(r, c)| 0 <= r && r < rows as i64 && 0 <= c && c < cols as i64)
                .map(|(r, c)| (r * cols as i64 + c) as u32)
                .collect()
        })
        .collect()
}
