use eframe::{
    egui::{Response, Ui},
    epaint::Vec2,
};
use zen::graphics::{gfx::GFX_TILE_WIDTH, Gfx};

use super::editor::Editor;

const SELECTION_SIZE: [f32; 2] = [GFX_TILE_WIDTH as f32, GFX_TILE_WIDTH as f32];

pub struct GraphicsEditor {
    pub editor: Editor,
}

impl Default for GraphicsEditor {
    fn default() -> Self {
        Self {
            editor: Editor::new("GraphicsEditor".to_string()),
        }
    }
}

impl GraphicsEditor {
    pub fn ui(&mut self, ui: &mut Ui, gfx: &Gfx, widget_size: Vec2) -> Response {
        self.editor.ui(ui, gfx.size(), SELECTION_SIZE, widget_size)
    }
}
