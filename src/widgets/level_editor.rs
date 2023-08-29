use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use eframe::{
    egui::{Context, Response, TextureOptions, Ui},
    epaint::{ColorImage, Pos2, Rect},
};
use zen::{
    graphics::{gfx::GFX_TILE_WIDTH, IndexedColor, Palette},
    super_metroid::{
        level_data::{Block, BtsBlock, LevelData},
        tile_table::BLOCK_SIZE,
    },
};

use crate::app::BtsTile;

use super::helpers::{
    editor::{Command, Editor},
    texture::Texture,
};

const SELECTION_SIZE: [f32; 2] = [GFX_TILE_WIDTH as f32, GFX_TILE_WIDTH as f32];

pub struct LevelEditor {
    pub editor: Editor,
    bts_layer: Texture,
    bts_icons: Arc<Mutex<HashMap<BtsTile, ColorImage>>>,
    draw_bts: bool,
    edit_selection: BlockSelection,
}

pub struct BlockSelection {
    pub data: Vec<(Block, u8)>,
    pub indexed_colors: Vec<IndexedColor>,
    pub rect: Rect,
}

impl Default for BlockSelection {
    fn default() -> Self {
        Self {
            data: Vec::default(),
            indexed_colors: Vec::default(),
            rect: Rect::NOTHING,
        }
    }
}

impl LevelEditor {
    pub fn new(bts_icons: Arc<Mutex<HashMap<BtsTile, ColorImage>>>) -> Self {
        Self {
            editor: Editor::new("Level", SELECTION_SIZE),
            bts_layer: Texture::new("BtsLayer_LevelEditor".to_string()),
            bts_icons,
            draw_bts: true,
            edit_selection: BlockSelection::default(),
        }
    }
}

impl LevelEditor {
    pub fn ui(
        &mut self,
        ui: &mut Ui,
        level: &mut LevelData,
        palette: &Palette,
    ) -> (Response, Rect, Option<Command>) {
        let (widget_response, widget_rect, command) = self.editor.ui(ui);

        match command {
            Some(Command::Selection(selection, ref indexed_colors)) => self.set_selection(
                ui.ctx(),
                BlockSelection {
                    data: self.extract_selected_tiles(level, selection),
                    indexed_colors: indexed_colors.clone(),
                    rect: selection,
                },
                palette,
            ),
            Some(Command::Apply(position)) => {
                self.apply_edit_selection(level, position, palette);
            }
            None => (),
        }

        if ui.input(|i| i.key_pressed(eframe::egui::Key::H)) {
            self.draw_bts = !self.draw_bts
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

    pub fn apply_edit_selection(
        &mut self,
        level: &mut LevelData,
        position: Pos2,
        palette: &Palette,
    ) {
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
        self.editor.edit_texture(
            position,
            self.edit_selection.rect.width(),
            &self.edit_selection.indexed_colors,
            palette,
        );

        // Collect bts icons to draw.
        let bts_icons = self.edit_selection.data.iter().map(|(block, bts_block)| {
            self.bts_icons
                .lock()
                .unwrap()
                .get(&BtsTile {
                    block_type: block.block_type,
                    bts_block: *bts_block,
                })
                .cloned()
        });

        // Draw them onto bts texture.
        let selection_height_in_blocks = self.edit_selection.rect.height() as usize;
        for (i, bts_icon) in bts_icons.enumerate() {
            if let Some(bts_icon) = bts_icon {
                self.bts_layer.texture.as_mut().unwrap().set_partial(
                    [
                        (position.x as usize + (i / selection_height_in_blocks)) * BLOCK_SIZE,
                        (position.y as usize + (i % selection_height_in_blocks)) * BLOCK_SIZE,
                    ],
                    bts_icon.clone(),
                    TextureOptions::NEAREST,
                );
            }
        }
    }

    pub fn apply_colors(&mut self, palette: &Palette) {
        self.editor.apply_colors(palette);
    }

    pub fn clear_selection(&mut self) {
        self.edit_selection = BlockSelection::default();
        self.editor.clear_selection();
    }

    pub fn load_level(
        &mut self,
        ctx: &Context,
        level_data: &LevelData,
        indexed_colors: Vec<zen::graphics::IndexedColor>,
        palette: Palette,
        texture_size: [usize; 2],
    ) {
        self.editor.set_size(texture_size);
        self.editor
            .load_colors(ctx, indexed_colors, &palette, texture_size);

        self.bts_layer.texture = Some(ctx.load_texture(
            "BTS Texture",
            ColorImage::from_rgba_unmultiplied(
                texture_size,
                &vec![0; texture_size[0] * texture_size[1] * 4],
            ),
            TextureOptions::NEAREST,
        ));

        let bts_icons =
            level_data
                .layer1
                .iter()
                .zip(level_data.bts.iter())
                .map(|(block, bts_block)| {
                    self.bts_icons
                        .lock()
                        .unwrap()
                        .get(&BtsTile {
                            block_type: block.block_type,
                            bts_block: *bts_block,
                        })
                        .or_else(|| {
                            println!(
                                "Block/BTS not found: ({:?} - {:#x})",
                                block.block_type, bts_block
                            );
                            None
                        })
                        .cloned()
                });

        let x_blocks = texture_size[0] / BLOCK_SIZE;
        for (i, bts_icon) in bts_icons.enumerate() {
            if let Some(bts_icon) = bts_icon {
                self.bts_layer.texture.as_mut().unwrap().set_partial(
                    [(i % x_blocks) * BLOCK_SIZE, (i / x_blocks) * BLOCK_SIZE],
                    bts_icon.clone(),
                    TextureOptions::NEAREST,
                );
            }
        }
    }

    pub fn set_selection(
        &mut self,
        ctx: &Context,
        block_selection: BlockSelection,
        palette: &Palette,
    ) {
        self.editor.set_selection(
            ctx,
            &block_selection.indexed_colors,
            block_selection.rect,
            palette,
        );
        self.edit_selection = block_selection;
    }
}
