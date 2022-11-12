use eframe::egui::{Context, Sense, Ui};
use zen::{graphics::IndexedColor, graphics::Palette};
// use zen::graphics::gfx::GFX_TILE_WIDTH;

use super::helpers::{indexed_texture::IndexedTexture, zoom_area::ZoomArea};

// const SELECTION_SIZE: [f32; 2] = [GFX_TILE_WIDTH as f32, GFX_TILE_WIDTH as f32];

pub struct TileTableEditor {
    zoomable_area: ZoomArea,
    texture: IndexedTexture,
}

impl Default for TileTableEditor {
    fn default() -> Self {
        Self {
            zoomable_area: ZoomArea::default(),
            texture: IndexedTexture::new("TileTableEditor".to_string()),
        }
    }
}

impl TileTableEditor {
    pub fn ui(&mut self, ui: &mut Ui) {
        let (widget_rect, _) =
            self.zoomable_area
                .create(ui, self.texture.size(), Sense::focusable_noninteractive());
        self.texture.ui(ui, widget_rect);
    }

    pub fn load_texture(
        &mut self,
        ctx: &Context,
        tile_colors: Vec<IndexedColor>,
        palette: &Palette,
        texture_size: [usize; 2],
    ) {
        self.texture
            .load_colors(ctx, tile_colors, palette, texture_size);
    }

    pub fn apply_colors(&mut self, palette: &Palette) {
        self.texture.apply_colors(palette);
    }
}
