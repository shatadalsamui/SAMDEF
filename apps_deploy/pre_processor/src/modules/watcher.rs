use notify::{ Watcher, RecursiveMode, Config, EventKind };
use std::path::PathBuf;
use std::time::Duration;
use crossbeam::channel::Sender;

pub fn start_watch_loop(watch_path: &str, tx: Sender<PathBuf>) -> notify::Result<()> {
    let (internal_tx, internal_rx) = std::sync::mpsc::channel();

    // Configure watcher with a 2-second poll for "settling"
    // This ensures huge .tif files are fully written before we tile them
    let mut watcher = notify::RecommendedWatcher::new(
        internal_tx,
        Config::default().with_poll_interval(Duration::from_secs(2))
    )?;

    watcher.watch(watch_path.as_ref(), RecursiveMode::NonRecursive)?;

    println!("Watcher Active: Monitoring [{}] for new satellite TIFFs...", watch_path);

    for res in internal_rx {
        match res {
            Ok(event) => {
                // We trigger on Create or Modify events
                if let EventKind::Create(_) | EventKind::Modify(_) = event.kind {
                    for path in event.paths {
                        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                            if ext == "tif" || ext == "tiff" {
                                if let Err(e) = tx.send(path.clone()) {
                                    eprintln!("Warning: Failed to send file path to main loop: {:?} (error: {})", path, e);
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => eprintln!("Watcher error: {:?}", e),
        }
    }
    Ok(())
}
