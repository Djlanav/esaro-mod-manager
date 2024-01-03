mod mod_installation;
mod management;
mod utils;

use std::error::Error;
use std::{fs, thread};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use dialog::DialogBox;
use eframe::egui::{Color32, Context, RichText};
use eframe::{egui, Frame};
use std::sync::mpsc;
use crate::management::ExtraWindow;

const SAVED_DIRECTORY: &str = "7daystodie_path.txt";
const SIZE_AND_ADD_SPACE_AMOUNT: f32 = 20.0;

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
    directory_string: String,
    directory_found: bool,
    windows: Vec<Box<dyn ExtraWindow>>,
    vector_string_channels: HashMap<String, (mpsc::Sender<String>, mpsc::Receiver<String>)>
    // mods: Vec<Mod>,
}

impl ModManager {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let dir_found;

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
            directory_found: dir_found,
            windows: Vec::new(),
            vector_string_channels: HashMap::new()
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
                ui.add_space(SIZE_AND_ADD_SPACE_AMOUNT);
                if ui.button(RichText::new("Find game directory").size(SIZE_AND_ADD_SPACE_AMOUNT)).clicked() {
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

            // ** Beginning of "Install mods" button code **
            ui.add_space(SIZE_AND_ADD_SPACE_AMOUNT);
            if ui.button(RichText::new("Install mod(s)").size(SIZE_AND_ADD_SPACE_AMOUNT)).clicked() {
                let mod_paths = match rfd::FileDialog::new().pick_files() {
                    Some(f) => f,
                    None => return
                };

                // let mods_vector_channel = mpsc::channel();

                let (mods_tx, mods_rx) = crossbeam_channel::unbounded();
                // Scoped thread version
                thread::scope(|scoped_spawner| {
                   scoped_spawner.spawn(|| {
                       let vectors = mod_installation::create_zip_vectors(&mod_paths);

                       mods_tx.send(vectors).expect("Failed to send data!");
                   });

                    // Extraction thread
                    scoped_spawner.spawn(|| {
                       let (zip_files, _zip_paths) = mods_rx.recv().unwrap();

                        dialog::Message::new("About to install mods. The mod manager may freeze for a few moments.")
                            .title("Installing mods")
                            .show()
                            .unwrap();
                       mod_installation::extract_zips(&zip_files);

                        // Match block start
                        let mod_directories_vector = match mod_installation::check_dirs(&self.directory_string) {
                            Ok(vec) => vec,
                            Err(e) => {
                                let message = format!("An error occurred when moving the extracted mod folders into the main game directory.\n
                        The error: {e}");

                                utils::zips_temp_exists(|zips_path| fs::remove_dir_all(zips_path).unwrap());
                                utils::write_to_panic_output(message.as_str()).unwrap();
                                panic!("{}", message);
                            }
                        }; // Match block end

                        utils::zips_temp_exists(|mods_path| fs::remove_dir_all(mods_path).unwrap());
                        dialog::Message::new("The mods have been successfully installed.")
                            .title("Installation status")
                            .show()
                            .unwrap();
                    });
                });
            }
            // ** End of "Install mods" button code **

            ui.add_space(SIZE_AND_ADD_SPACE_AMOUNT);
            if ui.button(RichText::new("Show installed mod(s)").size(SIZE_AND_ADD_SPACE_AMOUNT)).clicked() {

            }

           /* ui.add_space(ADD_SPACE_AMOUNT);
            if ui.button(RichText::new("Click me!").size(ADD_SPACE_AMOUNT)).clicked() {
                let new_window = Box::new(MultipleDirectoriesWindow::make("Test window", ctx.clone()));

                self.windows.push(new_window);
            } */

            for window in self.windows.iter_mut() {
                window.show();
            }
        });
    }
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("Esar", native_options, Box::new(|cc| Box::new(ModManager::new(cc))))
        .expect("EFrame error on native");
}


