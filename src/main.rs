mod mod_installation;

use std::error::Error;
use std::{fs, thread};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use dialog::DialogBox;
use eframe::egui::{Color32, Context, RichText};
use eframe::{egui, Frame};
use std::sync::Arc;
use std::sync::Mutex;

const SAVED_DIRECTORY: &str = "7daystodie_path.txt";

enum MatchesGameDirectory {
    Match,
    NoMatch
}

fn check_directory_match(directory: &PathBuf) -> MatchesGameDirectory {
    if directory.to_str().unwrap().contains("7 Days To Die") {
        MatchesGameDirectory::Match
    } else {
        MatchesGameDirectory::NoMatch
    }
}

fn load_saved_directory() -> Result<String, Box<dyn Error>> {
    let mut cache_file = File::open(SAVED_DIRECTORY)?;
    let mut directory_string = String::new();

    cache_file.read_to_string(&mut directory_string)?;

    Ok(directory_string)
}

struct ModManager {
   pub directory_string: String,
   pub directory_found: bool
    // mods: Vec<Mod>,
}

impl ModManager {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut dir_found = false;

        Self {
            directory_string: match load_saved_directory() {
                Ok(dir) => {
                    dir_found = true;
                    dir
                }
                Err(e) => {
                    eprintln!("Could not find 7 Days to Die directory. Error: {e}");
                    dir_found = false;
                    String::from("Locate 7 Days to Die directory")
                }
            },
            directory_found: dir_found
            // mods: Vec::new(),
        }
    }

    fn save_directory(&mut self) -> Result<(), Box<dyn Error>> {
        let mut new_file = File::create(SAVED_DIRECTORY)?;
        new_file.write(self.directory_string.as_bytes())?;

        Ok(())
    }

    fn check_mod_path(&mut self, directory: &PathBuf) -> String {
        let directory_display = directory.display();
        let directory_mods = format!("{directory_display}/Mods");

        let mod_path = Path::new(&directory_mods);
        if !mod_path.exists() {
            fs::create_dir(&format!("{directory_display}/Mods")).expect("Failed to create mods directory!");
        }

        directory_mods
    }
}

impl eframe::App for ModManager {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx,  |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Install mods to:").color(Color32::LIGHT_BLUE).heading());
                ui.label(RichText::new(format!("{}", &self.directory_string)).color(Color32::LIGHT_GREEN).strong().heading());
            });

            if self.directory_found != true {
                ui.add_space(20.0);
                if ui.button(RichText::new("Find game directory").size(20.0)).clicked() {
                    let directory = match rfd::FileDialog::new().pick_folder() {
                        Some(dir) => dir,
                        None => return
                    };

                    match check_directory_match(&directory) {
                        MatchesGameDirectory::Match => {}
                        MatchesGameDirectory::NoMatch => {
                            dialog::Message::new("Wrong directory to 7 Days To Die")
                                .title("Incorrect directory/path")
                                .show()
                                .expect("Failed to show dialog box!");
                            return;
                        }
                    }

                    let directory_with_mods = self.check_mod_path(&directory);

                    self.directory_string = directory_with_mods;
                    if let Err(e) = self.save_directory() {
                        eprint!("There was an error in saving the path to a file: {e}");
                    }
                }
            }

            ui.add_space(20.0);
            if ui.button(RichText::new("Install mod(s)").size(20.0)).clicked() {
                let mod_paths = match rfd::FileDialog::new().pick_files() {
                    Some(f) => f,
                    None => return
                };

                let (mods_tx, mods_rx) = std::sync::mpsc::channel();
              //  let (extracted_tx, extracted_rx) = std::sync::mpsc::channel();

                let mod_paths_rc = Arc::new(Mutex::new(mod_paths));
                let mods_rc_clone = mod_paths_rc.clone();

                // Vector creation thread
                thread::spawn(move || {
                   let vectors = mod_installation::create_zip_vectors(mods_rc_clone);

                    mods_tx.send(vectors).expect("Failure in send!");
                });

                // Extraction thread
                let dir_clone = self.directory_string.clone();
                thread::spawn(move || {
                    let (zip_files, _zip_paths) = mods_rx.recv().unwrap();
                    mod_installation::extract_zips(zip_files, dir_clone);

                });
            }
        });
    }
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("Esar", native_options, Box::new(|cc| Box::new(ModManager::new(cc))))
        .expect("EFrame error on native");
}


