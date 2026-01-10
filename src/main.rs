use std::error::Error;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use notify::{self, Watcher};
use syndicate_json_canvas_lib::jsoncanvas::JsonCanvas;
use syndicate_json_canvas_lib::{to_syndication_format, default_process_node};

// Configuration: Update this path to point to your JSON Canvas file
const CANVAS_FILE_PATH: &str = "canvas.canvas";

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
    println!("Watching canvas file: {}", canvas_path.display());

    let (tx, rx) = std::sync::mpsc::channel::<notify::Result<notify::Event>>();
    let mut watcher = notify::recommended_watcher(tx)?;
    watcher.watch(&canvas_path, notify::RecursiveMode::NonRecursive)?;

    for res in rx {
        use notify::EventKind;
        match res {
            Ok(event) => match event.kind {
                EventKind::Modify(_) | EventKind::Create(_) => {
                    // Read the file and parse it
                    let content = std::fs::read_to_string(&canvas_path)?;
                    let canvas = JsonCanvas::from_str(&content)?;
                    let syndication_items = to_syndication_format(
                        canvas,
                        Some(default_process_node)
                    );
                    println!("Found {} items to syndicate", syndication_items.len());
                    // TODO: actually syndicate the items
                },
                _ => {}
            },
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}
