// (c) Copyright 2026 Mikołaj Kuranowski
// SPDX-License-Identifier: MIT

use std::io::{Error, ErrorKind, Read, Result, Write};

pub const BINARY_FILE_MAGIC: &'static [u8; 8] = b"routx001";

/// BinarySerializer is a trait for graph-like objects which can serialize and deserialize
/// themselves to/from a custom routx binary format.
///
/// The format is as follows:
/// 1. File starts with 8 octets, "routx001", encoded in ASCII. This is the file magic.
/// 2. Then, 16 arbitrary padding octets are written.
/// 3. Then, for each node, the following 24 octets are written:
///   1. Node ID, as a little-endian int64,
///   2. Node OSM ID, as a little-endian int64,
///   3. Node latitude, as a little-endian float32,
///   4. Node longitude, as a little-endian float32.
/// 4. Then, a separator of 24 null bytes is written.
/// 5. Then, for each edge, the following 24 octets are written:
///   1. Source node ID, as a little-endian int64,
///   2. Destination node ID, as a little-endian int64,
///   3. Traversing cost, as a little-endian float32.
///   4. 4 arbitrary padding octets.
///
/// Zero node IDs are not allowed.
///
/// This format was designed to have the following properties:
/// 1. It can be read in chunks of 24 octets
/// 2. If the beginning of the file is aligned to at least 8 octets, then every number
///    is properly aligned.
/// 3. Little-Endian has been chosen, as that's what most common CPUs.
/// 4. A memory-mapped file could be simply cast to a `*const (i64, i64, f32, f32)`
pub trait BinarySerializer {
    fn read_bin<R: Read>(&mut self, r: R) -> Result<()>;
    fn write_bin<W: Write>(&self, w: W) -> Result<()>;
}

impl BinarySerializer for crate::Graph {
    fn read_bin<R: Read>(&mut self, mut r: R) -> Result<()> {
        let mut seen_header = false;
        let mut seen_nodes = false;
        let mut buf = [0_u8; 24];

        loop {
            match r.read_exact(&mut buf) {
                Ok(_) => {
                    if !seen_header {
                        if !buf.starts_with(BINARY_FILE_MAGIC) {
                            return Err(Error::new(
                                ErrorKind::InvalidData,
                                "invalid routx binary format magic",
                            ));
                        }
                        seen_header = true;
                    } else if !seen_nodes {
                        // Node, or separator
                        let id = i64::from_le_bytes(buf[0..8].try_into().unwrap());
                        if id != 0 {
                            let osm_id = i64::from_le_bytes(buf[8..16].try_into().unwrap());
                            let lat = f32::from_le_bytes(buf[16..20].try_into().unwrap());
                            let lon = f32::from_le_bytes(buf[20..24].try_into().unwrap());
                            self.set_node(crate::Node {
                                id,
                                osm_id,
                                lat,
                                lon,
                            });
                        } else {
                            seen_nodes = true;
                        }
                    } else {
                        // Way
                        let from = i64::from_le_bytes(buf[0..8].try_into().unwrap());
                        let to = i64::from_le_bytes(buf[8..16].try_into().unwrap());
                        let cost = f32::from_le_bytes(buf[16..20].try_into().unwrap());
                        self.set_edge(from, crate::Edge { to, cost });
                    }
                }
                Err(e) if e.kind() == ErrorKind::UnexpectedEof => return Ok(()),
                Err(e) => return Err(e),
            }
        }
    }

    fn write_bin<W: Write>(&self, mut w: W) -> Result<()> {
        // Header row
        w.write_all(BINARY_FILE_MAGIC)?;
        w.write_all(&[0_u8; 16])?;

        // Nodes
        for (_, (node, _)) in self.0.iter() {
            // NOTE: Graph prevents inserting nodes with id == 0
            w.write_all(&node.id.to_le_bytes())?;
            w.write_all(&node.osm_id.to_le_bytes())?;
            w.write_all(&node.lat.to_le_bytes())?;
            w.write_all(&node.lon.to_le_bytes())?;
        }

        // Separator
        w.write_all(&[0_u8; 24])?;

        // Edges
        for (_, (node, edges)) in self.0.iter() {
            for edge in edges {
                w.write_all(&node.id.to_le_bytes())?;
                w.write_all(&edge.to.to_le_bytes())?;
                w.write_all(&edge.cost.to_le_bytes())?;
                w.write_all(&[0_u8; 4])?;
            }
        }

        // Finalize
        w.flush()?;
        Ok(())
    }
}
