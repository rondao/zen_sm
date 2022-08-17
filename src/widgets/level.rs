use std::collections::HashMap;

use eframe::{
    egui::{Context, Response, Sense, TextureFilter, Ui},
    epaint::{ColorImage, Rect, Vec2},
};
use zen::super_metroid::room::Room;

use super::helpers::texture::Texture;

// const SELECTION_SIZE: [f32; 2] = [GFX_TILE_WIDTH as f32, GFX_TILE_WIDTH as f32];

#[derive(Default, Debug, Hash, Eq, PartialEq)]
pub struct BtsTile {
    pub block_type: u8,
    pub bts_block: u8,
}

pub struct LevelEditor {
    pub gfx_layer: Texture,
    pub bts_layer: Texture,
    pub bts_icons: HashMap<BtsTile, ColorImage>,
    draw_bts: bool,
}

impl Default for LevelEditor {
    fn default() -> Self {
        Self {
            gfx_layer: Texture::new("Layer01_LevelEditor".to_string()),
            bts_layer: Texture::new("BtsLayer_LevelEditor".to_string()),
            bts_icons: load_bts_icons(),
            draw_bts: true,
        }
    }
}

impl LevelEditor {
    pub fn ui(&mut self, ui: &mut Ui, room: &Room) -> (Response, Rect) {
        let widget_size = room.size_in_pixels();

        let (widget_rect, widget_response) = ui.allocate_exact_size(
            Vec2 {
                x: widget_size[0] as f32,
                y: widget_size[1] as f32,
            },
            Sense::click(),
        );
        self.gfx_layer.ui(ui, widget_rect);

        for event in &ui.input().events {
            match event {
                eframe::egui::Event::Key {
                    key: eframe::egui::Key::H,
                    pressed: true,
                    modifiers: _,
                } => self.draw_bts = !self.draw_bts,
                _ => {}
            }
        }

        if self.draw_bts {
            self.bts_layer.ui(ui, widget_rect);
        }

        (widget_response, widget_rect)
    }

    pub fn set_size(&mut self, ctx: &Context, size: [usize; 2]) {
        self.bts_layer.texture = Some(ctx.load_texture(
            "BTS Texture",
            ColorImage::from_rgba_unmultiplied(size, &vec![0; size[0] * size[1] * 4]),
            TextureFilter::Nearest,
        ));
    }
}

fn load_bts_icons() -> HashMap<BtsTile, ColorImage> {
    let mut bts_icons = HashMap::new();

    let paths = std::fs::read_dir("images").unwrap();
    for path in paths {
        let path = path.unwrap().path();
        let mut splitted_file_name = path.file_stem().unwrap().to_str().unwrap().split("_");

        let bts_tile = BtsTile {
            block_type: u8::from_str_radix(splitted_file_name.next().unwrap(), 16).unwrap(),
            bts_block: u8::from_str_radix(splitted_file_name.next().unwrap(), 16).unwrap(),
        };

        let bts_icon = load_image_from_path(&path).unwrap();

        bts_icons.insert(bts_tile, bts_icon);
    }

    bts_icons
}

fn load_image_from_path(path: &std::path::Path) -> Result<ColorImage, image::ImageError> {
    let image = image::io::Reader::open(path)?.decode()?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    Ok(ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()))
}
