use std::collections::HashMap;

use eframe::{
    egui::{Context, Response, TextureFilter, Ui},
    epaint::{ColorImage, Pos2, Rect, Vec2},
};
use zen::graphics::{gfx::GFX_TILE_WIDTH, Palette};

use super::helpers::{
    drag_area::DragArea, indexed_texture::IndexedTexture,
    painted_selectable_area::PaintedSelectableArea, selectable_area::Selectable, texture::Texture,
};

const SELECTION_SIZE: [f32; 2] = [GFX_TILE_WIDTH as f32, GFX_TILE_WIDTH as f32];

#[derive(Default, Debug, Hash, Eq, PartialEq)]
pub struct BtsTile {
    pub block_type: u8,
    pub bts_block: u8,
}

pub struct LevelEditor {
    drag_area: DragArea,
    selection: PaintedSelectableArea,
    pub gfx_layer: IndexedTexture,
    pub bts_layer: Texture,
    pub bts_icons: HashMap<BtsTile, ColorImage>,
    draw_bts: bool,
    selected_texture: Texture,
}

pub enum Command {
    Selection(Rect),
    Apply(Pos2),
}

impl Default for LevelEditor {
    fn default() -> Self {
        Self {
            drag_area: DragArea::default(),
            selection: PaintedSelectableArea::new([1.0, 1.0], SELECTION_SIZE),
            gfx_layer: IndexedTexture::new("Layer01_LevelEditor".to_string()),
            bts_layer: Texture::new("BtsLayer_LevelEditor".to_string()),
            bts_icons: load_bts_icons(),
            draw_bts: true,
            selected_texture: Texture::new("Selection_LevelEditor".to_string()),
        }
    }
}

impl LevelEditor {
    pub fn ui(&mut self, ui: &mut Ui) -> (Response, Rect, Option<Command>) {
        let (widget_rect, widget_response) = self.drag_area.create(ui, self.gfx_layer.size());

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

        let action = self.selection.ui(ui, widget_rect, &widget_response);
        if let Some(ref action) = action {
            match action {
                Selectable::SelectedHovering(selection) => self.selected_texture.ui(ui, *selection),
                Selectable::Selected(mut selection) => {
                    selection.max =
                        (selection.max.to_vec2() * Vec2::from(SELECTION_SIZE)).to_pos2();
                    selection.min =
                        (selection.min.to_vec2() * Vec2::from(SELECTION_SIZE)).to_pos2();

                    if let Some(selected_image) = self.gfx_layer.crop(selection) {
                        self.selected_texture.load_image(ui.ctx(), selected_image);
                    }
                }
                _ => (),
            }
        }

        let command = action.and_then(|action| match action {
            Selectable::Selected(selection) => Some(Command::Selection(selection)),
            Selectable::Clicked(position) => Some(Command::Apply(position)),
            _ => None,
        });

        (widget_response, widget_rect, command)
    }

    pub fn set_size(&mut self, ctx: &Context, size: [usize; 2]) {
        self.selection
            .set_sizes([size[0] as f32, size[1] as f32], SELECTION_SIZE);
        self.bts_layer.texture = Some(ctx.load_texture(
            "BTS Texture",
            ColorImage::from_rgba_unmultiplied(size, &vec![0; size[0] * size[1] * 4]),
            TextureFilter::Nearest,
        ));
    }

    pub fn apply_colors(&mut self, palette: &Palette) {
        self.gfx_layer.apply_colors(palette);
    }

    pub fn clear_selection(&mut self) {
        self.selected_texture.texture = None;
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
