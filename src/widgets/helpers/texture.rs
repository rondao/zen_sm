use eframe::{
    egui::{Context, Image, TextureFilter, Ui},
    epaint::{Color32, ColorImage, Rect, TextureHandle, Vec2},
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
        self.texture = Some(ctx.load_texture(&self.name, image, TextureFilter::Nearest));
    }

    pub fn load_image(&mut self, ctx: &Context, image: ColorImage) {
        self.image = Some(image.clone());
        self.texture = Some(ctx.load_texture(&self.name, image, TextureFilter::Nearest));
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
            texture.set(image.clone(), TextureFilter::Nearest);
        }
    }

    pub fn crop(&self, rect: Rect) -> Option<ColorImage> {
        self.image.as_ref().and_then(|image| {
            let mut crop = ColorImage::new(
                [rect.size().x as usize, rect.size().y as usize],
                Color32::BLACK,
            );

            let x_size = rect.width() as usize;
            let y_size = rect.height() as usize;

            let top_right_start = rect.min.x as usize + (rect.min.y as usize * image.width());
            for y in 0..y_size {
                for x in 0..x_size {
                    crop.pixels[x + (y * x_size)] =
                        image.pixels[top_right_start + x + (y * image.width())];
                }
            }

            Some(crop)
        })
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
