use eframe::egui::{Sense, Ui};
// use zen::graphics::gfx::GFX_TILE_WIDTH;

use super::helpers::{texture::Texture, zoom_area::ZoomArea};

// const SELECTION_SIZE: [f32; 2] = [GFX_TILE_WIDTH as f32, GFX_TILE_WIDTH as f32];

pub struct TileTableEditor {
    zoomable_area: ZoomArea,
    pub texture: Texture,
}

impl Default for TileTableEditor {
    fn default() -> Self {
        Self {
            zoomable_area: ZoomArea::default(),
            texture: Texture::new("TileTableEditor".to_string()),
        }
    }
}

impl TileTableEditor {
    pub fn ui(&mut self, ui: &mut Ui) {
        let (widget_rect, _) =
            self.zoomable_area
                .create(ui, self.texture.size, Sense::focusable_noninteractive());
        self.texture.ui(ui, widget_rect);
    }
}
