use clearscreen::clear;
use xxhash_rust::xxh3::xxh3_64;
use std::fs;
use std::path::PathBuf;
use std::io;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use std::process::Command;

fn xxh3(content : &str) -> u64 {
    return xxh3_64(content.as_bytes());
}

fn getfilespath(path: PathBuf) -> io::Result<Vec<PathBuf>> {
    let entries = fs::read_dir(&path)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect();
    
    Ok(entries)
}

fn getallfiles(mut files : Vec<PathBuf>, path: PathBuf) -> Vec<PathBuf> {
    let entries = match getfilespath(path) {
        Ok(entries) => entries,
        Err(_) => Vec::new()
    };
    for entry in entries {
        if entry.is_file() {
            files.push(entry);
        }
        else if entry.is_dir() {
            files = getallfiles(files, entry);
        }
    }
    files
}

fn main() {
    let mut waittime : i32 = std::env::args().nth(1)
        .unwrap_or("50".to_string())
        .parse()
        .expect("argument provided is not a valid number");
    waittime = ((waittime as f32 /1000.0)*(20.0)) as i32;
    let mut waitingtime = 0;

    let mut hashes : HashMap<PathBuf,u64> = HashMap::new();
    let mut prevhashes : HashMap<PathBuf,u64> = HashMap::new();
    
    match Command::new("cargo").arg("--version").output() {
        Ok(out) => assert!(out.status.success()),
        Err(_) => panic!()
    }

    loop {
        let files = getallfiles(Vec::new(), PathBuf::from("./src/"));
        for file in files {
            if !file.to_string_lossy().ends_with('~') {
                match fs::read_to_string(&file) {
                    Ok(content) => {
                        match hashes.insert(file,xxh3(content.as_str())) {
                            None => {},
                            Some(v) => {
                                if v != xxh3(content.as_str()) {
                                    waitingtime = 0;
                                } else {
                                    waitingtime = waitingtime + 1;
                                }
                            }
                        }
                    },
                    Err(_) => {}
                }
            }
        }

        if waitingtime == waittime {
            waitingtime = 0;
            if prevhashes != hashes {
                if !prevhashes.is_empty() {
                    clear().expect("");
                    Command::new("cargo").arg("check").status().expect("failed");
                    // todo: filter out every "Blocking waiting for file lock on package cache"
                }
                prevhashes = hashes.clone();
            }
        }

        thread::sleep(Duration::from_millis(50));
    }
}
