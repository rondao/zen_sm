use eframe::{
    egui::{Context, Image, TextureFilter, Ui},
    epaint::{Color32, ColorImage, Rect, TextureHandle, Vec2},
};
use zen::graphics::Rgb888;

pub struct Texture {
    name: String,
    size: Vec2,
    pub texture: Option<TextureHandle>,
    pub image: Option<ColorImage>,
}

impl Texture {
    pub fn new(name: String) -> Self {
        Self {
            name,
            texture: None,
            size: Vec2::default(),
            image: None,
        }
    }

    pub fn ui(&self, ui: &mut Ui, widget_rect: Rect) {
        if let Some(texture) = &self.texture {
            ui.put(widget_rect, Image::new(texture, widget_rect.size()));
        }
    }

    pub fn load_colors(&mut self, ctx: &Context, colors: Vec<Rgb888>, texture_size: [usize; 2]) {
        self.load_image(
            ctx,
            ColorImage::from_rgba_unmultiplied(texture_size, &rgb888s_to_texture(&colors)),
        );
    }

    pub fn load_image(&mut self, ctx: &Context, image: ColorImage) {
        self.size = Vec2 {
            x: image.size[0] as f32,
            y: image.size[1] as f32,
        };
        self.image = Some(image.clone());
        self.texture = Some(ctx.load_texture(&self.name, image, TextureFilter::Nearest));
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
        self.size
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
