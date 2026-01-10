use std::collections::HashMap;

pub use jsoncanvas;
use jsoncanvas::{JsonCanvas, node::GenericNodeInfo, NodeId, EdgeId};

// Error type for the library
#[derive(Debug, thiserror::Error)]
pub enum SyndicationError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Invalid node: {0}")]
    InvalidNode(String),
}

// Simplified SyndicationFormat without lifetimes
#[derive(Debug, Clone)]
pub struct SyndicationFormat {
    pub id: NodeId,
    pub text: String,
    pub out_edge_ids: Vec<EdgeId>,
}

// Simplified adjacency types - just store IDs
#[derive(Clone, Debug)]
pub struct OutAdjacencies(pub Vec<(NodeId, EdgeId)>);

#[derive(Clone, Debug)]
pub struct InAdjacencies(pub Vec<(NodeId, EdgeId)>);

pub fn to_syndication_format<F, M>(
    canvas: JsonCanvas,
    filter: Option<F>,
    mapper: Option<M>,
) -> Vec<SyndicationFormat>
where
    F: Fn(&jsoncanvas::Node, &OutAdjacencies, &InAdjacencies) -> bool,
    M: Fn(&jsoncanvas::Node, &OutAdjacencies, &InAdjacencies) -> SyndicationFormat,
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
        .map(|(node_id, node)| {
            let out_edges = out_adjacency_map
                .get(node_id)
                .cloned()
                .unwrap_or_default();

            let in_edges = in_adjacency_map
                .get(node_id)
                .cloned()
                .unwrap_or_default();

            (node, OutAdjacencies(out_edges), InAdjacencies(in_edges))
        })
        .filter(|(node, out_edges, in_edges)| {
            if let Some(ref f) = filter {
                f(node, out_edges, in_edges)
            } else {
                default_node_filter(node, out_edges, in_edges)
            }
        })
        .map(|(node, out_edges, in_edges)| {
            if let Some(ref m) = mapper {
                m(node, &out_edges, &in_edges)
            } else {
                default_node_to_syndication_format_mapper(node, &out_edges, &in_edges)
            }
        })
        .collect()
}

pub fn default_node_filter(node: &jsoncanvas::Node, _: &OutAdjacencies, _: &InAdjacencies) -> bool {
    use jsoncanvas::color::{Color, PresetColor};

    match node {
        jsoncanvas::Node::Text(text_node) => {
            let text = text_node.text();
            if text.is_empty() {
                return false;
            }
        },
        _ => { return false }
    }

    if let Some(color) = node.color() {
        if *color != Color::Preset(PresetColor::Red) {
            return false;
        }
    } else {
        return false;
    };
    return true;
}

pub fn default_node_to_syndication_format_mapper(
    node: &jsoncanvas::Node,
    out_adjacencies: &OutAdjacencies,
    _in_adjacencies: &InAdjacencies
) -> SyndicationFormat {
    match node {
        jsoncanvas::Node::Text(text_node) => {
            // Extract edge IDs from out_adjacencies
            let out_edge_ids = out_adjacencies.0.iter()
                .map(|(_, edge_id)| edge_id.clone())
                .collect();

            SyndicationFormat {
                id: text_node.id().clone(),
                text: text_node.text().to_string(),
                out_edge_ids,
            }
        },
        _ => {
            // For non-text nodes, we shouldn't reach here since filter removes them
            // But if we do, use the generic node id
            SyndicationFormat {
                id: node.id().clone(),
                text: String::new(),
                out_edge_ids: Vec::new(),
            }
        }
    }
}

mod tests {
    // TODO: add a test for cyclic nodes
}
