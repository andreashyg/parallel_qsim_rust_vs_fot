use ahash::HashMapExt;
use metis::{Graph, Idx};
use nohash_hasher::IntMap;
use std::collections::BTreeMap;
use tracing::info;

use crate::simulation::config::{EdgeWeight, MetisOptions, VertexWeight};
use crate::simulation::id::Id;
use crate::simulation::scenario::network::{Link, Network, Node};

/// Partitions the nodes of `network` into `num_parts` parts using METIS.
///
/// METIS partitions undirected graphs. Each node becomes a vertex. Each pair of nodes connected
/// by at least one link becomes one edge, whose weight is the sum of the weights of all links
/// between both nodes in either direction (see [`EdgeWeight`]), but at least 1. Loop links, whose
/// `from` and `to` node are the same, are ignored.
pub fn partition(
    network: &Network,
    num_parts: u32,
    options: &MetisOptions,
) -> IntMap<Id<Node>, Idx> {
    if num_parts <= 1 {
        let mut result = IntMap::new();
        for node in network.nodes() {
            result.insert(node.id.clone(), 0);
        }
        return result;
    }

    let mut metis_result: Vec<Idx> = vec![0x00; network.nodes().len()];

    let mut pos_by_node_id = IntMap::default();
    let mut node_id_by_pos = IntMap::default();
    for (i, node) in network.nodes().iter().enumerate() {
        pos_by_node_id.insert(node.id.internal(), i);
        node_id_by_pos.insert(i, node.id.internal());
    }

    info!("Converting network into Metis format");
    let metis_graph = to_metis_graph(network, options, &pos_by_node_id);

    let ncon = if options.vertex_weight.is_empty() {
        1
    } else {
        options.vertex_weight.len() as Idx
    };

    info!("Calling Metis Partitioning Library");
    let mut graph = Graph::new(
        ncon,
        num_parts as Idx,
        &metis_graph.xadj,
        &metis_graph.adjncy,
    )
    .expect("Failed to create graph")
    .set_option(metis::option::UFactor(options.ufactor() as Idx))
    .set_option(metis::option::Seed(4711))
    .set_option(metis::option::Contig(options.contiguous))
    .set_adjwgt(&metis_graph.adjwgt);

    if !metis_graph.vwgt.is_empty() {
        graph = graph.set_vwgt(&metis_graph.vwgt);
    }

    graph.part_kway(&mut metis_result).unwrap();

    let mut result = IntMap::default();
    for (pos, part) in metis_result.iter().enumerate() {
        let internal_node_id = node_id_by_pos
            .get(&pos)
            .expect("Something went wrong with node positions");
        let id = Id::get(*internal_node_id);
        result.insert(id, *part);
    }

    result
}

/// The network in the CSR format expected by METIS. Vertex `i` is the node at position `i` in
/// `pos_by_node_id`.
struct MetisGraph {
    //stores start positions of each node in the adjacency list
    xadj: Vec<Idx>,
    adjncy: Vec<Idx>,
    adjwgt: Vec<Idx>,
    vwgt: Vec<Idx>,
}

fn to_metis_graph(
    network: &Network,
    options: &MetisOptions,
    pos_by_node_id: &IntMap<u64, usize>,
) -> MetisGraph {
    let mut xadj: Vec<Idx> = Vec::from([0]);
    let mut adjncy: Vec<Idx> = Vec::new();
    let mut adjwgt: Vec<Idx> = Vec::new();
    let mut vwgt: Vec<Idx> = Vec::new();

    for node in network.nodes() {
        // Add vertex weights
        add_vwgt(network, options, &mut vwgt, node);

        // METIS expects an undirected graph without loops and without duplicate edges, in which
        // every edge u-v is also stored as v-u with the same weight. It does not check this, and
        // invalid input makes it access memory out of bounds. Links are directed, so all links
        // between two nodes, in either direction, are merged into one edge. The weight of each
        // link is truncated to an integer before summing, so the sum is the same from both ends.
        // Loop links do not connect two nodes and are dropped. The BTreeMap orders the neighbors
        // by position, which keeps the input to METIS reproducible.
        let own_pos = pos_by_node_id[&node.id.internal()];
        let mut neighbors: BTreeMap<usize, Idx> = BTreeMap::new();
        let out_neighbors = node.out_links.iter().map(|id| {
            let link = network.get_link(id);
            (&link.to, link)
        });
        let in_neighbors = node.in_links.iter().map(|id| {
            let link = network.get_link(id);
            (&link.from, link)
        });
        for (neighbor_id, link) in out_neighbors.chain(in_neighbors) {
            let neighbor_pos = pos_by_node_id[&neighbor_id.internal()];
            if neighbor_pos != own_pos {
                let weight = neighbors.entry(neighbor_pos).or_insert(0);
                *weight = weight.saturating_add(get_adjwgt(options, link) as Idx);
            }
        }

        for (neighbor_pos, weight) in neighbors {
            adjncy.push(neighbor_pos as Idx);
            // METIS requires positive edge weights, but links with a capacity below 1 yield 0.
            adjwgt.push(weight.max(1));
        }
        xadj.push(adjncy.len() as Idx);
    }

    MetisGraph {
        xadj,
        adjncy,
        adjwgt,
        vwgt,
    }
}

fn add_vwgt(network: &Network, options: &MetisOptions, vwgt: &mut Vec<Idx>, node: &Node) {
    for weight in options.vertex_weight.iter() {
        match weight {
            VertexWeight::InLinkCapacity => {
                vwgt.push(
                    node.in_links
                        .iter()
                        .map(|id| network.get_link(id).capacity as Idx)
                        .sum(),
                );
            }
            VertexWeight::InLinkCount => {
                vwgt.push(node.in_links.len() as Idx);
            }
            VertexWeight::Constant => {
                vwgt.push(1);
            }
            VertexWeight::PreComputed => {
                vwgt.push(node.cmp_weight as Idx);
            }
        }
    }
}

fn get_adjwgt(options: &MetisOptions, link: &Link) -> f64 {
    match options.edge_weight {
        EdgeWeight::Capacity => link.capacity,
        EdgeWeight::Constant => 1.,
    }
}

#[cfg(test)]
mod tests {
    use crate::simulation::config::{EdgeWeight, MetisOptions, PartitionMethod, VertexWeight};
    use crate::simulation::id::Id;
    use crate::simulation::network::metis_partitioning::{MetisGraph, partition, to_metis_graph};
    use crate::simulation::scenario::Coordinate;
    use crate::simulation::scenario::network::{Link, Network, Node};
    use macros::deterministic_id_test;
    use metis::Idx;
    use nohash_hasher::IntMap;
    use std::collections::BTreeMap;

    #[deterministic_id_test]
    fn simple_graph() {
        let mut net = Network::new();
        let from_id = Id::create("from");
        let to_id = Id::create("to");
        net.add_node(Node::new(from_id.clone(), Coordinate::new_2d(0., 0.), 0, 1));
        net.add_node(Node::new(to_id.clone(), Coordinate::new_2d(100., 0.), 0, 1));
        let link_id = Id::create("link");
        net.add_link(Link::new_with_default(
            link_id,
            net.get_node(&from_id),
            net.get_node(&to_id),
        ));

        for _n in 0..100 {
            let partition_result = partition(&net, 2, &MetisOptions::default());
            assert_ne!(partition_result[&from_id], partition_result[&to_id]);
        }
    }

    /// Covers all cases in which links do not map one-to-one to METIS edges: a one-way link, a
    /// pair of opposite links, parallel links, a loop link, an isolated node and capacities
    /// below 1.
    #[deterministic_id_test]
    fn metis_graph_merges_links_into_undirected_edges() {
        let mut net = Network::new();
        for (i, name) in ["a", "b", "c", "d", "e"].into_iter().enumerate() {
            let coord = Coordinate::new_2d(i as f64 * 100., 0.);
            net.add_node(Node::new(Id::create(name), coord, 0, 1));
        }
        for (name, from, to, capacity) in [
            ("ab", "a", "b", 100.),
            ("ad", "a", "d", 0.5),
            ("bc", "b", "c", 300.),
            ("cb", "c", "b", 200.),
            ("cd-1", "c", "d", 50.),
            ("cd-2", "c", "d", 0.4),
            ("dd", "d", "d", 10.),
        ] {
            let mut link = Link::new_with_default(
                Id::create(name),
                net.get_node(&Id::get_from_ext(from)),
                net.get_node(&Id::get_from_ext(to)),
            );
            link.capacity = capacity;
            net.add_link(link);
        }

        let capacity = MetisOptions::default()
            .set_edge_weight(EdgeWeight::Capacity)
            .add_vertex_weight(VertexWeight::InLinkCount);
        let graph = to_metis_graph(&net, &capacity, &positions(&net));
        assert_valid_metis_graph(&graph, 1);
        // Capacities are truncated per link before they are summed: 0.5 becomes 0 and is raised
        // to the minimum weight 1, 0.4 becomes 0 and adds nothing to the 50 of the parallel link.
        assert_eq!(
            BTreeMap::from([
                ("a", vec![("b", 100), ("d", 1)]),
                ("b", vec![("a", 100), ("c", 500)]),
                ("c", vec![("b", 500), ("d", 50)]),
                ("d", vec![("a", 1), ("c", 50)]),
                ("e", vec![]),
            ]),
            adjacency_by_name(&net, &graph)
        );

        let constant = MetisOptions::default().set_edge_weight(EdgeWeight::Constant);
        let graph = to_metis_graph(&net, &constant, &positions(&net));
        assert_valid_metis_graph(&graph, 0);
        assert_eq!(
            BTreeMap::from([
                ("a", vec![("b", 1), ("d", 1)]),
                ("b", vec![("a", 1), ("c", 2)]),
                ("c", vec![("b", 2), ("d", 2)]),
                ("d", vec![("a", 1), ("c", 2)]),
                ("e", vec![]),
            ]),
            adjacency_by_name(&net, &graph)
        );
    }

    #[deterministic_id_test]
    fn metis_graph_of_andorra_is_valid() {
        let network =
            Network::from_file("./assets/andorra-network.xml.gz", 1, &PartitionMethod::None);
        for edge_weight in [EdgeWeight::Capacity, EdgeWeight::Constant] {
            let options = MetisOptions::default()
                .set_edge_weight(edge_weight)
                .add_vertex_weight(VertexWeight::InLinkCapacity)
                .add_vertex_weight(VertexWeight::InLinkCount);
            let graph = to_metis_graph(&network, &options, &positions(&network));
            assert_valid_metis_graph(&graph, 2);
        }
    }

    /// Node positions in the same order as in [`partition`].
    fn positions(network: &Network) -> IntMap<u64, usize> {
        network
            .nodes()
            .iter()
            .enumerate()
            .map(|(i, node)| (node.id.internal(), i))
            .collect()
    }

    /// Neighbors and edge weights of each node by external id, independent of node positions.
    fn adjacency_by_name<'a>(
        network: &'a Network,
        graph: &MetisGraph,
    ) -> BTreeMap<&'a str, Vec<(&'a str, Idx)>> {
        let nodes = network.nodes();
        let name = |pos: usize| nodes[pos].id.external();
        (0..nodes.len())
            .map(|u| {
                let edges = graph.xadj[u] as usize..graph.xadj[u + 1] as usize;
                let mut neighbors: Vec<_> = edges
                    .map(|e| (name(graph.adjncy[e] as usize), graph.adjwgt[e]))
                    .collect();
                neighbors.sort();
                (name(u), neighbors)
            })
            .collect()
    }

    /// Checks the input requirements of METIS, similar to its `graphchk` program: an undirected
    /// graph without loops and without duplicate edges, in which every edge is stored in both
    /// directions with the same positive weight.
    fn assert_valid_metis_graph(graph: &MetisGraph, num_vertex_weights: usize) {
        let num_vertices = graph.xadj.len() - 1;
        assert_eq!(0, graph.xadj[0]);
        assert!(graph.xadj.windows(2).all(|w| w[0] <= w[1]));
        assert_eq!(graph.adjncy.len(), *graph.xadj.last().unwrap() as usize);
        assert_eq!(graph.adjncy.len(), graph.adjwgt.len());
        assert_eq!(num_vertex_weights * num_vertices, graph.vwgt.len());

        let mut edges = BTreeMap::new();
        for u in 0..num_vertices {
            for e in graph.xadj[u] as usize..graph.xadj[u + 1] as usize {
                let v = graph.adjncy[e] as usize;
                let weight = graph.adjwgt[e];
                assert!(v < num_vertices, "vertex {u} has neighbor {v} out of range");
                assert_ne!(u, v, "vertex {u} has a loop");
                assert!(weight > 0, "edge {u}-{v} has weight {weight}");
                assert!(
                    edges.insert((u, v), weight).is_none(),
                    "edge {u}-{v} is duplicated"
                );
            }
        }
        for (&(u, v), weight) in &edges {
            assert_eq!(
                Some(weight),
                edges.get(&(v, u)),
                "edge {u}-{v} has no reverse edge with the same weight"
            );
        }
    }

    #[deterministic_id_test]
    fn test_andorra_with_default() {
        let network = Network::from_file(
            "./assets/andorra-network.xml.gz",
            5,
            &PartitionMethod::Metis(MetisOptions::default().set_contiguous(false)),
        );
        println!("=== Default ===");
        let _node_count = node_count(&network);
        let _edge_count = edge_count(network);
    }

    #[deterministic_id_test]
    fn test_andorra_with_capacity() {
        let network = Network::from_file(
            "./assets/andorra-network.xml.gz",
            5,
            &PartitionMethod::Metis(
                MetisOptions::default()
                    .add_vertex_weight(VertexWeight::InLinkCapacity)
                    .set_imbalance_factor(0.)
                    .set_contiguous(false),
            ),
        );
        println!("=== Capacity ===");
        let _node_count = node_count(&network);
        let _edge_count = edge_count(network);
    }

    #[deterministic_id_test]
    fn test_andorra_with_inlinkcount() {
        let network = Network::from_file(
            "./assets/andorra-network.xml.gz",
            5,
            &PartitionMethod::Metis(
                MetisOptions::default()
                    .add_vertex_weight(VertexWeight::InLinkCount)
                    .set_imbalance_factor(0.)
                    .set_contiguous(false),
            ),
        );
        println!("=== InLinkCount ===");
        let _node_count = node_count(&network);
        let _edge_count = edge_count(network);
    }

    #[deterministic_id_test]
    fn test_andorra_with_inlinkcount_and_capacity() {
        let network = Network::from_file(
            "./assets/andorra-network.xml.gz",
            5,
            &PartitionMethod::Metis(
                MetisOptions::default()
                    .add_vertex_weight(VertexWeight::InLinkCapacity)
                    .add_vertex_weight(VertexWeight::InLinkCount)
                    .set_imbalance_factor(0.)
                    .set_contiguous(false),
            ),
        );
        println!("=== Capacity & InLinkCount ===");
        let _node_count = node_count(&network);
        let _edge_count = edge_count(network);
    }

    #[deterministic_id_test]
    fn test_andorra_with_vertex_constant() {
        let network = Network::from_file(
            "./assets/andorra-network.xml.gz",
            5,
            &PartitionMethod::Metis(
                MetisOptions::default()
                    .add_vertex_weight(VertexWeight::Constant)
                    .set_imbalance_factor(0.)
                    .set_contiguous(false),
            ),
        );
        println!("=== Constant Vertex ===");
        let _node_count = node_count(&network);
        let _edge_count = edge_count(network);
    }

    #[deterministic_id_test]
    fn test_andorra_with_vertex_constant_and_inlinkcount() {
        let network = Network::from_file(
            "./assets/andorra-network.xml.gz",
            5,
            &PartitionMethod::Metis(
                MetisOptions::default()
                    .add_vertex_weight(VertexWeight::Constant)
                    .add_vertex_weight(VertexWeight::InLinkCount)
                    .set_imbalance_factor(0.)
                    .set_iteration_number(100)
                    .set_contiguous(false),
            ),
        );
        println!("=== Constant Vertex & InLinkCount ===");
        let _node_count = node_count(&network);
        let _edge_count = edge_count(network);
    }

    fn node_count(network: &Network) -> BTreeMap<u32, usize> {
        let map = network.nodes().iter().map(|n| n.partition).fold(
            BTreeMap::<u32, usize>::new(),
            |mut m, x| {
                *m.entry(x).or_default() += 1;
                m
            },
        );
        println!("Node count per partition: {:?}", map);
        map
    }

    fn edge_count(network: Network) -> BTreeMap<u32, usize> {
        let map = network.links().iter().map(|l| l.partition).fold(
            BTreeMap::<u32, usize>::new(),
            |mut m, x| {
                *m.entry(x).or_default() += 1;
                m
            },
        );
        println!("Edge count per partition: {:?}", map);
        map
    }
}
