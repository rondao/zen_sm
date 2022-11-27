use eframe::{
    egui::{Context, Ui},
    epaint::Rect,
};
use zen::{
    graphics::{gfx::GFX_TILE_WIDTH, IndexedColor, Palette},
    super_metroid::{
        level_data::{Block, BtsBlock},
        tile_table::BLOCK_SIZE,
    },
};

use super::{helpers::editor::Editor, level_editor::BlockSelection, Command};

const SELECTION_SIZE: [f32; 2] = [GFX_TILE_WIDTH as f32, GFX_TILE_WIDTH as f32];

pub enum TileTableCommand {
    Selected(BlockSelection),
    None,
}

pub struct TileTableEditor {
    editor: Editor,
}

impl Default for TileTableEditor {
    fn default() -> Self {
        Self {
            editor: Editor::new("TileTable", SELECTION_SIZE),
        }
    }
}

impl TileTableEditor {
    pub fn ui(&mut self, ui: &mut Ui) -> TileTableCommand {
        let (_, _, command) = self.editor.ui(ui);

        match command {
            Some(Command::Selection(selection, indexed_colors)) => {
                TileTableCommand::Selected(BlockSelection {
                    data: self.extract_selected_tiles(selection),
                    indexed_colors,
                    rect: selection,
                })
            }
            _ => TileTableCommand::None,
        }
    }

    fn extract_selected_tiles(&self, selection: Rect) -> Vec<(Block, BtsBlock)> {
        let width_in_blocks = self.editor.size().x as usize / BLOCK_SIZE;

        let mut selected_tiles: Vec<(Block, BtsBlock)> = Vec::new();
        for x in (selection.min.x as usize)..(selection.max.x as usize) {
            for y in (selection.min.y as usize)..(selection.max.y as usize) {
                let index = x + y * width_in_blocks;
                selected_tiles.push((
                    Block {
                        block_number: index as u16,
                        ..Default::default()
                    },
                    0,
                ));
            }
        }

        selected_tiles
    }

    pub fn load_texture(
        &mut self,
        ctx: &Context,
        tile_colors: Vec<IndexedColor>,
        palette: &Palette,
        texture_size: [usize; 2],
    ) {
        self.editor.set_size(texture_size);
        self.editor
            .load_colors(ctx, tile_colors, palette, texture_size);
    }

    pub fn apply_colors(&mut self, palette: &Palette) {
        self.editor.apply_colors(palette);
    }
}
