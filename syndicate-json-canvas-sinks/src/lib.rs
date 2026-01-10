use syndicate_json_canvas_lib::SyndicationFormat;

pub mod jj_sink;

// Re-export the main trait
pub use jj_sink::JjRepositorySink;

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
    /// Publish an item to the sink
    ///
    /// # Arguments
    /// * `item` - The content to syndicate
    /// * `dry_run` - If true, only log what would happen without actually publishing
    ///
    /// # Returns
    /// Ok(()) on success, or SinkError on failure
    fn publish(&mut self, item: &SyndicationFormat, dry_run: bool) -> Result<(), SinkError>;

    /// Returns the name of this sink for logging/debugging
    fn name(&self) -> &str;
}
