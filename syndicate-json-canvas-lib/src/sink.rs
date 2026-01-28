use std::collections::HashMap;
use crate::{SyndicationFormat, jsoncanvas::NodeId};

/// Error types for syndication sinks
#[derive(Debug, thiserror::Error)]
pub enum SinkError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Command execution failed: {0}")]
    CommandFailed(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// Trait for syndication sinks
///
/// Implementors can publish SyndicationFormat items to various destinations
/// (Twitter, git repositories, etc.)
pub trait SyndicationSink {
    /// Publish all items to the sink
    ///
    /// # Arguments
    /// * `items` - HashMap of NodeId to SyndicationFormat containing all items to syndicate
    /// * `dry_run` - If true, only log what would happen without actually publishing
    ///
    /// # Returns
    /// Ok(()) on success, or SinkError on failure
    ///
    /// # Notes
    /// Takes all items at once to enable computing slugs and creating cross-references between posts
    fn publish(&mut self, items: &HashMap<NodeId, SyndicationFormat>, dry_run: bool) -> Result<(), SinkError>;

    /// Returns the name of this sink. This name should not have spaces & be unique.
    ///
    /// # Examples
    ///
    /// - jj
    /// - twitter
    fn name(&self) -> &str;
}
