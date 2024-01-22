use std::error::Error;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::fs;
use fs_extra::dir;
use fs_extra::dir::CopyOptions;
use crate::utils;

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


fn create_paths_vector(mods: &Vec<PathBuf>) -> Vec<String> {
    let mut paths_vector: Vec<String> = Vec::new();

    for addon in mods {
        paths_vector.push(addon.display().to_string());
        println!("Pushed path to paths vector!");
    }

    paths_vector
}

fn create_files_vector(mods: &Vec<PathBuf>) -> Vec<File> {
    let mut files_vector: Vec<File> = Vec::new();

    for addon in mods {
        let file = File::open(addon).unwrap();
        files_vector.push(file);

        println!("Pushed file to files vector!");
    }

    files_vector
}

pub fn create_zip_vectors(mods: &Vec<PathBuf>) -> (Vec<File>, Vec<String>) {
    let paths = create_paths_vector(&mods);
    let files = create_files_vector(&mods);

    (files, paths)
}

pub fn check_dirs(mods_directory: &String) -> Result<(), Box<dyn Error>> {
    let zips_temp = dir::get_dir_content(".zips_temp")?;
    let copy_options = CopyOptions::default();

    for directory in zips_temp.directories {
        if directory.contains("Config") {
            println!("Skipping directory: {directory}");
            continue;
        }

        println!("Current directory: {directory}");

        for file in fs::read_dir(&directory)? {
            let entry = file?;

            println!("Current file: {:?}", entry.file_name());
            if entry.file_name() == "ModInfo.xml" {
                let dir = Path::new(&directory);

                let mods_path = Path::new(mods_directory);
                dir::move_dir(dir, mods_path, &copy_options)?;

                break;
            }
        }
    }

    Ok(())
}

pub fn scan_game_mods(game_mods_directory: &String) -> Result<Vec<String>, Box<dyn Error>> {
    let mut mods_vector = Vec::new();
    let mods_directory = dir::get_dir_content(game_mods_directory)?;

    for directory in mods_directory.directories {
        if directory.contains("Config") {
            continue;
        }

        println!("Current mod: {directory}");

        for file in fs::read_dir(&directory)? {
            let entry = file?;
            if entry.file_name() == "ModInfo.xml" {

            }
        }
    }

    Ok(mods_vector)
}

pub fn extract_zips(zip_files: &Vec<File>) {
    if let Err(e) = fs::create_dir(".zips_temp") {
        let message = format!("An error occurred when trying to create a directory where the zip files would be held before
        being extracted. The name of said directory should be '.zips_temp'. The error: {e}");
        utils::write_to_panic_output(message.as_str()).unwrap();

        panic!("CRASH OCCURRED. LOOK IN PANIC_OUTPUT.TXT FOR MORE DETAILS!");
    }

    for zip in zip_files {
        let mut archive = zip::ZipArchive::new(zip).unwrap();

        archive.extract(".zips_temp").unwrap();
    }
}