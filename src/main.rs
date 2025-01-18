#![windows_subsystem = "windows"]

use notify::{Watcher, RecursiveMode, Result, RecommendedWatcher, Config};
use std::{collections::HashMap, ffi::OsStr, fs, path::PathBuf, str::FromStr, thread, time::Duration};
use chrono;
use dirs_next;

use fern::Dispatch;
use log::LevelFilter;
use log::info;
use std::fs::File;
use serde_json;

fn setup_logger() {
    Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}] {}",
                chrono::Local::now().format("[%Y-%m-%d %H:%M:%S]"),
                record.level(),
                message
            ))
        })
        .level(LevelFilter::Debug)
        .chain(File::create("app.log").expect("failed to create log file")) // Logs to "app.log"
        .chain(std::io::stdout()) // Also logs to stdout
        .apply().expect("å nei");
}

fn main() -> Result<()> {
    setup_logger();

    std::panic::set_hook(Box::new(|info| {
        log::error!("Panic occurred: {:?}", info);
    }));

    sort_files();

    // get download directory
    let download_dir = dirs_next::download_dir();
    if download_dir.is_none() {
        info!("Failed to get download directory!");
        return Ok(());
    }
    let download_dir = download_dir.unwrap();

    // Create a channel to receive the events
    let (tx, rx) = std::sync::mpsc::channel();

    // Automatically select the best implementation for your platform
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Config::default())?;

    // Add the Downloads directory to be watched
    watcher.watch(&download_dir, RecursiveMode::NonRecursive)?;

    info!("Beginning to monitor {:?} for changes...", download_dir);

    // Run the loop forever, waiting for file events
    loop {
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(Ok(event)) => {
                println!("Event detected: {:?}", event);

                if event.kind.is_create() {
                    let mut b = sort_files();
                    while b {
                        thread::sleep(Duration::from_secs(1));
                        b = sort_files();
                    }
                }
            }
            Ok(Err(e)) => println!("Watch error: {:?}", e),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // This is fine; it just means no events were received
            }
            Err(e) => {
                println!("Receiver error: {:?}", e);
                break;
            }
        }
    }

    Ok(())
}

fn get_file_groups() -> HashMap<String, Vec<String>> {
    let default_map = HashMap::from([
        ("Documents".to_string(), vec![
            "txt".to_string(), 
            "pdf".to_string(), 
            "docx".to_string(), 
            "xlsx".to_string(), 
            "odt".to_string()
        ]),
        ("Archives".to_string(), vec![
            "zip".to_string(), 
            "rar".to_string(), 
            "7z".to_string()
        ]),
        ("Executables".to_string(), vec![
            "exe".to_string(), 
            "msi".to_string()
        ]),
        ("Images".to_string(), vec![
            "png".to_string(), 
            "jpg".to_string(), 
            "jpeg".to_string(), 
            "gif".to_string(), 
            "bmp".to_string(), 
            "svg".to_string(), 
            "webp".to_string()
        ]),
        ("Videos".to_string(), vec![
            "mp4".to_string(), 
            "mov".to_string(), 
            "avi".to_string(), 
            "webm".to_string(), 
            "mkv".to_string(), 
            "wmv".to_string()
        ]),
        ("Audio".to_string(), vec![
            "mp3".to_string(), 
            "wav".to_string(), 
            "ogg".to_string(), 
            "flac".to_string(), 
            "m4a".to_string()
        ]),
    ]);

    let json = match fs::read_to_string("config.json") {
        Ok(s) => s,
        Err(e) => {
            info!("Failed to read config file: {:?}", e);
            "".to_string()
        },
    };
    
    let map: HashMap<String, Vec<String>> = match serde_json::from_str(&json) {
        Ok(m) => m,
        Err(e) => {
            info!("{:?}", e);
            default_map
        },
    };

    for (k, v) in map.iter() {
        info!("{}: {:?}", k, v);
    }

    map
}

fn sort_files() -> bool {
    let file_groups = get_file_groups();

    // get download directory
    let download_dir = dirs_next::download_dir();
    if download_dir.is_none() {
        info!("Failed to get download directory!");
        return false;
    }
    let download_dir = download_dir.unwrap();
    
    let paths = fs::read_dir(&download_dir).expect("Failed to read contents of downloads directory");

    // for each path in download directory
    for path in paths {
        let path_clone = path.expect("Failed to unwrap path").path().clone();
        // if path is file
        if !path_clone.is_dir() {
            info!("path: {:?}", path_clone);
            let extension = match path_clone.extension().and_then(OsStr::to_str) {
                Some(e) => e,
                None => "",
            };

            if extension == "tmp" || extension == "crdownload" {
                return true;
            }
            
            // for each file group
            for (i, (k, v)) in file_groups.iter().enumerate() {
                // create group directories
                let mut group_dir = PathBuf::from(&download_dir);
                group_dir.push(PathBuf::from_str(&k).expect("Failed to push group_dir path buffer"));
                fs::create_dir_all(&group_dir).expect("Failed to create group_dir");

                // info!("Current iteration: {i}; map length: {}", file_groups.len());

                if v.contains(&extension.to_string()) {
                    let filename = path_clone.file_name().expect("Failed to get file name").to_str().unwrap();
                    group_dir.push(filename);
                    
                    match fs::rename(path_clone.to_path_buf(), group_dir) {
                        Ok(_) => {},
                        Err(e) => {
                            info!("Failed to move file: {:?} ERROR: {:?}", path_clone, e);
                            continue;
                        }
                    };
                }
                // move uncategorised files to Other
                else if i == file_groups.len() - 1 {
                    info!("Uncategorised file: {:?}", path_clone);
                    let filename = path_clone.file_name().expect("Failed to get file name").to_str().unwrap();
                    
                    let mut group_dir = PathBuf::from(&download_dir);
                    group_dir.push(PathBuf::from_str("Other").expect("Failed to push group_dir path buffer"));
                    fs::create_dir_all(&group_dir).expect("Failed to create group_dir");

                    group_dir.push(filename);
                    
                    match fs::rename(path_clone.to_path_buf(), group_dir) {
                        Ok(_) => {},
                        Err(e) => {
                            info!("Failed to move file: {:?} ERROR: {:?}", path_clone, e);
                            continue;
                        }
                    };
                }
            }
        }
    }

    return false;
}

