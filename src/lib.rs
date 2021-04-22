#![feature(proc_macro_hygiene)]


#[macro_use]
extern crate lazy_static;

use std::{fs, io::{Error, ErrorKind, Result}, path::{Path, PathBuf}, collections::HashMap, sync::Mutex};
use rand::Rng;
use walkdir::WalkDir;
use arcropolis_api::*;

const RANDOMIZE_PATH: &str = "rom:/Randomizer/";

lazy_static! {
    static ref FILE_HOLDER: Mutex<HashMap<u64, PathBuf>> = {
        let m = HashMap::new();
        Mutex::new(m)
    };    
}

pub fn random_file_select(directory: &Path) -> Result<String>{
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
        return Err(Error::new(ErrorKind::Other, "No Files Found!"))
    }
    
    let random_result = rng.gen_range(0..count);

    Ok(files.get(&random_result).unwrap().to_string())
}

#[arc_callback]
fn arc_file_callback(hash: u64, data: &mut [u8]) -> Option<usize>{
    match random_file_select(FILE_HOLDER.lock().unwrap().get(&hash).unwrap()){
        Ok(s) => {
            let file = fs::read(s).unwrap();
            
            // Shoutouts to Genwald
            data[..file.len()].copy_from_slice(&file);

            Some(file.len())
        },
        Err(_err) => None
    }
}

#[stream_callback]
fn stream_file_callback(hash: u64) -> Option<String>{    
    match random_file_select(FILE_HOLDER.lock().unwrap().get(&hash).unwrap()){
        Ok(s) => Some(s),
        Err(_err) => None
    }
}

fn get_biggest_size_from_path(path: &Path) -> usize{
    let mut biggest_size: usize = 0;

    for entry in fs::read_dir(path).unwrap() {
        let size = entry.unwrap().metadata().unwrap().len() as usize;
        if size > biggest_size {
            biggest_size = size;
        }
    };

    biggest_size
}

#[skyline::main(name = "arc-randomizer")]
pub fn main() {
    if !Path::new(RANDOMIZE_PATH).exists(){
        return;
    }

    for entry in WalkDir::new(&RANDOMIZE_PATH) {
        let entry = entry.unwrap();

        if entry.path().is_dir() && format!("{}", &entry.path().display()).contains("."){

            let path = &format!("{}", &entry.path().display())[RANDOMIZE_PATH.len()..].replace(";", ":").replace(".mp4", ".webm");
            
            let hash = hash40(path);
            
            FILE_HOLDER.lock().unwrap().insert(hash.as_u64(), entry.path().to_path_buf());
            
            if path.contains("stream"){
                stream_file_callback::install(hash);
            }else{
                arc_file_callback::install(hash, get_biggest_size_from_path(&entry.path()));
            }

        }
    }
}