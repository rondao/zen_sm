use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use eframe::epaint::ColorImage;
use futures::Future;

use crate::assets;
use crate::widgets::{self, TileTableCommand};
use eframe::egui::{self, Context, Ui};

use zen::graphics::IndexedColor;
use zen::super_metroid::{
    self,
    tileset::{tileset_size, tileset_to_indexed_colors, Tileset},
    SuperMetroid,
};

lazy_static::lazy_static! {
    static ref SELECTED_FILE_DATA: Mutex<Option<Vec<u8>>> = Mutex::new(std::fs::read("/home/rondao/roms/snes/SuperMetroid.smc").ok());
    static ref EDITOR_ASSETS: Arc<Mutex<HashMap<assets::BtsTile, ColorImage>>> = Arc::new(Mutex::new(HashMap::new()));
}

pub struct ZenSM {
    sm: SuperMetroid,
    palette_editor: widgets::PaletteEditor,
    graphics_editor: widgets::GraphicsEditor,
    tiletable_editor: widgets::TileTableEditor,
    level_editor: widgets::LevelEditor,
    sorted_room_list: Vec<usize>,
    selected_tileset: Option<TilesetSelection>,
    selected_room: Option<RoomSelection>,
}

impl Default for ZenSM {
    fn default() -> Self {
        assets::load_bts_icons(Arc::clone(&EDITOR_ASSETS));

        Self {
            sm: SuperMetroid::default(),
            palette_editor: widgets::PaletteEditor::default(),
            graphics_editor: widgets::GraphicsEditor::default(),
            tiletable_editor: widgets::TileTableEditor::default(),
            level_editor: widgets::LevelEditor::new(Arc::clone(&EDITOR_ASSETS)),
            sorted_room_list: Vec::default(),
            selected_tileset: Option::default(),
            selected_room: Option::default(),
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
struct RoomSelection {
    pub addr: usize,
    pub state_addr: usize,
}

#[derive(Default, Debug, Clone, Copy)]
struct TilesetSelection {
    pub index: usize,
    pub data: Tileset,
}

enum Menu {
    LoadFromFile,
    SaveToFile,
    None,
}

impl eframe::App for ZenSM {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // puffin::profile_function!();
        // puffin::GlobalProfiler::lock().new_frame();

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

        egui::TopBottomPanel::bottom("bottom")
            .resizable(true)
            .default_height(150.0)
            .show(ctx, |ui| {
                egui::SidePanel::right("bottom_right").show_inside(ui, |ui| {
                    self.palette_editor(ui);
                });
                self.draw_tile_table(ui);
            });

        egui::SidePanel::right("right_panel")
            .resizable(true)
            .default_width(150.0)
            .show(ctx, |ui| {
                egui::TopBottomPanel::top("RoomSelectorPanel").show_inside(ui, |ui| {
                    self.draw_room_selector(ui);
                });
                egui::TopBottomPanel::top("TilesetPanel").show_inside(ui, |ui| {
                    self.tileset_selector(ui);
                });
                self.draw_graphics(ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_level(ui);
        });
    }
}

// File manipulation.
impl ZenSM {
    fn load_data_rom(&mut self, data: &Vec<u8>) {
        if let Ok(sm) = super_metroid::load_unheadered_rom(data.clone()) {
            self.sm = sm;

            self.sorted_room_list = self.sm.rooms.keys().map(|value| *value).collect();
            self.sorted_room_list.sort();

            let room = &self.sm.rooms[&self.sorted_room_list[0]];
            let state_addr = room.state_conditions[0].state_address as usize;

            self.selected_room = Some(RoomSelection {
                addr: self.sorted_room_list[0],
                state_addr,
            });

            let state = self.sm.states[&(state_addr as usize)];
            let tileset_index = state.tileset as usize;
            self.selected_tileset = Some(TilesetSelection {
                index: tileset_index,
                data: self.sm.tilesets[tileset_index],
            });
        }
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
}

// Texture manipulation.
impl ZenSM {
    fn reload_textures(&mut self, ctx: &Context) {
        self.level_editor.clear_selection();

        if let Some(selected_tileset) = self.selected_tileset {
            let (palette, graphics, tile_table) = self.sm.get_tileset_data(selected_tileset.index);

            self.palette_editor.load_texture(ctx, palette.to_colors());
            self.graphics_editor.load_texture(
                ctx,
                graphics
                    .to_indexed_colors()
                    .iter()
                    .map(|index| IndexedColor {
                        index: *index as usize,
                        sub_palette: 0,
                    })
                    .collect(),
                palette,
                graphics.size(),
            );
            self.tiletable_editor.load_texture(
                ctx,
                tileset_to_indexed_colors(&tile_table, &graphics),
                palette,
                tileset_size(),
            );
        }

        self.reload_level_texture(ctx);
    }

    fn reload_level_texture(&mut self, ctx: &Context) {
        let Some(selected_room) = self.selected_room else {return};

        let room = &self.sm.rooms[&selected_room.addr];
        let (level_data, _, palette, graphics, tile_table) = self
            .sm
            .get_state_data(&self.sm.states[&selected_room.state_addr]);

        let size = room.size_in_pixels();
        let indexed_colors = level_data.to_indexed_colors(room.size(), &tile_table, &graphics);

        self.level_editor
            .load_level(ctx, level_data, indexed_colors, *palette, size);
    }
}

// Drawing functions.
impl ZenSM {
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

    fn palette_editor(&mut self, ui: &mut Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            let Some(tileset) = self.selected_tileset else {return};
            let Some(palette) = self.sm.palettes.get_mut(&(tileset.data.palette as usize)) else {return};

            let (response, _) = self.palette_editor.ui(ui, palette);
            if response.changed() {
                self.palette_editor.load_texture(ui.ctx(), palette.to_colors());
                self.graphics_editor.apply_colors(palette);
                self.tiletable_editor.apply_colors(palette);
                self.level_editor.apply_colors(palette);
            }
        });
    }

    fn draw_graphics(&mut self, ui: &mut Ui) {
        if !self.sm.graphics.is_empty() {
            egui::ScrollArea::vertical().show(ui, |ui| {
                self.graphics_editor.ui(ui);
            });
        }
    }

    fn draw_tile_table(&mut self, ui: &mut Ui) {
        if !self.sm.tile_tables.is_empty() {
            egui::ScrollArea::both().show(ui, |ui| match self.tiletable_editor.ui(ui) {
                TileTableCommand::Selected(block_selection) => {
                    let Some(tileset) = self.selected_tileset else {return};
                    let Some(palette) = self.sm.palettes.get_mut(&(tileset.data.palette as usize)) else {return};

                    self.level_editor
                        .set_selection(ui.ctx(), block_selection, palette)
                }
                TileTableCommand::None => (),
            });
        }
    }

    fn tileset_selector(&mut self, ui: &mut Ui) {
        let Some(mut tileset) = self.selected_tileset else {return};
        let tileset_index = tileset.index;

        if let Some(selection) = ZenSM::draw_combo_box(
            ui,
            "Tileset",
            (0..self.sm.tilesets.len()).collect::<Vec<usize>>().iter(),
            tileset_index,
        ) {
            let new_tileset = self.sm.tilesets[selection];
            self.selected_tileset = Some(TilesetSelection {
                index: selection,
                data: new_tileset,
            });

            if let Some(selected_room) = self.selected_room {
                self.sm
                    .states
                    .get_mut(&selected_room.state_addr)
                    .unwrap()
                    .tileset = selection as u8;
            }
            self.reload_textures(ui.ctx());
        };

        if let Some(selection) = ZenSM::draw_combo_box(
            ui,
            "Palette",
            self.sm.palettes.keys(),
            tileset.data.palette as usize,
        ) {
            tileset.data.palette = selection as u32;
            self.selected_tileset = Some(tileset);

            self.sm.tilesets[tileset_index].palette = selection as u32;
            self.reload_textures(ui.ctx());
        }

        if let Some(selection) = ZenSM::draw_combo_box(
            ui,
            "Graphic",
            self.sm.graphics.keys(),
            tileset.data.graphic as usize,
        ) {
            tileset.data.graphic = selection as u32;
            self.selected_tileset = Some(tileset);

            self.sm.tilesets[tileset_index].graphic = selection as u32;
            self.reload_textures(ui.ctx());
        };

        if let Some(selection) = ZenSM::draw_combo_box(
            ui,
            "Tile Table",
            self.sm.tile_tables.keys(),
            tileset.data.tile_table as usize,
        ) {
            tileset.data.tile_table = selection as u32;
            self.selected_tileset = Some(tileset);

            self.sm.tilesets[tileset_index].tile_table = selection as u32;
            self.reload_textures(ui.ctx());
        };
    }

    fn draw_room_selector(&mut self, ui: &mut Ui) {
        let Some(selected_room) = self.selected_room else {return};

        if let Some(selection) =
            ZenSM::draw_combo_box(ui, "Room", self.sorted_room_list.iter(), selected_room.addr)
        {
            let room = &self.sm.rooms[&selection];
            let state_addr = room.state_conditions[0].state_address as usize;
            self.selected_room = Some(RoomSelection {
                addr: selection,
                state_addr,
            });

            let tileset = self.sm.states[&state_addr].tileset as usize;
            self.selected_tileset = Some(TilesetSelection {
                index: tileset,
                data: self.sm.tilesets[tileset],
            });

            self.reload_textures(ui.ctx());
        };

        let room = &self.sm.rooms[&selected_room.addr];
        if let Some(selection) = ZenSM::draw_combo_box(
            ui,
            "State",
            room.state_conditions
                .iter()
                .map(|state_condition| state_condition.state_address as usize)
                .collect::<Vec<_>>()
                .iter(),
            selected_room.state_addr,
        ) {
            self.selected_room.as_mut().unwrap().state_addr = selection;

            let tileset = self.sm.states[&selection].tileset as usize;
            self.selected_tileset = Some(TilesetSelection {
                index: tileset,
                data: self.sm.tilesets[tileset],
            });

            self.reload_textures(ui.ctx());
        };
    }

    fn draw_level(&mut self, ui: &mut Ui) {
        let Some(tileset) = self.selected_tileset else {return};
        let Some(palette) = self.sm.palettes.get_mut(&(tileset.data.palette as usize)) else {return};
        let Some(selected_room) = self.selected_room else {return};

        let state = self.sm.states[&selected_room.state_addr];
        let level = self
            .sm
            .levels
            .get_mut(&(state.level_address as usize))
            .unwrap();

        egui::ScrollArea::both().show(ui, |ui| {
            self.level_editor.ui(ui, level, palette);
        });
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
