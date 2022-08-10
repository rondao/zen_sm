use std::sync::Mutex;

use futures::Future;

use crate::widgets::{self};
use eframe::egui::{self, Context, Vec2};

use zen::{
    graphics::gfx::{GFX_TILE_WIDTH, TILE_SIZE},
    super_metroid::{
        self,
        tileset::{tileset_size, tileset_to_colors},
        SuperMetroid,
    },
};

lazy_static::lazy_static! {
    static ref SELECTED_FILE_DATA: Mutex<Option<Vec<u8>>> = Mutex::new(None);
}

#[derive(Default)]
pub struct ZenSM {
    sm: SuperMetroid,
    palette: widgets::PaletteEditor,
    graphics: widgets::GraphicsEditor,
    tiletable: widgets::TileTableEditor,
    selected_palette: usize,
    selected_graphic: usize,
}

enum Menu {
    LoadFromFile,
    SaveToFile,
    None,
}

impl eframe::App for ZenSM {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        puffin::profile_function!();
        puffin::GlobalProfiler::lock().new_frame();

        // Check if user selected a file.
        if let Ok(mut mutex_content) = SELECTED_FILE_DATA.lock() {
            if let Some(data) = &*mutex_content {
                self.load_data_rom(data);
                self.reload_textures(ctx);
                *mutex_content = None;
            }
        }

        egui::TopBottomPanel::top("top_menu").show(ctx, |ui| match self.draw_menu(ui) {
            Menu::LoadFromFile => self.load_from_file(),
            Menu::SaveToFile => self.save_to_file(),
            Menu::None => (),
        });

        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(true)
            .show(ctx, |ui| {
                egui::SidePanel::right("bottom_right_panel").show_inside(ui, |ui| {
                    if !self.sm.palettes.is_empty() {
                        if let Some(selected_palette) = self.draw_combo_box_palette_selection(ui) {
                            self.selected_palette = selected_palette;
                            self.reload_textures(ctx);
                        }

                        let response = self.palette.ui(
                            ui,
                            self.sm.palettes.get_mut(&self.selected_palette).unwrap(),
                            Vec2 { x: 300.0, y: 150.0 },
                        );
                        if response.changed() {
                            self.reload_textures(ctx);
                        }
                    }
                });
                if !self.sm.tile_tables.is_empty() {
                    let size = super_metroid::tileset::tileset_size();
                    self.tiletable.ui(
                        ui,
                        Vec2 {
                            x: size[0] as f32,
                            y: size[1] as f32,
                        },
                    );
                }
            });

        egui::SidePanel::right("right_panel")
            .default_width((GFX_TILE_WIDTH * TILE_SIZE) as f32)
            .show(ctx, |ui| {
                if !self.sm.graphics.is_empty() {
                    if let Some(selected_graphic) = self.draw_combo_box_graphic_selection(ui) {
                        self.selected_graphic = selected_graphic;
                        self.reload_textures(ctx);
                    }

                    let gfx = self.sm.gfx_with_cre(self.selected_graphic);
                    let [x, y] = gfx.size();

                    self.graphics.ui(
                        ui,
                        &gfx,
                        Vec2 {
                            x: x as f32,
                            y: y as f32,
                        },
                    );
                }
            });
    }
}

impl ZenSM {
    fn load_data_rom(&mut self, data: &Vec<u8>) {
        if let Ok(sm) = super_metroid::load_unheadered_rom(data.clone()) {
            self.sm = sm;
            self.selected_palette = *self.sm.palettes.keys().next().unwrap();
            self.selected_graphic = *self.sm.graphics.keys().next().unwrap();
        }
    }

    fn draw_menu(&mut self, ui: &mut egui::Ui) -> Menu {
        let mut selected_menu = Menu::None;
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Load ROM from file").clicked() {
                    selected_menu = Menu::LoadFromFile;
                    ui.close_menu();
                };
                if ui.button("Save ROM to file").clicked() {
                    selected_menu = Menu::SaveToFile;
                    ui.close_menu();
                };
            });
        });
        selected_menu
    }

    fn load_from_file(&self) {
        super::app::execute_async(async move {
            if let Some(file) = rfd::AsyncFileDialog::new().pick_file().await {
                let file_data = file.read().await;
                *SELECTED_FILE_DATA.lock().unwrap() = Some(file_data);
            }
        });
    }

    fn save_to_file(&mut self) {
        let remapped_address = self.sm.save_to_rom();
        self.selected_palette = remapped_address[&self.selected_palette];

        super::app::save_file(&self.sm.rom);
    }

    fn reload_textures(&mut self, ctx: &Context) {
        self.reload_palette_texture(ctx);
        self.reload_gfx_texture(ctx);
        self.reload_tile_table_texture(ctx);
    }

    fn reload_palette_texture(&mut self, ctx: &Context) {
        self.palette
            .load_texture(ctx, self.sm.palettes[&self.selected_palette].to_colors());
    }

    fn reload_gfx_texture(&mut self, ctx: &Context) {
        let gfx = self.sm.gfx_with_cre(self.selected_graphic);
        self.graphics.editor.load_texture(
            ctx,
            gfx.to_indexed_colors()
                .into_iter()
                .map(|idx_color| {
                    self.sm.palettes[&self.selected_palette].sub_palettes[0].colors
                        [idx_color as usize]
                        .into()
                })
                .collect(),
            gfx.size(),
        );
    }

    fn reload_tile_table_texture(&mut self, ctx: &Context) {
        self.tiletable.editor.load_texture(
            ctx,
            tileset_to_colors(
                &self
                    .sm
                    .tile_table_with_cre(*self.sm.tile_tables.keys().next().unwrap()),
                &self.sm.palettes[&self.selected_palette],
                &self.sm.gfx_with_cre(self.selected_graphic),
            ),
            tileset_size(),
        );
    }

    fn draw_combo_box_palette_selection(&self, ui: &mut egui::Ui) -> Option<usize> {
        let mut selection = usize::default();

        egui::ComboBox::from_label("Palette")
            .selected_text(format!("{:x?}", self.selected_palette))
            .show_ui(ui, |ui| {
                for palette in self.sm.palettes.keys() {
                    ui.selectable_value(&mut selection, *palette, format!("{:x?}", palette));
                }
            });

        (selection != usize::default() && selection != self.selected_palette).then(|| selection)
    }

    fn draw_combo_box_graphic_selection(&self, ui: &mut egui::Ui) -> Option<usize> {
        let mut selection = usize::default();

        egui::ComboBox::from_label("Graphic")
            .selected_text(format!("{:x?}", self.selected_graphic))
            .show_ui(ui, |ui| {
                for palette in self.sm.graphics.keys() {
                    ui.selectable_value(&mut selection, *palette, format!("{:x?}", palette));
                }
            });

        (selection != usize::default() && selection != self.selected_graphic).then(|| selection)
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
