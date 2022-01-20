use std::sync::Mutex;

use futures::Future;

use eframe::{
    egui::{self, Vec2},
    epi,
};
use zen::super_metroid::{self, SuperMetroid};

use crate::widgets;

lazy_static::lazy_static! {
    static ref SELECTED_FILE_DATA: Mutex<Option<Vec<u8>>> = Mutex::new(None);
}

#[derive(Default)]
pub struct ZenSM {
    sm: SuperMetroid,
    palette: widgets::Palette,
}

impl epi::App for ZenSM {
    fn name(&self) -> &str {
        "Zen SM"
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
        if let Ok(mut selected_file) = SELECTED_FILE_DATA.lock() {
            if let Some(data) = &*selected_file {
                if let Ok(sm) = super_metroid::load_unheadered_rom(data.clone()) {
                    self.sm = sm;
                }
                *selected_file = None;
            }
        }

        egui::TopBottomPanel::top("top_menu").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.small_button("Load ROM file").clicked() {
                        super::app::execute_async(async move {
                            if let Some(file) = rfd::AsyncFileDialog::new().pick_file().await {
                                let file_data = file.read().await;
                                *SELECTED_FILE_DATA.lock().unwrap() = Some(file_data);
                            }
                        });
                    };
                });
            });
        });

        if !self.palette.is_texture_loaded() {
            if let Some(palette) = self.sm.palettes.get(&0xC2AE5D) {
                self.palette.load_texture(frame, palette);
            }
        }

        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(true)
            .default_height(300.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if self.palette.is_texture_loaded() {
                        let response = self.palette.ui(
                            ui,
                            self.sm.palettes.get_mut(&0xC2AE5D).unwrap(),
                            Vec2 { x: 300.0, y: 150.0 },
                        );
                        if response.changed() {
                            self.palette
                                .load_texture(frame, &self.sm.palettes[&0xC2AE5D]);
                        }
                    }
                });
            });
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn execute_async<F: Future<Output = ()> + Send + 'static>(f: F) {
    std::thread::spawn(move || futures::executor::block_on(f));
}

#[cfg(target_arch = "wasm32")]
fn execute_async<F: Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}
