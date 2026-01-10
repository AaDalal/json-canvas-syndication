use std::error::Error;
use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;
use notify::{self, Watcher};
use syndicate_json_canvas_lib::jsoncanvas::JsonCanvas;
use syndicate_json_canvas_lib::{to_syndication_format, default_node_filter, default_node_to_syndication_format_mapper};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    path: PathBuf,
}

impl Args {
    fn validate(&self) -> Result<(), &str> {
        if !self.path.is_file() {
            return Err("Provided path must be a file");
        }
        if self.path.extension().and_then(|s| s.to_str()) != Some("canvas") {
            return Err("Expect the extension to be .canvas");
        }
        Ok(())
    }
}

fn main() -> Result<(), Box<(dyn Error)>> {
    let args = Args::parse();
    args.validate()?;
    let (tx, rx) = std::sync::mpsc::channel::<notify::Result<notify::Event>>();
    let mut watcher = notify::recommended_watcher(tx)?;
    watcher.watch(&args.path, notify::RecursiveMode::NonRecursive)?;

    for res in rx {
        use notify::EventKind;
        match res {
            Ok(event) => match event.kind {
                EventKind::Modify(_) | EventKind::Create(_) => {
                    // Read the file and parse it
                    let content = std::fs::read_to_string(&args.path)?;
                    let canvas = JsonCanvas::from_str(&content)?;
                    let syndication_items = to_syndication_format(
                        canvas,
                        Some(default_node_filter),
                        Some(default_node_to_syndication_format_mapper)
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
