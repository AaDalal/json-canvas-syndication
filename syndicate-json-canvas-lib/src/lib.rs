use std::collections::HashMap;

pub use jsoncanvas;
use jsoncanvas::{JsonCanvas, node::GenericNodeInfo};

pub struct SyndicationFormat<'a> {
    pub id: String,
    pub text: String,
    pub out_edges: Vec<&'a SyndicationFormat<'a>>,
}

type ResolvedEdge<'b> = (
    &'b jsoncanvas::Node,
    jsoncanvas::NodeId,
    &'b jsoncanvas::edge::Edge,
);
pub struct OutAdjacencies<'b>(Vec<ResolvedEdge<'b>>);
pub struct InAdjacencies<'b>(Vec<ResolvedEdge<'b>>);

pub fn to_syndication_format<'a, 'b, F, M>(
    canvas: JsonCanvas,
    filter: Option<F>,
    mapper: Option<M>,
) -> Vec<SyndicationFormat<'a>>
where
    F: Fn(&jsoncanvas::Node, OutAdjacencies<'b>, InAdjacencies<'b>) -> bool,
    M: Fn(&jsoncanvas::Node, OutAdjacencies<'b>, InAdjacencies<'b>) -> SyndicationFormat<'a>,
{
    let nodes = canvas.get_nodes();
    let edges = canvas.get_edges();

    type AdjacencyMap<'b> =
        HashMap<jsoncanvas::NodeId, Vec<(jsoncanvas::NodeId, &'b jsoncanvas::edge::Edge)>>;
    let (out_adjacency_map, in_adjacency_map): (AdjacencyMap, AdjacencyMap) = edges.iter().fold(
        (AdjacencyMap::new(), AdjacencyMap::new()),
        |(out_adjacency_map, in_adjacency_map), (edge_id, edge)| {
            let _: () = out_adjacency_map.get(edge.from_node()).map_or_else(
                || {
                    out_adjacency_map.insert(
                        edge.from_node().clone(),
                        vec![(edge.to_node().clone(), edge)],
                    );
                    return ();
                },
                |adjacencies| adjacencies.push((edge.to_node().clone(), edge)),
            );
            let _: () = out_adjacency_map.get(edge.to_node()).map_or_else(
                || {
                    out_adjacency_map.insert(
                        edge.to_node().clone(),
                        vec![(edge.from_node().clone(), edge)],
                    );
                    return ();
                },
                |adjacencies| adjacencies.push((edge.from_node().clone(), edge)),
            );
            (out_adjacency_map, in_adjacency_map)
        },
    );

    let filter = filter.unwrap_or(default_node_filter);
    let mapper = mapper.unwrap_or(default_mapper)

    nodes
        .iter()
        .map(|(node_id, node)| {
            let out_edges: Vec<ResolvedEdge> = out_adjacency_map
                .get(node_id)
                .unwrap_or_else(|| return &Vec::new())
                .iter()
                .map(|(adjacent_node_id, edge)| {
                    return (
                        nodes
                            .get(adjacent_node_id)
                            .expect("A NodeId should always correspond to a Node"),
                        adjacent_node_id.clone(),
                        *edge,
                    );
                })
                .collect();

            let in_edges: Vec<ResolvedEdge> = in_adjacency_map
                .get(node_id)
                .unwrap_or_else(|| return &Vec::new())
                .iter()
                .map(|(adjacent_node_id, edge)| {
                    return (
                        nodes
                            .get(adjacent_node_id)
                            .expect("A NodeId should always correspond to a Node"),
                        adjacent_node_id.clone(),
                        *edge,
                    );
                })
                .collect();
            (node, OutAdjacencies(out_edges), InAdjacencies(in_edges))
        })
        .filter(|(node, out_edges, in_edges)| {
            return filter(*node, *out_edges, *in_edges);
        })
        .map(|(node, out_edges, in_edges)| return mapper(node, out_edges, in_edges))
        .collect()
}

pub fn default_node_filter(node: &jsoncanvas::Node, _: OutAdjacencies, _: InAdjacencies) -> bool {
    use jsoncanvas::color::{Color, PresetColor};

    match node {
        &jsoncanvas::Node::Text(text_node) => {
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

pub fn default_node_to_syndication_format_mapper(node: &jsoncanvas::Node, out_adjacencies: OutAdjacencies, in_adjacencies: InAdjacencies) -> SyndicationFormat {
    return SyndicationFormat { id: (), text: (), out_edges: () }
}

mod tests {
    // TODO: add a test for cyclic nodes
}
