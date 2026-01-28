//! Core library for syndicating JSON Canvas content.
//!
//! This crate provides:
//!
//! - **Data types**: [`SyndicationFormat`] for representing content to syndicate
//! - **Canvas processing**: [`to_syndication_format`] and [`default_process_node`] for
//!   parsing and filtering JSON Canvas files
//! - **Sink trait**: [`SyndicationSink`] trait that sink implementations must implement
//! - **Tracker**: [`SyndicationTracker`] for deduplication (tracking published nodes)
//! - **Orchestration**: [`watch_and_process`] for file watching and publishing workflow
//!
//! Sink implementations (JJ repository, Twitter) are in the `syndicate-json-canvas-sinks` crate.

use std::collections::HashMap;

pub use jsoncanvas;
use jsoncanvas::{JsonCanvas, node::GenericNodeInfo, NodeId, EdgeId};

pub mod sink;
pub mod tracker;
pub mod orchestrator;

// Re-exports for convenient access
pub use sink::{SinkError, SyndicationSink};
pub use tracker::SyndicationTracker;
pub use orchestrator::{validate_canvas_path, process_canvas, watch_and_process};

// Simplified SyndicationFormat without lifetimes
#[derive(Debug, Clone)]
pub struct SyndicationFormat {
    pub id: NodeId,
    pub text: String,
    pub in_neighbor_ids: Vec<NodeId>,  // nodes that point TO this node
    pub out_neighbor_ids: Vec<NodeId>, // nodes that this node points TO
}

// Simplified adjacency types - just store IDs
#[derive(Clone, Debug)]
pub struct OutAdjacencies(pub Vec<(NodeId, EdgeId)>);

#[derive(Clone, Debug)]
pub struct InAdjacencies(pub Vec<(NodeId, EdgeId)>);

pub fn to_syndication_format<F>(
    canvas: JsonCanvas,
    process_node: Option<F>,
) -> HashMap<NodeId, SyndicationFormat>
where
    F: Fn(&jsoncanvas::Node, &OutAdjacencies, &InAdjacencies) -> Option<SyndicationFormat>,
{
    let nodes = canvas.get_nodes();
    let edges = canvas.get_edges();

    type AdjacencyMap = HashMap<NodeId, Vec<(NodeId, EdgeId)>>;

    let mut out_adjacency_map = AdjacencyMap::new();
    let mut in_adjacency_map = AdjacencyMap::new();

    for (edge_id, edge) in edges.iter() {
        out_adjacency_map
            .entry(edge.from_node().clone())
            .or_insert_with(Vec::new)
            .push((edge.to_node().clone(), edge_id.clone()));

        in_adjacency_map
            .entry(edge.to_node().clone())
            .or_insert_with(Vec::new)
            .push((edge.from_node().clone(), edge_id.clone()));
    }

    nodes
        .iter()
        .filter_map(|(node_id, node)| {
            let out_edges = out_adjacency_map
                .get(node_id)
                .cloned()
                .unwrap_or_default();

            let in_edges = in_adjacency_map
                .get(node_id)
                .cloned()
                .unwrap_or_default();

            let out_adjacencies = OutAdjacencies(out_edges);
            let in_adjacencies = InAdjacencies(in_edges);

            let item = if let Some(ref processor) = process_node {
                processor(node, &out_adjacencies, &in_adjacencies)?
            } else {
                default_process_node(node, &out_adjacencies, &in_adjacencies)?
            };

            Some((item.id.clone(), item))
        })
        .collect()
}

/// Default node processor that filters for red text nodes and converts them to SyndicationFormat
/// Returns Some(SyndicationFormat) if the node should be syndicated, None otherwise
pub fn default_process_node(
    node: &jsoncanvas::Node,
    _out_adjacencies: &OutAdjacencies,
    _in_adjacencies: &InAdjacencies
) -> Option<SyndicationFormat> {
    use jsoncanvas::color::{Color, PresetColor};

    // Filter: Only process Text nodes
    let text_node = match node {
        jsoncanvas::Node::Text(text_node) => text_node,
        _ => return None,
    };

    // Filter: Skip empty text
    if text_node.text().is_empty() {
        return None;
    }

    // Filter: Only red colored nodes
    match node.color() {
        Some(color) if *color == Color::Preset(PresetColor::Red) => {},
        _ => return None,
    }

    // Map: Convert to SyndicationFormat
    let in_neighbor_ids = _in_adjacencies.0.iter()
        .map(|(node_id, _)| node_id.clone())
        .collect();

    let out_neighbor_ids = _out_adjacencies.0.iter()
        .map(|(node_id, _)| node_id.clone())
        .collect();

    Some(SyndicationFormat {
        id: text_node.id().clone(),
        text: text_node.text().to_string(),
        in_neighbor_ids,
        out_neighbor_ids,
    })
}

mod tests {
    // TODO: add a test for cyclic nodes
}
