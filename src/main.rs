use std::error::Error;
use std::path::PathBuf;

use clap::Parser;
use notify::{self, Watcher};
use syndicate_json_canvas_lib::jsoncanvas::JsonCanvas;
use syndicate_json_canvas_lib::to_syndication_format;

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
        if !self.path.extension() == ".canvas" {
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
    watcher.watch(&args.path, notify::RecursiveMode::NonRecursive);

    for res in rx {
        use notify::EventKind;
        match res {
            Ok(event) => match event.kind {
                EventKind::Modify(_) | EventKind::Create(_) => {}
            },
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}
