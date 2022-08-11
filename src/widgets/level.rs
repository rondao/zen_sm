use eframe::{
    egui::{Response, Ui},
    epaint::Vec2,
};
use zen::{graphics::gfx::GFX_TILE_WIDTH, super_metroid::room::Room};

use super::editor::Editor;

const SELECTION_SIZE: [f32; 2] = [GFX_TILE_WIDTH as f32, GFX_TILE_WIDTH as f32];

pub struct LevelEditor {
    pub editor: Editor,
}

impl Default for LevelEditor {
    fn default() -> Self {
        Self {
            editor: Editor::new("LevelEditor".to_string()),
        }
    }
}

impl LevelEditor {
    pub fn ui(&mut self, ui: &mut Ui, room: &Room) -> Response {
        let size = room.pixel_size();
        self.editor.ui(
            ui,
            [size.0, size.1],
            SELECTION_SIZE,
            Vec2 {
                x: size.0 as f32,
                y: size.1 as f32,
            },
        )
    }
}
