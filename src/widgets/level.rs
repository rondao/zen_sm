use std::collections::HashMap;

use eframe::{
    egui::{Context, Image, Response, TextureFilter, Ui},
    epaint::{ColorImage, Rect, TextureHandle, Vec2},
};
use zen::{graphics::gfx::GFX_TILE_WIDTH, super_metroid::room::Room};

use super::editor::Editor;

const SELECTION_SIZE: [f32; 2] = [GFX_TILE_WIDTH as f32, GFX_TILE_WIDTH as f32];

#[derive(Default, Debug, Hash, Eq, PartialEq)]
pub struct BtsTile {
    pub block_type: u8,
    pub bts_block: u8,
}

pub struct LevelEditor {
    pub editor: Editor,
    pub bts_texture: Option<TextureHandle>,
    pub bts_icons: HashMap<BtsTile, ColorImage>,
    draw_bts: bool,
}

impl Default for LevelEditor {
    fn default() -> Self {
        Self {
            editor: Editor::new("LevelEditor".to_string()),
            bts_texture: None,
            bts_icons: load_bts_icons(),
            draw_bts: true,
        }
    }
}

impl LevelEditor {
    pub fn ui(&mut self, ui: &mut Ui, room: &Room) -> (Response, Rect) {
        let size = room.size_in_pixels();
        let (response, widget_rect) = self.editor.ui(
            ui,
            size,
            SELECTION_SIZE,
            Vec2 {
                x: size[0] as f32,
                y: size[1] as f32,
            },
        );

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
            ui.put(
                widget_rect,
                Image::new(self.bts_texture.as_ref().unwrap(), widget_rect.size()),
            );
        }

        (response, widget_rect)
    }

    pub fn set_size(&mut self, ctx: &Context, size: [usize; 2]) {
        self.bts_texture = Some(ctx.load_texture(
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
