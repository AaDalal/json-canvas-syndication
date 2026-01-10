use std::error::Error;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;

use notify_debouncer_mini::{new_debouncer, DebouncedEventKind, notify::RecursiveMode};
use syndicate_json_canvas_lib::jsoncanvas::JsonCanvas;
use syndicate_json_canvas_lib::{to_syndication_format, default_process_node};

// Configuration: Update this path to point to your JSON Canvas file
const CANVAS_FILE_PATH: &str = "canvas.canvas";

// Debounce duration in milliseconds - waits this long after last change before processing
const DEBOUNCE_DURATION_MS: u64 = 500;

fn validate_canvas_path(path: &Path) -> Result<(), &str> {
    if !path.is_file() {
        return Err("Provided path must be a file");
    }
    if path.extension().and_then(|s| s.to_str()) != Some("canvas") {
        return Err("Expect the extension to be .canvas");
    }
    Ok(())
}

fn main() -> Result<(), Box<(dyn Error)>> {
    let canvas_path = PathBuf::from(CANVAS_FILE_PATH);
    validate_canvas_path(&canvas_path)?;
    println!("Watching canvas file: {} (debounce: {}ms)",
             canvas_path.display(), DEBOUNCE_DURATION_MS);

    let (tx, rx) = std::sync::mpsc::channel();

    // Create a debounced watcher
    let mut debouncer = new_debouncer(
        Duration::from_millis(DEBOUNCE_DURATION_MS),
        tx
    )?;

    debouncer.watcher().watch(&canvas_path, RecursiveMode::NonRecursive)?;

    for res in rx {
        match res {
            Ok(events) => {
                // Process debounced events
                for event in events {
                    match event.kind {
                        DebouncedEventKind::Any => {
                            println!("File changed, processing...");
                            // Read the file and parse it
                            match std::fs::read_to_string(&canvas_path) {
                                Ok(content) => {
                                    match JsonCanvas::from_str(&content) {
                                        Ok(canvas) => {
                                            let syndication_items = to_syndication_format(
                                                canvas,
                                                Some(default_process_node)
                                            );
                                            println!("Found {} items to syndicate", syndication_items.len());
                                            // TODO: actually syndicate the items
                                        }
                                        Err(e) => eprintln!("Failed to parse canvas: {}", e),
                                    }
                                }
                                Err(e) => eprintln!("Failed to read file: {}", e),
                            }
                        }
                        _ => {}
                    }
                }
            }
            Err(error) => {
                eprintln!("Watch error: {:?}", error);
            }
        }
    }

    Ok(())
}
