use eframe::{
    egui::{Context, Ui},
    epaint::{ColorImage, Rect, Vec2},
};
use zen::graphics::{IndexedColor, Palette, Rgb888};

use super::texture::Texture;

pub struct IndexedTexture {
    pub texture: Texture,
    indexed_colors: Vec<IndexedColor>,
}

impl IndexedTexture {
    pub fn new(name: String) -> Self {
        Self {
            texture: Texture::new(name),
            indexed_colors: Vec::default(),
        }
    }

    pub fn ui(&self, ui: &mut Ui, widget_rect: Rect) {
        self.texture.ui(ui, widget_rect);
    }

    pub fn load_colors(
        &mut self,
        ctx: &Context,
        indexed_colors: Vec<IndexedColor>,
        palette: &Palette,
        texture_size: [usize; 2],
    ) {
        self.indexed_colors = indexed_colors;

        let colors: Vec<Rgb888> = self
            .indexed_colors
            .iter()
            .map(|idx_color| {
                palette.sub_palettes[idx_color.sub_palette].colors[idx_color.index].into()
            })
            .collect();
        self.texture.load_colors(ctx, colors, texture_size);
    }

    pub fn apply_colors(&mut self, palette: &Palette) {
        let colors = self.indexed_colors.iter().map(|idx_color| {
            palette.sub_palettes[idx_color.sub_palette].colors[idx_color.index].into()
        });

        self.texture.apply_colors(colors);
    }

    pub fn crop(&self, rect: Rect) -> Option<ColorImage> {
        self.texture.crop(rect)
    }

    pub fn size(&self) -> Vec2 {
        self.texture.size()
    }
}
