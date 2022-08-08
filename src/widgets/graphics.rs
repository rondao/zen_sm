use eframe::{
    egui::{self, Ui},
    epaint::Vec2,
};
use zen::graphics::{gfx::GFX_TILE_WIDTH, Gfx, SubPalette};

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
    pub fn ui(
        &mut self,
        ui: &mut Ui,
        gfx: &Gfx,
        sub_palette: &SubPalette,
        widget_size: Vec2,
    ) -> egui::Response {
        self.editor.ui(
            ui,
            gfx.to_indexed_colors()
                .into_iter()
                .map(|idx_color| sub_palette.colors[idx_color as usize].into())
                .collect(),
            gfx.size(),
            SELECTION_SIZE,
            widget_size,
        )
    }
}
