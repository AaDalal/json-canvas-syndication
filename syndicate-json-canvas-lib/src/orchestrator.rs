use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;

use notify_debouncer_mini::{DebouncedEventKind, new_debouncer, notify::RecursiveMode};
use tracing::{debug, error, info};

use crate::jsoncanvas::JsonCanvas;
use crate::sink::SyndicationSink;
use crate::tracker::SyndicationTracker;
use crate::{default_process_node, to_syndication_format};

/// Validate that the path points to a .canvas file
pub fn validate_canvas_path(path: &Path) -> Result<(), &'static str> {
    if !path.is_file() {
        return Err("Provided path must be a file");
    }
    if path.extension().and_then(|s| s.to_str()) != Some("canvas") {
        return Err("Expect the extension to be .canvas");
    }
    Ok(())
}

/// Process the canvas file and publish only new items
pub fn process_canvas(
    canvas_path: &Path,
    sink: &mut impl SyndicationSink,
    tracker: &mut SyndicationTracker,
    dry_run: bool,
) {
    let content = match std::fs::read_to_string(canvas_path) {
        Ok(c) => c,
        Err(e) => {
            error!(error = %e, "Failed to read file");
            return;
        }
    };

    let canvas = match JsonCanvas::from_str(&content) {
        Ok(c) => c,
        Err(e) => {
            error!(error = %e, "Failed to parse canvas");
            return;
        }
    };

    let all_items = to_syndication_format(canvas, Some(default_process_node));
    let total_count = all_items.len();
    info!(total_items = total_count, "Found items matching filter");

    // Filter out already-published items
    let new_items: HashMap<_, _> = all_items
        .into_iter()
        .filter(|(node_id, _)| !tracker.is_published(node_id))
        .collect();

    let already_published = total_count - new_items.len();
    debug!(
        new_items = new_items.len(),
        already_published = already_published,
        "Filtered to new items only"
    );

    if new_items.is_empty() {
        info!("No new items to publish");
        return;
    }

    info!(
        new_items = new_items.len(),
        "Publishing new items"
    );

    // Collect node IDs before publishing (for tracking)
    let published_ids: Vec<_> = new_items.keys().cloned().collect();

    match sink.publish(&new_items, dry_run) {
        Ok(()) => {
            info!("Successfully published all items");

            // Mark as published (skip in dry-run mode)
            if !dry_run {
                if let Err(e) = tracker.mark_published(&published_ids) {
                    error!(error = %e, "Failed to save tracker");
                }
            }
        }
        Err(e) => error!(error = %e, "Failed to publish items"),
    }
}

/// Watch the canvas file and process changes
///
/// This function processes the canvas on startup, then watches for file changes
/// and re-processes when modifications are detected.
pub fn watch_and_process(
    canvas_path: &Path,
    mut sink: impl SyndicationSink,
    mut tracker: SyndicationTracker,
    dry_run: bool,
    debounce_duration: Duration,
) -> Result<(), Box<dyn Error>> {
    // Process on startup
    info!("Processing canvas file on startup...");
    process_canvas(canvas_path, &mut sink, &mut tracker, dry_run);

    // Setup file watcher
    let (tx, rx) = std::sync::mpsc::channel();
    let mut debouncer = new_debouncer(debounce_duration, tx)?;

    debouncer
        .watcher()
        .watch(canvas_path, RecursiveMode::NonRecursive)?;

    info!("Watching for file changes...");

    for res in rx {
        match res {
            Ok(events) => {
                for event in events {
                    if let DebouncedEventKind::Any = event.kind {
                        info!("File changed, processing...");
                        process_canvas(canvas_path, &mut sink, &mut tracker, dry_run);
                    }
                }
            }
            Err(error) => error!(error = ?error, "Watch error"),
        }
    }

    Ok(())
}
