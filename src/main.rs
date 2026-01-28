use std::error::Error;
use std::path::PathBuf;
use std::time::Duration;

use syndicate_json_canvas_lib::{
    SyndicationSink, SyndicationTracker, validate_canvas_path, watch_and_process,
};
use syndicate_json_canvas_sinks::JjRepositorySink;
use tracing::info;
use tracing_subscriber::EnvFilter;

// ===== CONFIGURATION =====
const DRY_RUN: bool = false;
const DEBOUNCE_DURATION_MS: u64 = 500;

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging (DEBUG when dry-run, INFO otherwise)
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(if DRY_RUN { "debug" } else { "info" }))
        .with_line_number(true)
        .with_file(true)
        .with_target(false)
        .init();

    // ===== Canvas Configuration =====
    let canvas_path = PathBuf::from("/Users/aadalal/Documents/scratchpad/Thoughts.canvas");
    validate_canvas_path(&canvas_path)?;

    // ===== Sink Configuration =====
    let sink = JjRepositorySink::new(
        "/Users/aadalal/dev/aadalal.github.io/",
        "main",
        "origin",
        "_tiny_thoughts",
    )?;

    // ===== Tracker Setup =====
    let tracker = SyndicationTracker::new(&canvas_path, sink.name())?;

    // ===== Logging =====
    info!(
        canvas_file = %canvas_path.display(),
        debounce_ms = DEBOUNCE_DURATION_MS,
        sink = sink.name(),
        dry_run = DRY_RUN,
        "Starting syndication"
    );

    // ===== Run =====
    watch_and_process(
        &canvas_path,
        sink,
        tracker,
        DRY_RUN,
        Duration::from_millis(DEBOUNCE_DURATION_MS),
    )
}
