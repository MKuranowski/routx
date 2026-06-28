// (c) Copyright 2026 Mikołaj Kuranowski
// SPDX-License-Identifier: MIT

mod binary;

pub use binary::BinarySerializer;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Edge, Graph, Node};
    use std::io::Result;

    #[inline]
    fn simple_graph_fixture() -> Graph {
        //   200   200   200
        // 1─────2─────3─────4
        //       └─────5─────┘
        //         100    100
        Graph::from_iter(
            [
                Node {
                    id: 1,
                    osm_id: 1,
                    lat: 0.01,
                    lon: 0.01,
                },
                Node {
                    id: 2,
                    osm_id: 2,
                    lat: 0.02,
                    lon: 0.01,
                },
                Node {
                    id: 3,
                    osm_id: 3,
                    lat: 0.03,
                    lon: 0.01,
                },
                Node {
                    id: 4,
                    osm_id: 4,
                    lat: 0.04,
                    lon: 0.01,
                },
                Node {
                    id: 5,
                    osm_id: 5,
                    lat: 0.03,
                    lon: 0.00,
                },
            ],
            [
                (1, 2, 200.0),
                (2, 1, 200.0),
                (2, 3, 200.0),
                (2, 5, 100.0),
                (3, 2, 200.0),
                (3, 4, 200.0),
                (4, 3, 200.0),
                (4, 5, 100.0),
                (5, 2, 100.0),
                (5, 4, 100.0),
            ],
        )
    }

    fn graph_matches_simple(g: &Graph) {
        assert_eq!(g.len(), 5);

        assert_eq!(
            g.get_node(1),
            Some(Node {
                id: 1,
                osm_id: 1,
                lat: 0.01,
                lon: 0.01,
            })
        );
        assert_eq!(
            g.get_node(2),
            Some(Node {
                id: 2,
                osm_id: 2,
                lat: 0.02,
                lon: 0.01,
            })
        );
        assert_eq!(
            g.get_node(3),
            Some(Node {
                id: 3,
                osm_id: 3,
                lat: 0.03,
                lon: 0.01,
            })
        );
        assert_eq!(
            g.get_node(4),
            Some(Node {
                id: 4,
                osm_id: 4,
                lat: 0.04,
                lon: 0.01,
            })
        );
        assert_eq!(
            g.get_node(5),
            Some(Node {
                id: 5,
                osm_id: 5,
                lat: 0.03,
                lon: 0.00,
            })
        );

        assert_eq!(g.get_edges(1), &[Edge { to: 2, cost: 200.0 }]);
        assert_eq!(
            g.get_edges(2),
            &[
                Edge { to: 1, cost: 200.0 },
                Edge { to: 3, cost: 200.0 },
                Edge { to: 5, cost: 100.0 },
            ],
        );
        assert_eq!(
            g.get_edges(3),
            &[Edge { to: 2, cost: 200.0 }, Edge { to: 4, cost: 200.0 }],
        );
        assert_eq!(
            g.get_edges(4),
            &[Edge { to: 3, cost: 200.0 }, Edge { to: 5, cost: 100.0 }],
        );
        assert_eq!(
            g.get_edges(5),
            &[Edge { to: 2, cost: 100.0 }, Edge { to: 4, cost: 100.0 }],
        );
    }

    #[test]
    fn test_binary_round_trip() -> Result<()> {
        let mut data = Vec::new();
        simple_graph_fixture().write_bin(&mut data)?;

        let mut graph = Graph::new();
        graph.read_bin(data.as_slice())?;

        graph_matches_simple(&graph);
        Ok(())
    }
}
