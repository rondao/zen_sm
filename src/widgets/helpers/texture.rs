use eframe::{
    egui::{Context, Image, TextureFilter, Ui},
    epaint::{ColorImage, Rect, TextureHandle, Vec2},
};
use zen::graphics::Rgb888;

pub struct Texture {
    name: String,
    pub texture: Option<TextureHandle>,
    pub size: Vec2,
}

impl Texture {
    pub fn new(name: String) -> Self {
        Self {
            name,
            texture: None,
            size: Vec2::default(),
        }
    }

    pub fn ui(&self, ui: &mut Ui, widget_rect: Rect) {
        if let Some(texture) = &self.texture {
            ui.put(widget_rect, Image::new(texture, widget_rect.size()));
        }
    }

    pub fn load_texture(&mut self, ctx: &Context, colors: Vec<Rgb888>, texture_size: [usize; 2]) {
        self.size = Vec2 {
            x: texture_size[0] as f32,
            y: texture_size[1] as f32,
        };
        self.texture = Some(ctx.load_texture(
            &self.name,
            ColorImage::from_rgba_unmultiplied(texture_size, &rgb888s_to_texture(&colors)),
            TextureFilter::Nearest,
        ));
    }
}

fn rgb888s_to_texture(colors: &[Rgb888]) -> Vec<u8> {
    colors.into_iter().fold(Vec::new(), |mut pixels, color| {
        pixels.push(color.r);
        pixels.push(color.g);
        pixels.push(color.b);
        pixels.push(255);
        pixels
    })
}
