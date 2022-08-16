use eframe::{
    egui::{Response, Ui},
    epaint::{Rect, Vec2},
};
use zen::{graphics::gfx::GFX_TILE_WIDTH, super_metroid::tileset::tileset_size};

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
    pub fn ui(&mut self, ui: &mut Ui, widget_size: Vec2) -> (Response, Rect) {
        self.editor
            .ui(ui, tileset_size(), SELECTION_SIZE, widget_size)
    }
}
