use std::sync::Mutex;

use futures::Future;

use crate::widgets::{self};
use eframe::egui::{self, Context, TextureFilter, Ui};

use zen::super_metroid::{
    self,
    tile_table::BLOCK_SIZE,
    tileset::{tileset_size, tileset_to_colors},
    SuperMetroid,
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
    level: widgets::LevelEditor,
    sorted_room_list: Vec<usize>,
    selected_room: usize,
    selected_state: usize,
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

        egui::TopBottomPanel::top("top_menu").show(ctx, |ui| match ZenSM::draw_menu(ui) {
            Menu::LoadFromFile => self.load_from_file(),
            Menu::SaveToFile => self.save_to_file(),
            Menu::None => (),
        });

        if !self.sm.states.is_empty() {
            let state = &self.sm.states[&self.selected_state];
            let tileset = self.sm.tilesets[state.tileset as usize];
            let tileset_idx = state.tileset;

            egui::TopBottomPanel::bottom("bottom")
                .resizable(true)
                .default_height(150.0)
                .show(ctx, |ui| {
                    egui::SidePanel::right("bottom_right").show_inside(ui, |ui| {
                        self.draw_palette(ui, tileset.palette as usize);
                    });
                    self.draw_tile_table(ui);
                });

            egui::SidePanel::right("right_panel")
                .resizable(true)
                .default_width(150.0)
                .show(ctx, |ui| {
                    egui::TopBottomPanel::top("Tileset").show_inside(ui, |ui| {
                        self.draw_room_selector(ui);
                    });
                    egui::TopBottomPanel::top("Tileset").show_inside(ui, |ui| {
                        self.draw_tileset_selector(ui, tileset_idx as usize);
                    });
                    self.draw_graphics(ui);
                });

            egui::CentralPanel::default().show(ctx, |ui| {
                self.draw_level(ui);
            });
        }
    }
}

impl ZenSM {
    fn load_data_rom(&mut self, data: &Vec<u8>) {
        if let Ok(sm) = super_metroid::load_unheadered_rom(data.clone()) {
            self.sm = sm;

            self.sorted_room_list = self.sm.rooms.keys().map(|value| *value).collect();
            self.sorted_room_list.sort();

            let room = &self.sm.rooms[&self.sorted_room_list[0]];
            self.selected_room = self.sorted_room_list[0];
            self.selected_state = room.state_conditions[0].state_address as usize;
        }
    }

    fn draw_menu(ui: &mut Ui) -> Menu {
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

    fn draw_palette(&mut self, ui: &mut Ui, palette: usize) {
        if !self.sm.palettes.is_empty() {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let (response, _) = self
                    .palette
                    .ui(ui, self.sm.palettes.get_mut(&palette).unwrap());
                if response.changed() {
                    self.reload_textures(ui.ctx());
                }
            });
        }
    }

    fn draw_graphics(&mut self, ui: &mut Ui) {
        if !self.sm.graphics.is_empty() {
            egui::ScrollArea::vertical().show(ui, |ui| {
                self.graphics.ui(ui);
            });
        }
    }

    fn draw_tile_table(&mut self, ui: &mut Ui) {
        if !self.sm.tile_tables.is_empty() {
            egui::ScrollArea::both().show(ui, |ui| {
                self.tiletable.ui(ui);
            });
        }
    }

    fn draw_tileset_selector(&mut self, ui: &mut Ui, tileset_idx: usize) {
        if !self.sm.tilesets.is_empty() {
            if let Some(selection) = ZenSM::draw_combo_box(
                ui,
                "Tileset",
                (0..self.sm.tilesets.len()).collect::<Vec<usize>>().iter(),
                tileset_idx,
            ) {
                self.sm
                    .states
                    .get_mut(&self.selected_state)
                    .unwrap()
                    .tileset = selection as u8;
                self.reload_textures(ui.ctx());
            };

            let tileset = self.sm.tilesets[tileset_idx];
            if let Some(selection) = ZenSM::draw_combo_box(
                ui,
                "Palette",
                self.sm.palettes.keys(),
                tileset.palette as usize,
            ) {
                self.sm.tilesets[tileset_idx].palette = selection as u32;
                self.reload_textures(ui.ctx());
            }
            if let Some(selection) = ZenSM::draw_combo_box(
                ui,
                "Graphic",
                self.sm.graphics.keys(),
                tileset.graphic as usize,
            ) {
                self.sm.tilesets[tileset_idx].graphic = selection as u32;
                self.reload_textures(ui.ctx());
            };
            if let Some(selection) = ZenSM::draw_combo_box(
                ui,
                "Tile Table",
                self.sm.tile_tables.keys(),
                tileset.tile_table as usize,
            ) {
                self.sm.tilesets[tileset_idx].tile_table = selection as u32;
                self.reload_textures(ui.ctx());
            };
        }
    }

    fn draw_room_selector(&mut self, ui: &mut Ui) {
        if !self.sm.rooms.is_empty() {
            if let Some(selection) =
                ZenSM::draw_combo_box(ui, "Room", self.sorted_room_list.iter(), self.selected_room)
            {
                self.selected_room = selection;
                let room = &self.sm.rooms[&self.selected_room];

                self.selected_state = room.state_conditions[0].state_address as usize;
                self.reload_textures(ui.ctx());
            };

            let room = &self.sm.rooms[&self.selected_room];
            if let Some(selection) = ZenSM::draw_combo_box(
                ui,
                "State",
                room.state_conditions
                    .iter()
                    .map(|state_condition| state_condition.state_address as usize)
                    .collect::<Vec<_>>()
                    .iter(),
                self.selected_state,
            ) {
                self.selected_state = selection;
                self.reload_textures(ui.ctx());
            };
        }
    }

    fn draw_level(&mut self, ui: &mut Ui) {
        if !self.sm.states.is_empty() {
            egui::ScrollArea::both().show(ui, |ui| {
                self.level.ui(ui);
            });
        }
    }

    fn draw_combo_box<'a>(
        ui: &mut egui::Ui,
        label: &str,
        items: impl IntoIterator<Item = &'a usize>,
        selected: usize,
    ) -> Option<usize> {
        let mut selection = usize::MAX;

        egui::ComboBox::from_label(label)
            .selected_text(format!("{:x?}", selected))
            .show_ui(ui, |ui| {
                for item in items {
                    ui.selectable_value(&mut selection, *item, format!("{:x?}", item));
                }
            });

        (selection != usize::MAX && selection != selected).then(|| selection)
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
        self.sm.save_to_rom();
        super::app::save_file(&self.sm.rom);
    }

    fn reload_textures(&mut self, ctx: &Context) {
        self.reload_palette_texture(ctx);
        self.reload_gfx_texture(ctx);
        self.reload_tile_table_texture(ctx);
        self.reload_level_texture(ctx);
    }

    fn reload_palette_texture(&mut self, ctx: &Context) {
        let state = self.sm.states[&self.selected_state];
        self.palette.load_texture(
            ctx,
            self.sm.palettes[&(self.sm.tilesets[state.tileset as usize].palette as usize)]
                .to_colors(),
        );
    }

    fn reload_gfx_texture(&mut self, ctx: &Context) {
        let state = self.sm.states[&self.selected_state];
        let gfx = self
            .sm
            .gfx_with_cre(self.sm.tilesets[state.tileset as usize].graphic as usize);
        self.graphics.texture.load_texture(
            ctx,
            gfx.to_indexed_colors()
                .into_iter()
                .map(|idx_color| {
                    self.sm.palettes[&(self.sm.tilesets[state.tileset as usize].palette as usize)]
                        .sub_palettes[0]
                        .colors[idx_color as usize]
                        .into()
                })
                .collect(),
            gfx.size(),
        );
    }

    fn reload_tile_table_texture(&mut self, ctx: &Context) {
        let state = self.sm.states[&self.selected_state];
        self.tiletable.texture.load_texture(
            ctx,
            tileset_to_colors(
                &self.sm.tile_table_with_cre(
                    self.sm.tilesets[state.tileset as usize].tile_table as usize,
                ),
                &self.sm.palettes[&(self.sm.tilesets[state.tileset as usize].palette as usize)],
                &self
                    .sm
                    .gfx_with_cre(self.sm.tilesets[state.tileset as usize].graphic as usize),
            ),
            tileset_size(),
        );
    }

    fn reload_level_texture(&mut self, ctx: &Context) {
        let room = &self.sm.rooms[&self.selected_room];
        let (level_data, _, palette, graphics, tile_table) = self
            .sm
            .get_state_data(&self.sm.states[&self.selected_state]);

        let size = room.size_in_pixels();
        let colors = level_data.to_colors(room.size(), &tile_table, &palette, &graphics);

        self.level.gfx_layer.load_texture(ctx, colors, size);
        self.level.set_size(ctx, size);

        let bts_icons =
            level_data
                .layer1
                .iter()
                .zip(level_data.bts.iter())
                .map(|(block, bts_block)| {
                    self.level.bts_icons.get(&widgets::BtsTile {
                        block_type: block.block_type,
                        bts_block: *bts_block,
                    })
                });

        let [x_blocks, _] = room.size_in_blocks();
        for (i, bts_icon) in bts_icons.enumerate() {
            if let Some(bts_icon) = bts_icon {
                self.level.bts_layer.texture.as_mut().unwrap().set_partial(
                    [(i % x_blocks) * BLOCK_SIZE, (i / x_blocks) * BLOCK_SIZE],
                    bts_icon.clone(),
                    TextureFilter::Nearest,
                );
            }
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
