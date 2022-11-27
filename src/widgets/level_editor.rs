use std::collections::HashMap;

use eframe::{
    egui::{Context, Response, TextureFilter, Ui},
    epaint::{ColorImage, Pos2, Rect},
};
use zen::{
    graphics::{gfx::GFX_TILE_WIDTH, Palette},
    super_metroid::{
        level_data::{Block, BlockType, BtsBlock, LevelData},
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
    pub block_type: BlockType,
    pub bts_block: u8,
}

pub struct LevelEditor {
    pub editor: Editor,
    pub bts_layer: Texture,
    pub bts_icons: HashMap<BtsTile, ColorImage>,
    draw_bts: bool,
    edit_selection: BlockSelection,
}

pub struct BlockSelection {
    pub data: Vec<(Block, u8)>,
    pub image: ColorImage,
    pub rect: Rect,
}

impl Default for BlockSelection {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            image: ColorImage::default(),
            rect: Rect::NOTHING,
        }
    }
}

impl Default for LevelEditor {
    fn default() -> Self {
        Self {
            editor: Editor::new("Level", SELECTION_SIZE),
            bts_layer: Texture::new("BtsLayer_LevelEditor".to_string()),
            bts_icons: load_bts_icons(),
            draw_bts: true,
            edit_selection: BlockSelection::default(),
        }
    }
}

impl LevelEditor {
    pub fn ui(&mut self, ui: &mut Ui, level: &mut LevelData) -> (Response, Rect, Option<Command>) {
        let (widget_response, widget_rect, command) = self.editor.ui(ui);

        match command {
            Some(Command::Selection(selection, ref image_selection)) => self.set_selection(
                ui.ctx(),
                BlockSelection {
                    data: self.extract_selected_tiles(level, selection),
                    image: self
                        .editor
                        .crop_selection(selection)
                        .expect("Extracting selection without texture."),
                    rect: selection,
                },
                image_selection.clone(),
            ),
            Some(Command::Apply(position)) => {
                self.apply_edit_selection(level, position);
            }
            None => (),
        }

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

    fn extract_selected_tiles(&self, level: &mut LevelData, selection: Rect) -> Vec<(Block, u8)> {
        let width_in_blocks = self.editor.size().x as usize / BLOCK_SIZE;

        let mut selected_tiles: Vec<(Block, BtsBlock)> = Vec::new();
        for x in (selection.min.x as usize)..(selection.max.x as usize) {
            for y in (selection.min.y as usize)..(selection.max.y as usize) {
                let index = x + y * width_in_blocks;
                selected_tiles.push((level.layer1[index], level.bts[index]));
            }
        }

        selected_tiles
    }

    pub fn apply_edit_selection(&mut self, level: &mut LevelData, position: Pos2) {
        let width_in_blocks = self.editor.size().x as usize / BLOCK_SIZE;
        let mut selected_tiles = self.edit_selection.data.iter();

        // Apply them to the level, from the extracted tiles.
        let index_cursor_position = (position.x as usize) + (position.y as usize) * width_in_blocks;
        for x in 0..self.edit_selection.rect.width() as usize {
            for y in 0..self.edit_selection.rect.height() as usize {
                let index = index_cursor_position + x + y * width_in_blocks;
                if let Some((layer1_block, bts)) = selected_tiles.next() {
                    level.layer1[index] = *layer1_block;
                    level.bts[index] = *bts;
                }
            }
        }

        // Draw them onto texture.
        // TODO: We also need to draw onto the BTS layer.
        self.editor.edit_texture(
            position,
            self.edit_selection.rect.width(),
            &self.edit_selection.image,
        );
    }

    pub fn apply_colors(&mut self, palette: &Palette) {
        self.editor.apply_colors(palette);
    }

    pub fn clear_selection(&mut self) {
        self.edit_selection = BlockSelection::default();
        self.editor.clear_selection();
    }

    pub fn load_colors(
        &mut self,
        ctx: &Context,
        indexed_colors: Vec<zen::graphics::IndexedColor>,
        palette: Palette,
        texture_size: [usize; 2],
    ) {
        self.editor.set_size(texture_size);
        self.bts_layer.texture = Some(ctx.load_texture(
            "BTS Texture",
            ColorImage::from_rgba_unmultiplied(
                texture_size,
                &vec![0; texture_size[0] * texture_size[1] * 4],
            ),
            TextureFilter::Nearest,
        ));

        self.editor
            .load_colors(ctx, indexed_colors, &palette, texture_size);
    }

    pub fn set_selection(
        &mut self,
        ctx: &Context,
        block_selection: BlockSelection,
        image_selection: ColorImage,
    ) {
        self.editor
            .set_selection(ctx, image_selection, block_selection.rect);
        self.edit_selection = block_selection;
    }
}

fn load_bts_icons() -> HashMap<BtsTile, ColorImage> {
    let mut bts_icons = HashMap::new();

    let paths = std::fs::read_dir("images").unwrap();
    for path in paths {
        let path = path.unwrap().path();
        let mut splitted_file_name = path.file_stem().unwrap().to_str().unwrap().split("_");

        let bts_tile = BtsTile {
            block_type: u8::from_str_radix(splitted_file_name.next().unwrap(), 16)
                .unwrap()
                .into(),
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
