use eframe::{
    egui::{Context, Image, Ui, TextureOptions},
    epaint::{ColorImage, Rect, TextureHandle, Vec2},
};
use zen::graphics::Rgb888;

use crate::colors::rgb888s_to_rgba;

pub struct Texture {
    name: String,
    pub texture: Option<TextureHandle>,
    pub image: Option<ColorImage>,
}

impl Texture {
    pub fn new(name: String) -> Self {
        Self {
            name,
            texture: None,
            image: None,
        }
    }

    pub fn ui(&self, ui: &mut Ui, widget_rect: Rect) {
        if let Some(texture) = &self.texture {
            ui.put(widget_rect, Image::new(texture, widget_rect.size()));
        }
    }

    pub fn load_colors(&mut self, ctx: &Context, colors: Vec<Rgb888>, texture_size: [usize; 2]) {
        let image =
            ColorImage::from_rgba_unmultiplied(texture_size, &rgb888s_to_rgba(colors.into_iter()));

        self.image = Some(image.clone());
        self.texture = Some(ctx.load_texture(&self.name, image, TextureOptions::NEAREST));
    }

    pub fn load_image(&mut self, ctx: &Context, image: ColorImage) {
        self.image = Some(image.clone());
        self.texture = Some(ctx.load_texture(&self.name, image, TextureOptions::NEAREST));
    }

    pub fn apply_colors(&mut self, colors: impl Iterator<Item = Rgb888>) {
        let Some(image) = self.image.as_mut() else {return};
        image
            .pixels
            .iter_mut()
            .zip(colors)
            .for_each(|(pixel, color)| {
                pixel[0] = color.r;
                pixel[1] = color.g;
                pixel[2] = color.b;
            });
        if let Some(texture) = self.texture.as_mut() {
            texture.set(image.clone(), TextureOptions::NEAREST);
        }
    }

    pub fn size(&self) -> Vec2 {
        let size = if let Some(image) = self.image.as_ref() {
            image.size
        } else {
            [0, 0]
        };

        Vec2 {
            x: size[0] as f32,
            y: size[1] as f32,
        }
    }
}
