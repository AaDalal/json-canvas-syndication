//! Syndication sink implementations.
//!
//! This crate provides concrete implementations of the [`SyndicationSink`] trait
//! for publishing content to various destinations:
//!
//! - [`JjRepositorySink`] - Publishes to a Jujutsu (jj) git repository
//! - [`TwitterSink`] - Publishes to Twitter/X via API v2
//!
//! The [`SyndicationSink`] trait and [`SinkError`] type are defined in
//! `syndicate-json-canvas-lib` and re-exported here for convenience.

pub mod jj_sink;
pub mod twitter_sink;

// Re-export sink implementations
pub use jj_sink::JjRepositorySink;
pub use twitter_sink::TwitterSink;

// Re-export trait and error from lib crate for convenience
pub use syndicate_json_canvas_lib::{SinkError, SyndicationSink};
