use std::collections::{HashMap, VecDeque};

use eframe::{
    egui::{Context, Response, TextureFilter, Ui},
    epaint::{ColorImage, Pos2, Rect},
};
use zen::{
    graphics::{gfx::GFX_TILE_WIDTH, Palette},
    super_metroid::{
        level_data::{Block, BtsBlock, LevelData},
        tile_table::BLOCK_SIZE,
    },
};

use super::helpers::{
    editor::{Command, Editor},
    texture::Texture,
};

const SELECTION_SIZE: [f32; 2] = [GFX_TILE_WIDTH as f32, GFX_TILE_WIDTH as f32];

#[derive(Default, Debug, Hash, Eq, PartialEq)]
pub struct BtsTile {
    pub block_type: u8,
    pub bts_block: u8,
}

pub struct LevelEditor {
    pub editor: Editor,
    pub bts_layer: Texture,
    pub bts_icons: HashMap<BtsTile, ColorImage>,
    draw_bts: bool,
}

impl Default for LevelEditor {
    fn default() -> Self {
        Self {
            editor: Editor::new("Level", SELECTION_SIZE),
            bts_layer: Texture::new("BtsLayer_LevelEditor".to_string()),
            bts_icons: load_bts_icons(),
            draw_bts: true,
        }
    }
}

impl LevelEditor {
    pub fn ui(&mut self, ui: &mut Ui) -> (Response, Rect, Option<Command>) {
        let (widget_response, widget_rect, command) = self.editor.ui(ui);

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

        (widget_response, widget_rect, command)
    }

    pub fn apply_edit_selection(&mut self, level: &mut LevelData, selection: Rect, position: Pos2) {
        let width_in_blocks = self.editor.texture_to_edit.size().x as usize / BLOCK_SIZE;

        // Extract selected tiles data, to avoid overwriting them.
        let mut selected_tiles: VecDeque<(Block, BtsBlock)> = VecDeque::new();
        for x in (selection.min.x as usize)..(selection.max.x as usize) {
            for y in (selection.min.y as usize)..(selection.max.y as usize) {
                let index = x + y * width_in_blocks;
                selected_tiles.push_back((level.layer1[index], level.bts[index]));
            }
        }

        // Apply them to the level, from the extracted tiles.
        let index_cursor_position = (position.x as usize) + (position.y as usize) * width_in_blocks;
        for x in 0..(selection.max.x - selection.min.x) as usize {
            for y in 0..(selection.max.y - selection.min.y) as usize {
                let index = index_cursor_position + x + y * width_in_blocks;
                if let Some((layer1_block, bts)) = selected_tiles.pop_front() {
                    level.layer1[index] = layer1_block;
                    level.bts[index] = bts;
                }
            }
        }

        // Draw them onto texture.
        if let Some(texture) = self.editor.texture_to_edit.texture.texture.as_mut() {
            if let Some(gfx_image) = self.editor.texture_to_edit.texture.image.as_mut() {
                let click_pixel_position = [
                    position.x as usize * BLOCK_SIZE,
                    position.y as usize * BLOCK_SIZE,
                ];
                let selection_pixel_position = [
                    selection.min.x as usize * BLOCK_SIZE,
                    selection.min.y as usize * BLOCK_SIZE,
                ];

                let screen_width_in_pixels = texture.size()[0];
                let selection_width_in_pixels = (selection.width() as usize) * BLOCK_SIZE;

                let selection_size_in_pixels = selection.area() as usize * BLOCK_SIZE * BLOCK_SIZE;
                let selected_pixels: Vec<_> = (0..selection_size_in_pixels)
                    .map(|index| {
                        let x = selection_pixel_position[0] + (index % selection_width_in_pixels);
                        let y = selection_pixel_position[1] * screen_width_in_pixels
                            + (index / selection_width_in_pixels) * screen_width_in_pixels;

                        gfx_image.pixels[x + y]
                    })
                    .collect();

                for (index, pixel) in selected_pixels.iter().enumerate() {
                    let x = click_pixel_position[0] + (index % selection_width_in_pixels);
                    let y = click_pixel_position[1] * screen_width_in_pixels
                        + (index / selection_width_in_pixels) * screen_width_in_pixels;
                    gfx_image.pixels[x + y] = *pixel;
                }

                texture.set(gfx_image.clone(), TextureFilter::Nearest);
            }
        }
    }

    pub fn set_size(&mut self, ctx: &Context, size: [usize; 2]) {
        self.editor.set_size(size);
        self.bts_layer.texture = Some(ctx.load_texture(
            "BTS Texture",
            ColorImage::from_rgba_unmultiplied(size, &vec![0; size[0] * size[1] * 4]),
            TextureFilter::Nearest,
        ));
    }

    pub fn apply_colors(&mut self, palette: &Palette) {
        self.editor.apply_colors(palette);
    }

    pub fn clear_selection(&mut self) {
        self.editor.clear_selection();
    }

    pub fn load_colors(
        &mut self,
        ctx: &Context,
        indexed_colors: Vec<zen::graphics::IndexedColor>,
        palette: Palette,
        texture_size: [usize; 2],
    ) {
        self.editor
            .load_colors(ctx, indexed_colors, &palette, texture_size);
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
