use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

pub struct Mod {
   pub name: String,
   pub version: String,
   pub author: String
}

impl Mod {
    pub fn new(name: String, version: String, author: String) -> Self {
        Self {
            name,
            version,
            author
        }
    }
}

fn create_paths_vector(mods: Arc<Mutex<Vec<PathBuf>>>) -> Vec<String> {
    let mut paths_vector: Vec<String> = Vec::new();

    let mods_iter = mods.lock().unwrap();

    for addon in mods_iter.iter() {
        paths_vector.push(addon.display().to_string());
        println!("Pushed path to paths vector!");
    }

    paths_vector
}

fn create_files_vector(mods: Arc<Mutex<Vec<PathBuf>>>) -> Vec<File> {
    let mut files_vector: Vec<File> = Vec::new();

    let mods_iter = mods.lock().unwrap();

    for addon in mods_iter.iter() {
        let file = File::open(addon).unwrap();
        files_vector.push(file);

        println!("Pushed file to files vector!");
    }

    files_vector
}

pub fn create_zip_vectors(mods: Arc<Mutex<Vec<PathBuf>>>) -> (Vec<File>, Vec<String>) {
    let mods_path = mods.clone();
    let mods_file = mods.clone();

    let create_paths = thread::spawn(|| {
       let paths_vector = create_paths_vector(mods_path);

        paths_vector
    });

    let create_files = thread::spawn(|| {
       let files_vector = create_files_vector(mods_file);

        files_vector
    });

    let paths = create_paths.join().unwrap();
    let files = create_files.join().unwrap();

    (files, paths)
}

pub fn extract_zips(zip_files: Vec<File>, directory: String) {
    for zip in zip_files {
        let mut archive = zip::ZipArchive::new(zip).unwrap();

        archive.extract(&directory).unwrap();
    }
}