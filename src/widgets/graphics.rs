use eframe::{
    egui::{Sense, Ui},
    epaint::Vec2,
};
// use zen::graphics::gfx::GFX_TILE_WIDTH;

use super::helpers::{texture::Texture, zoom_area::ZoomArea};

// const SELECTION_SIZE: [f32; 2] = [GFX_TILE_WIDTH as f32, GFX_TILE_WIDTH as f32];

pub struct GraphicsEditor {
    zoomable_area: ZoomArea,
    pub texture: Texture,
}

impl Default for GraphicsEditor {
    fn default() -> Self {
        Self {
            zoomable_area: ZoomArea::default(),
            texture: Texture::new("GraphicsEditor".to_string()),
        }
    }
}

impl GraphicsEditor {
    pub fn ui(&mut self, ui: &mut Ui, widget_size: Vec2) {
        let (widget_rect, _) =
            self.zoomable_area
                .create(ui, widget_size, Sense::focusable_noninteractive());
        self.texture.ui(ui, widget_rect);
    }
}
