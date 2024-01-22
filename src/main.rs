mod mod_installation;
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

const SAVED_DIRECTORY: &str = "7daystodie_path.txt";
const SIZE_AND_ADD_SPACE_AMOUNT: f32 = 20.0;

enum MatchesGameDirectory {
    Match,
    NoMatch
}

struct ModManager {
    directory_string: String,
    directory_found: bool,
    vector_string_channels: HashMap<String, (crossbeam_channel::Sender<Vec<String>>, crossbeam_channel::Receiver<Vec<String>>)>,
    //mods: Vec<Mod>
}

impl ModManager {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let dir_found;

        Self {
            directory_string: match utils::load_saved_directory() {
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
            vector_string_channels: HashMap::new(),
           //mods: Vec::new()
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

                    match utils::check_directory_match(&directory) {
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

                let (vector_send, vector_recv) = crossbeam_channel::unbounded();

                thread::scope(|scoped_spawner| {
                   scoped_spawner.spawn(|| {
                       let vectors = mod_installation::create_zip_vectors(&mod_paths);

                       vector_send.send(vectors).expect("Failed to send data!");
                   });

                });// end of scope

                let mods_list_channel = crossbeam_channel::unbounded();
                self.vector_string_channels.insert("Mods List".to_string(), mods_list_channel);

                let mods_list_send1 = self.vector_string_channels.get("Mods List").unwrap().clone().0;
                let directory_string_clone = self.directory_string.clone();

                make_named_thread!("Extract Zips").spawn(move || {
                    let directory_string = directory_string_clone;
                    let (zip_files, _zip_paths) = vector_recv.recv().unwrap();

                    dialog::Message::new("About to install mods. The mod manager may freeze for a few moments.")
                        .title("Installing mods")
                        .show()
                        .unwrap();
                    mod_installation::extract_zips(&zip_files);

                    if let Err(e) = mod_installation::check_dirs(&directory_string) {
                        let message = format!("An error occurred when moving the extracted mod folders into the main game directory.\n
                        The error: {e}");

                        utils::zips_temp_exists(|zips_path| fs::remove_dir_all(zips_path).unwrap());
                        utils::write_to_panic_output(message.as_str()).unwrap();
                        panic!("{}", message);
                    }

                    utils::zips_temp_exists(|mods_path| fs::remove_dir_all(mods_path).unwrap());
                    dialog::Message::new("The mods have been successfully installed.")
                        .title("Installation status")
                        .show()
                        .unwrap();

                    let mods_vector = mod_installation::scan_game_mods(&directory_string).unwrap_or_else(|e| {
                        let mut single_element_vec = Vec::new();
                        single_element_vec.push(String::from("Mods list could not be loaded/displayed"));

                        single_element_vec
                    });

                   mods_list_send1.send(mods_vector).unwrap();
                }).expect("Failed to spawn thread!");
            }
            // ** End of "Install mods" button code **

            ui.add_space(SIZE_AND_ADD_SPACE_AMOUNT);
            if ui.button(RichText::new("Show installed mod(s)").size(SIZE_AND_ADD_SPACE_AMOUNT)).clicked() {
                let mods_vector = mod_installation::scan_game_mods(&self.directory_string).unwrap_or_else(|e| {
                    let mut single_element_vec = Vec::new();
                    single_element_vec.push(String::from("Mods list could not be loaded/displayed"));

                    single_element_vec
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


