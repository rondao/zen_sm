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
    palette: widgets::PaletteEditor,
    selected_palette: usize,
}

impl epi::App for ZenSM {
    fn name(&self) -> &str {
        "Zen SM"
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
        if let Ok(mut mutex_content) = SELECTED_FILE_DATA.lock() {
            if let Some(data) = &*mutex_content {
                self.load_rom_from_file(data, frame);
                *mutex_content = None;
            }
        }

        egui::TopBottomPanel::top("top_menu").show(ctx, |ui| {
            self.draw_menu(ui);
        });

        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(true)
            .default_height(300.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if !self.sm.palettes.is_empty() {
                        self.draw_combo_box_palette_selection(ui, frame);

                        let response = self.palette.ui(
                            ui,
                            self.sm.palettes.get_mut(&self.selected_palette).unwrap(),
                            Vec2 { x: 300.0, y: 150.0 },
                        );
                        if response.changed() {
                            self.palette
                                .load_texture(frame, &self.sm.palettes[&self.selected_palette]);
                        }
                    }
                });
            });
    }
}

impl ZenSM {
    fn load_rom_from_file(&mut self, data: &Vec<u8>, frame: &epi::Frame) {
        if let Ok(sm) = super_metroid::load_unheadered_rom(data.clone()) {
            self.sm = sm;

            self.selected_palette = *self.sm.palettes.keys().next().unwrap();
            self.palette
                .load_texture(frame, &self.sm.palettes[&self.selected_palette]);
        }
    }

    fn draw_menu(&mut self, ui: &mut egui::Ui) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Load ROM from file").clicked() {
                    super::app::execute_async(async move {
                        if let Some(file) = rfd::AsyncFileDialog::new().pick_file().await {
                            let file_data = file.read().await;
                            *SELECTED_FILE_DATA.lock().unwrap() = Some(file_data);
                        }
                    });
                    ui.close_menu();
                };
                if ui.button("Save ROM to file").clicked() {
                    self.sm.save_to_rom();
                    super::app::save_file(&self.sm.rom);
                    ui.close_menu();
                };
            });
        });
    }

    fn draw_combo_box_palette_selection(&mut self, ui: &mut egui::Ui, frame: &epi::Frame) {
        let previous_selection = self.selected_palette;
        egui::ComboBox::from_label("Palette")
            .selected_text(format!("{:x?}", self.selected_palette))
            .show_ui(ui, |ui| {
                for palette in self.sm.palettes.keys() {
                    ui.selectable_value(
                        &mut self.selected_palette,
                        *palette,
                        format!("{:x?}", palette),
                    );
                }
            });
        if previous_selection != self.selected_palette {
            self.palette
                .load_texture(frame, &self.sm.palettes[&self.selected_palette]);
        }
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

#[cfg(not(target_arch = "wasm32"))]
fn save_file(data: &Vec<u8>) {
    if let Some(file) = rfd::FileDialog::new().save_file() {
        if let Err(e) = std::fs::write(file, data) {
            println!("{:?}", e);
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn save_file(data: &Vec<u8>) {
    use wasm_bindgen::JsCast;
    use web_sys::BlobPropertyBag;

    // Create a JsArray for the ROM data.
    let uint8arr = js_sys::Uint8Array::new(&unsafe { js_sys::Uint8Array::view(&data) }.into());
    let js_bytes = js_sys::Array::new();
    js_bytes.push(&uint8arr.buffer());

    if let Ok(blob) = web_sys::Blob::new_with_u8_array_sequence_and_options(
        &js_bytes,
        BlobPropertyBag::new().type_("application/octet-stream"),
    ) {
        if let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) {
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    if let Some(element) = document.get_element_by_id("download_rom") {
                        let anchor = element.dyn_into::<web_sys::HtmlAnchorElement>().unwrap();
                        anchor.set_href(&url);
                        anchor.click();
                    }
                }
            }
            web_sys::Url::revoke_object_url(&url).unwrap();
        }
    }
}
