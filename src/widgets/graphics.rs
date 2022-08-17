use eframe::{
    egui::{Sense, Ui},
    epaint::Vec2,
};
// use zen::graphics::gfx::GFX_TILE_WIDTH;

use super::helpers::texture::Texture;

// const SELECTION_SIZE: [f32; 2] = [GFX_TILE_WIDTH as f32, GFX_TILE_WIDTH as f32];

pub struct GraphicsEditor {
    pub texture: Texture,
}

impl Default for GraphicsEditor {
    fn default() -> Self {
        Self {
            texture: Texture::new("GraphicsEditor".to_string()),
        }
    }
}

impl GraphicsEditor {
    pub fn ui(&mut self, ui: &mut Ui, widget_size: Vec2) {
        let (widget_rect, _) =
            ui.allocate_exact_size(widget_size, Sense::focusable_noninteractive());
        self.texture.ui(ui, widget_rect);
    }
}
