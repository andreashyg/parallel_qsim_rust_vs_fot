//! Benchmark of the ALT router as used by the ReRoute strategy. It is ignored by default. Run it in
//! release mode:
//!
//! ```shell
//! cargo test -p matsim-rust --release --lib routing::benchmark -- --ignored --nocapture --test-threads=1
//! ```
//!
//! It routes the car legs of all selected plans with the router and the scoring-based travel
//! disutility that the controller creates for network modes. By default, it uses the Berlin v6.4
//! network and the 0.1 % population from `assets/berlin-v6.4`. The travel times are synthetic, but
//! every car link has observed travel times, so that every link lookup goes through the same code
//! path as after the first iteration of a simulation run.
//!
//! Environment variables:
//! - `MATSIM_RUST_ROUTING_BENCH_NETWORK`, `MATSIM_RUST_ROUTING_BENCH_PLANS`: network and XML plans
//!   file, e.g. those of Berlin v7.1.
//! - `MATSIM_RUST_ROUTING_BENCH_IDS`: id store file loaded before the network, for `.binpb`
//!   networks.
//! - `MATSIM_RUST_ROUTING_BENCH_REPETITIONS`: number of timed passes over all requests
//!   (default 5).
//! - `MATSIM_RUST_ROUTING_BENCH_LIMIT`: only route the first requests, e.g. for profiling.
//! - `MATSIM_RUST_ROUTING_BENCH_THREADS`: additionally route all requests in parallel with this
//!   many threads.
//! - `MATSIM_RUST_ROUTING_BENCH_FREESPEED=1`: route with freespeed travel times.
//! - `MATSIM_RUST_ROUTING_BENCH_VERIFY=1`: compare the disutility of every route with Dijkstra.
//! - `MATSIM_RUST_ROUTING_BENCH_OUT`: write one line per request with the disutility, travel time
//!   and a hash of the path, so that the routes of two versions can be compared.

use crate::simulation::config::Config;
use crate::simulation::id;
use crate::simulation::id::Id;
use crate::simulation::io::xml::population::{IOPlanElement, IOPopulation};
use crate::simulation::replanning::routing::a_star::{
    Alt, Dijkstra, last_search_reached_and_settled,
};
use crate::simulation::replanning::routing::alt_landmark_data::AltLandmarkData;
use crate::simulation::replanning::routing::cost::ScoringBasedTravelTimeAndDisutility;
use crate::simulation::replanning::routing::graph::{Graph, find_dead_ends};
use crate::simulation::replanning::routing::least_cost_path_calculator::{
    LeastCostPath, LeastCostPathCalculator, LeastCostPathRequest, LeastCostPathRequestBuilder,
};
use crate::simulation::replanning::routing::network_converter::convert_network_for_mode;
use crate::simulation::replanning::routing::travel_time_calculator::{
    GlobalTravelTimeCalculator, PartitionTravelTimes,
};
use crate::simulation::scenario::network::{Link, Network};
use crate::simulation::time::SimTime;
use macros::deterministic_id_test;
use nohash_hasher::IntMap;
use rayon::prelude::*;
use std::fmt::Write as _;
use std::hint::black_box;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

const ENV_PREFIX: &str = "MATSIM_RUST_ROUTING_BENCH_";
const DEFAULT_NETWORK: &str = "./assets/berlin-v6.4/berlin-v6.4-network-with-pt.xml.gz";
const DEFAULT_PLANS: &str = "./assets/berlin-v6.4/berlin-v6.4-0.1pct.plans-filtered.xml.gz";
const MODE: &str = "car";
/// Travel time bins as configured by default (`travel_time_calculator.bin_size`) up to 36 h.
const BIN_SIZE: Duration = Duration::from_secs(900);
const MAX_TIME: Duration = Duration::from_secs(36 * 3600);

/// A car leg of a selected plan.
struct BenchRequest {
    from: Id<Link>,
    to: Id<Link>,
    departure_time: SimTime,
}

impl BenchRequest {
    fn lcp_request(&self) -> LeastCostPathRequest<'static> {
        LeastCostPathRequestBuilder::default()
            .from(self.from.clone())
            .to(self.to.clone())
            .departure_time(self.departure_time)
            .build()
            .unwrap()
    }
}

#[deterministic_id_test]
#[ignore = "benchmark, run in release mode with --ignored --nocapture"]
fn alt_routing_benchmark() {
    let network_path = env_path("NETWORK", DEFAULT_NETWORK);
    let plans_path = env_path("PLANS", DEFAULT_PLANS);
    if let Some(ids) = env("IDS") {
        id::load_from_file(&PathBuf::from(ids));
    }
    let repetitions = env("REPETITIONS").map_or(5, |r| r.parse().unwrap());
    let threads: Option<usize> = env("THREADS").map(|t| t.parse().unwrap());
    let freespeed = env("FREESPEED").is_some_and(|v| v == "1");
    let verify = env("VERIFY").is_some_and(|v| v == "1");

    let network = Arc::new(timed("network", || Network::from_file(&network_path)));
    let mode = Id::<String>::create(MODE);
    let mut requests = timed("plans", || car_legs(&plans_path, &network, &mode));
    if let Some(limit) = env("LIMIT") {
        requests.truncate(limit.parse().unwrap());
    }

    let global = Arc::new(GlobalTravelTimeCalculator::new(1, BIN_SIZE, MAX_TIME));
    if !freespeed {
        global.submit(0, 0, synthetic_congestion(&network, &mode));
    }
    let cost = Arc::new(ScoringBasedTravelTimeAndDisutility::new(
        &Config::default(),
        mode.clone(),
        global,
    ));

    // time the preprocessing steps separately; the router below repeats them
    let graph = timed("graph", || {
        convert_network_for_mode(network.clone(), Some(mode.clone()))
    });
    let landmarks = timed("landmarks", || {
        AltLandmarkData::from_graph(&graph, cost.as_ref()).unwrap()
    });
    println!(
        "BENCHMARK routing graph: {} nodes, {} links, {} landmarks, {} requests ({} travel times), {} rayon threads",
        graph.num_nodes(),
        graph.forward_link_ids().len(),
        landmarks.landmarks().len(),
        requests.len(),
        if freespeed { "freespeed" } else { "congested" },
        rayon::current_num_threads(),
    );
    println!(
        "BENCHMARK routing landmarks: {:?}",
        landmarks
            .landmarks()
            .iter()
            .map(|&idx| graph.get_node_id(idx).unwrap().external().to_string())
            .collect::<Vec<_>>()
    );
    let dead_ends = find_dead_ends(&graph).unwrap();
    let mut dead_end_ids: Vec<_> = dead_ends.iter().flatten().collect();
    let dead_end_nodes = dead_end_ids.len();
    dead_end_ids.sort_unstable();
    dead_end_ids.dedup();
    println!(
        "BENCHMARK routing dead ends: {dead_end_nodes} nodes in {} dead ends",
        dead_end_ids.len()
    );
    drop(landmarks);
    drop(graph);
    let router = Alt::new(
        network.clone(),
        Some(mode.clone()),
        cost.clone(),
        cost.clone(),
    )
    .unwrap();

    // untimed pass that collects the routes and the search statistics
    let mut routes = Vec::with_capacity(requests.len());
    let mut settled = Vec::with_capacity(requests.len());
    let mut reached = Vec::with_capacity(requests.len());
    for request in &requests {
        routes.push(router.calc_least_cost_path(request.lcp_request()));
        let (r, s) = last_search_reached_and_settled();
        reached.push(r);
        settled.push(s);
    }
    let without_path = routes.iter().filter(|r| r.is_none()).count();
    println!(
        "BENCHMARK routing nodes per request: settled mean {:.0}, median {}, p90 {}; reached mean {:.0}; {} requests without path",
        mean(&settled),
        percentile(&mut settled.clone(), 0.5),
        percentile(&mut settled.clone(), 0.9),
        mean(&reached),
        without_path,
    );
    println!(
        "BENCHMARK routing checksum: paths {:016x}, total disutility {:.6}, total links {}",
        routes_hash(&routes),
        routes
            .iter()
            .flatten()
            .map(|r| r.travel_disutility)
            .sum::<f64>(),
        routes.iter().flatten().map(|r| r.path.len()).sum::<usize>(),
    );

    let mut times = Vec::with_capacity(repetitions);
    for _ in 0..repetitions {
        let start = Instant::now();
        route_sequentially(&router, &requests);
        times.push(start.elapsed());
    }
    report_times("1 thread", &times, requests.len());

    if let Some(threads) = threads {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .unwrap();
        let mut times = Vec::with_capacity(repetitions);
        for _ in 0..repetitions + 1 {
            let start = Instant::now();
            pool.install(|| {
                requests.par_iter().for_each(|request| {
                    black_box(router.calc_least_cost_path(request.lcp_request()));
                })
            });
            times.push(start.elapsed());
        }
        // the first pass allocates the search buffers of the pool threads
        times.remove(0);
        report_times(&format!("{threads} threads"), &times, requests.len());
    }

    if let Some(out) = env("OUT") {
        let mut lines = String::new();
        for (request, route) in requests.iter().zip(&routes) {
            let (disutility, travel_time, links, hash) = match route {
                Some(r) => (
                    r.travel_disutility,
                    r.travel_time.as_nanos(),
                    r.path.len(),
                    path_hash(&r.path),
                ),
                None => (f64::NAN, 0, 0, 0),
            };
            writeln!(
                lines,
                "{} {} {} {disutility:?} {travel_time} {links} {hash:016x}",
                request.from.external(),
                request.to.external(),
                request.departure_time.as_secs(),
            )
            .unwrap();
        }
        std::fs::write(&out, lines).unwrap();
        println!("BENCHMARK routing wrote routes to {out}");
    }

    if verify {
        verify_with_dijkstra(&network, &mode, cost, &requests, &routes);
    }
}

/// Routes all requests on the current thread. Not inlined, so that profilers can restrict the
/// measurement to it, e.g. `valgrind --tool=callgrind --collect-atstart=no
/// --toggle-collect='*route_sequentially*'`.
#[inline(never)]
fn route_sequentially(router: &Alt, requests: &[BenchRequest]) {
    for request in requests {
        black_box(router.calc_least_cost_path(request.lcp_request()));
    }
}

/// Compares the disutility of the given ALT routes with the ones found by Dijkstra.
fn verify_with_dijkstra(
    network: &Arc<Network>,
    mode: &Id<String>,
    cost: Arc<ScoringBasedTravelTimeAndDisutility>,
    requests: &[BenchRequest],
    routes: &[Option<LeastCostPath>],
) {
    let dijkstra = Dijkstra::new(network.clone(), Some(mode.clone()), cost.clone(), cost).unwrap();
    let start = Instant::now();
    let dijkstra_routes: Vec<_> = requests
        .par_iter()
        .map(|request| dijkstra.calc_least_cost_path(request.lcp_request()))
        .collect();

    let mut more_expensive = 0;
    let mut same_path = 0;
    let mut max_relative_excess: f64 = 0.0;
    let mut excess_sum = 0.0;
    for (alt, dijkstra) in routes.iter().zip(&dijkstra_routes) {
        match (alt, dijkstra) {
            (Some(alt), Some(dijkstra)) => {
                let excess = alt.travel_disutility - dijkstra.travel_disutility;
                if excess > 1e-9 * dijkstra.travel_disutility.abs().max(1.0) {
                    more_expensive += 1;
                    excess_sum += excess;
                    max_relative_excess =
                        max_relative_excess.max(excess / dijkstra.travel_disutility);
                }
                if alt.path == dijkstra.path {
                    same_path += 1;
                }
            }
            (None, None) => same_path += 1,
            (alt, dijkstra) => panic!(
                "ALT and Dijkstra disagree on reachability: ALT {:?}, Dijkstra {:?}",
                alt.is_some(),
                dijkstra.is_some()
            ),
        }
    }
    println!(
        "BENCHMARK routing verification ({:.1}s): {more_expensive} of {} ALT routes more expensive than Dijkstra (sum of excess {excess_sum:.6}, max {:.4} %), {same_path} identical paths",
        start.elapsed().as_secs_f64(),
        routes.len(),
        max_relative_excess * 100.0,
    );
}

/// Reads the car legs with a route of all selected plans. Legs that start and end on the same link
/// are skipped, since `NetworkRoutingModule` does not route them.
fn car_legs(path: &PathBuf, network: &Network, mode: &Id<String>) -> Vec<BenchRequest> {
    let population = IOPopulation::from_file(path);
    let car_link = |external: &Option<String>| -> Option<Id<Link>> {
        let id = Id::<Link>::try_get_from_ext(external.as_deref()?)?;
        network
            .links_with_ids()
            .get(&id)?
            .modes
            .contains(mode)
            .then_some(id)
    };
    let mut requests = Vec::new();
    for person in &population.persons {
        let Some(plan) = person.plans.iter().find(|p| p.selected) else {
            continue;
        };
        for element in &plan.elements {
            let IOPlanElement::Leg(leg) = element else {
                continue;
            };
            let (Some(route), Some(dep_time)) = (&leg.route, &leg.dep_time) else {
                continue;
            };
            if leg.mode != MODE {
                continue;
            }
            let (Some(from), Some(to)) = (car_link(&route.start_link), car_link(&route.end_link))
            else {
                continue;
            };
            if from != to {
                requests.push(BenchRequest {
                    from,
                    to,
                    departure_time: SimTime::parse(dep_time).unwrap(),
                });
            }
        }
    }
    requests
}

/// Travel times of all links of `mode` that are up to twice the freespeed travel time in the
/// morning and afternoon peaks. They are never below the freespeed travel time, which the landmark
/// data is based on.
fn synthetic_congestion(network: &Network, mode: &Id<String>) -> PartitionTravelTimes {
    let num_bins = (MAX_TIME.as_secs() / BIN_SIZE.as_secs() + 1) as usize;
    let peak = |hour: f64, center: f64, half_width: f64| {
        (1.0 - (hour - center).abs() / half_width).max(0.0)
    };
    let mut times = IntMap::default();
    for link in network.links_with_ids().values() {
        if !link.modes.contains(mode) || !(link.freespeed.is_finite() && link.freespeed > 0.0) {
            continue;
        }
        let freespeed_time = link.length / link.freespeed;
        // a deterministic pseudo-random amplitude in [0.2, 1.0) per link
        let hash = link.id.internal().wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 32;
        let amplitude = 0.2 + 0.8 * (hash % 1000) as f64 / 1000.0;
        let link_times = (0..num_bins)
            .map(|bin| {
                let hour = (bin as f64 + 0.5) * BIN_SIZE.as_secs_f64() / 3600.0;
                let load = peak(hour, 8.0, 2.0) + peak(hour, 17.0, 2.5);
                Duration::from_secs_f64(freespeed_time * (1.0 + amplitude * load))
            })
            .collect();
        times.insert(link.id.clone(), link_times);
    }
    let mut result = IntMap::default();
    result.insert(mode.clone(), times);
    result
}

fn report_times(label: &str, times: &[Duration], requests: usize) {
    let mut secs: Vec<f64> = times.iter().map(Duration::as_secs_f64).collect();
    secs.sort_by(f64::total_cmp);
    let best = secs[0];
    let median = secs[secs.len() / 2];
    println!(
        "BENCHMARK routing {label}: best {best:.3}s, median {median:.3}s of {} passes, {:.0} requests/s, {:.1} us/request (best)",
        secs.len(),
        requests as f64 / best,
        best * 1e6 / requests as f64,
    );
}

/// FNV-1a over the internal ids of all links of all paths, with a marker for missing paths.
fn routes_hash(routes: &[Option<LeastCostPath>]) -> u64 {
    routes.iter().fold(FNV_OFFSET, |hash, route| {
        let value = route.as_ref().map_or(u64::MAX, |r| path_hash(&r.path));
        fnv(hash, value)
    })
}

fn path_hash(path: &[Id<Link>]) -> u64 {
    path.iter()
        .fold(FNV_OFFSET, |hash, link| fnv(hash, link.internal()))
}

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;

fn fnv(hash: u64, value: u64) -> u64 {
    value.to_le_bytes().iter().fold(hash, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    })
}

fn mean(values: &[usize]) -> f64 {
    values.iter().sum::<usize>() as f64 / values.len().max(1) as f64
}

fn percentile(values: &mut [usize], p: f64) -> usize {
    values.sort_unstable();
    values
        .get(((values.len() as f64 * p) as usize).min(values.len().saturating_sub(1)))
        .copied()
        .unwrap_or(0)
}

fn env(name: &str) -> Option<String> {
    std::env::var(format!("{ENV_PREFIX}{name}")).ok()
}

fn env_path(name: &str, default: &str) -> PathBuf {
    PathBuf::from(env(name).unwrap_or_else(|| default.to_string()))
}

fn timed<T>(label: &str, f: impl FnOnce() -> T) -> T {
    let start = Instant::now();
    let result = f();
    println!(
        "BENCHMARK routing setup {label}: {:.3}s",
        start.elapsed().as_secs_f64()
    );
    result
}
