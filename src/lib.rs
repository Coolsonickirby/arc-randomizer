#![feature(proc_macro_hygiene)]

#[macro_use]
extern crate lazy_static;

use arcropolis_api::*;
use rand::Rng;
use std::{
    collections::HashMap,
    fs,
    io::{Error, ErrorKind, Result},
    path::{Path, PathBuf},
    sync::Mutex,
};
use walkdir::WalkDir;

const RANDOMIZE_PATH: &str = "rom:/Randomizer/";

enum CallbackType {
    Arc,
    Stream,
}

struct Callback {
    callback_type: CallbackType,
    size: usize,
}

lazy_static! {
    static ref CALLBACKS: Mutex<HashMap<u64, Callback>> = {
        let m = HashMap::new();
        Mutex::new(m)
    };
    static ref PARENT_FOLDER: Mutex<HashMap<u64, String>> = {
        let m = HashMap::new();
        Mutex::new(m)
    };
    static ref USE_FOLDER_FROM_PARENT: Mutex<HashMap<String, String>> = {
        let m = HashMap::new();
        Mutex::new(m)
    };
    static ref HASH_TO_ARC_PATH: Mutex<HashMap<u64, String>> = {
        let m = HashMap::new();
        Mutex::new(m)
    };
}

pub fn pick_random_folder_for_parent(parent_path: String) {
    let mut rng = rand::thread_rng();

    let folders: Vec<String> = std::fs::read_dir(&parent_path)
        .unwrap()
        .enumerate()
        .filter_map(|(_i, folder)| {
            let folder = folder.unwrap().path();

            if !folder.is_dir() {
                return None;
            }

            let name = folder
                .file_name()
                .unwrap()
                .to_os_string()
                .into_string()
                .unwrap();
            Some(name)
        })
        .collect();

    let count = folders.len();

    if count <= 0 {
        println!("No folders found under {}", parent_path);
        return;
    }

    let random_result = rng.gen_range(0..count);
    USE_FOLDER_FROM_PARENT
        .lock()
        .unwrap()
        .insert(parent_path, folders[random_result].clone());
}

pub fn random_file_select(directory: &Path) -> Result<String> {
    let mut rng = rand::thread_rng();

    let mut files = HashMap::new();

    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if !&path.is_dir() {
            files.insert(files.len(), format!("{}", path.display()));
        }
    }

    let count = files.len();

    if count <= 0 {
        return Err(Error::new(ErrorKind::Other, "No Files Found!"));
    }

    let random_result = rng.gen_range(0..count);

    Ok(files.get(&random_result).unwrap().to_string())
}

fn get_path_from_hash(hash: &u64) -> PathBuf {
    let mut path = PathBuf::new();
    let parent_folder = PARENT_FOLDER.lock().unwrap().get(&hash).unwrap().clone();
    path.push(&parent_folder);
    path.push(USE_FOLDER_FROM_PARENT.lock().unwrap().get(&parent_folder).unwrap());
    path.push(HASH_TO_ARC_PATH.lock().unwrap().get(&hash).unwrap());
    path
}

#[arc_callback]
fn randomize_folders(_hash: u64, _data: &mut [u8]) -> Option<usize> {
    std::fs::read_dir(&RANDOMIZE_PATH).unwrap().enumerate().for_each(|(_i, outer_path)| {
        let outer_path = outer_path.unwrap().path();
        if outer_path.is_dir(){
            pick_random_folder_for_parent(format!("{}", outer_path.display()));
        }
    });
    None
}

#[arc_callback]
fn arc_file_callback(hash: u64, data: &mut [u8]) -> Option<usize> {
    let path = get_path_from_hash(&hash);
    let _parent_folder = PARENT_FOLDER.lock().unwrap().get(&hash).unwrap().clone();
    // println!("\nPath to use for parent: {}\n\nARC Path: {}\n\nPath: {}", USE_FOLDER_FROM_PARENT.lock().unwrap().get(&parent_folder).unwrap(), HASH_TO_ARC_PATH.lock().unwrap().get(&hash).unwrap(), path.display());
    let res = {
        if path.is_dir() {
            random_file_select(&path)
        } else {
            Ok(format!("{}", path.display()))
        }
    };

    match res {
        Ok(s) => {
            let s = {
                if Path::new(&s).exists() {
                    s
                } else {
                    format!("arc:/{}", HASH_TO_ARC_PATH.lock().unwrap().get(&hash).unwrap())
                }
            };
            match fs::read(s) {
                Ok(file) => {
                    // Shoutouts to Genwald
                    data[..file.len()].copy_from_slice(&file);
            
                    Some(file.len())
                },
                Err(_err) => {
                    None
                }
            }
    
        }
        Err(_err) => None,
    }
}

#[stream_callback]
fn stream_file_callback(hash: u64) -> Option<String> {
    let path = get_path_from_hash(&hash);
    let res = {
        if path.is_dir() {
            random_file_select(&path)
        } else {
            Ok(format!("{}", path.display()))
        }
    };

    match res {
        Ok(s) => {
            if Path::new(&s).exists() {
                Some(s)
            } else {
                None
            }
        },
        Err(_err) => None,
    }
}

fn get_biggest_size_from_path(path: &Path) -> usize {
    let mut biggest_size: usize = 0;

    for entry in fs::read_dir(path).unwrap() {
        let size = entry.unwrap().metadata().unwrap().len() as usize;
        if size > biggest_size {
            biggest_size = size;
        }
    }

    biggest_size
}

#[skyline::main(name = "arc-randomizer-folder")]
pub fn main() {
    if !Path::new(RANDOMIZE_PATH).exists() {
        return;
    }

    std::fs::read_dir(&RANDOMIZE_PATH)
        .unwrap()
        .enumerate()
        .for_each(|(_i, outer_path)| {
            let outer_path = outer_path.unwrap().path();
            std::fs::read_dir(&outer_path)
                .unwrap()
                .enumerate()
                .for_each(|(_y, inner_path)| {
                    let inner_path = inner_path.unwrap().path();
                    let inner_path_str = format!("{}", &inner_path.display());
                    for entry in WalkDir::new(&inner_path) {
                        let entry = entry.unwrap();

                        let arc_path = &format!("{}", &entry.path().display())
                            [inner_path_str.len() + 1..]
                            .replace(";", ":")
                            .replace(".mp4", ".webm");
                        
                        if arc_path.contains(".") {
                            // File or Folder found
                            let hash = hash40(arc_path).as_u64();

                            let callback: Callback = Callback {
                                size: {
                                    if entry.path().is_dir() {
                                        get_biggest_size_from_path(&entry.path())
                                    } else {
                                        entry.metadata().unwrap().len() as usize
                                    }
                                },
                                callback_type: {
                                    if arc_path.contains("stream") {
                                        CallbackType::Stream
                                    } else {
                                        CallbackType::Arc
                                    }
                                },
                            };

                            if CALLBACKS.lock().unwrap().contains_key(&hash) {
                                if CALLBACKS.lock().unwrap().get(&hash).unwrap().size
                                    < callback.size
                                {
                                    *CALLBACKS.lock().unwrap().get_mut(&hash).unwrap() = callback;
                                }
                            } else {
                                CALLBACKS.lock().unwrap().insert(hash, callback);
                            }

                            PARENT_FOLDER
                                .lock()
                                .unwrap()
                                .insert(hash, format!("{}", outer_path.display()));
                            HASH_TO_ARC_PATH
                                .lock()
                                .unwrap()
                                .insert(hash, arc_path.to_string());
                        }
                    }
                });
        });

    for (key, value) in &*CALLBACKS.lock().unwrap() {
        match value.callback_type {
            CallbackType::Arc => arc_file_callback::install(*key, value.size),
            CallbackType::Stream => stream_file_callback::install(*key),
        }
    }

    std::fs::read_dir(&RANDOMIZE_PATH).unwrap().enumerate().for_each(|(_i, outer_path)| {
        let outer_path = outer_path.unwrap().path();
        if outer_path.is_dir(){
            pick_random_folder_for_parent(format!("{}", outer_path.display()));
        }
    });
    randomize_folders::install(
        Hash40::from("stage/resultstage/normal/motion/resultstage_set/resultstage_set_00.nuanmb"),
        0,
    );
}
