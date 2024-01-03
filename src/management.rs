use eframe::egui;
use eframe::egui::Context;
use egui::containers::Window;

pub trait ExtraWindow {
    fn show(&mut self);
    fn set_visibility(&mut self, visibility: bool);
    fn set_title_bar(&mut self, title_bar: bool);
    fn check_visibility(&self) -> bool;
}

pub struct MultipleDirectoriesWindow {
   pub name: String,
   pub has_title_bar: bool,
   pub is_visible: bool,
   pub context: Context
}

impl MultipleDirectoriesWindow {
   pub fn make(name: &str, context: Context) -> Self {
        Self {
            name: name.to_string(),
            has_title_bar: true,
            is_visible: true,
            context
        }
    }
}

impl ExtraWindow for MultipleDirectoriesWindow {
    fn show(&mut self) {
        let w_handle = Window::new(self.name.to_string()).auto_sized().open(&mut self.is_visible);

        println!("Showing window with name: {}", self.name);
        w_handle.show(&self.context, |ui| {
            ui.label("Hoe hoe hoe!");
        });
    }

    fn set_visibility(&mut self, visibility: bool) {
        self.is_visible = visibility;
    }
    fn set_title_bar(&mut self, title_bar: bool) {
        self.has_title_bar = title_bar;
    }

    fn check_visibility(&self) -> bool {
        self.is_visible
    }
}