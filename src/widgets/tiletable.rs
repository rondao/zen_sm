use eframe::{
    egui::{self, Ui},
    epaint::Vec2,
};
use zen::{
    graphics::{gfx::GFX_TILE_WIDTH, Gfx, Palette},
    super_metroid::{
        tile_table::TileTable,
        tileset::{tileset_size, tileset_to_colors},
    },
};

use super::editor::Editor;

const SELECTION_SIZE: [f32; 2] = [GFX_TILE_WIDTH as f32, GFX_TILE_WIDTH as f32];

pub struct TileTableEditor {
    pub editor: Editor,
}

impl Default for TileTableEditor {
    fn default() -> Self {
        Self {
            editor: Editor::new("TileTableEditor".to_string()),
        }
    }
}

impl TileTableEditor {
    pub fn ui(
        &mut self,
        ui: &mut Ui,
        tile_table: &TileTable,
        graphics: &Gfx,
        palette: &Palette,
        widget_size: Vec2,
    ) -> egui::Response {
        self.editor.ui(
            ui,
            tileset_to_colors(tile_table, palette, graphics),
            tileset_size(),
            SELECTION_SIZE,
            widget_size,
        )
    }
}
