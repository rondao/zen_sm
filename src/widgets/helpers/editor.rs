use eframe::{
    egui::{Context, Response, Ui},
    epaint::{ColorImage, Pos2, Rect, Vec2},
};
use zen::graphics::{IndexedColor, Palette};

use super::{
    drag_area::DragArea, indexed_texture::IndexedTexture,
    painted_selectable_area::PaintedSelectableArea, selectable_area::Selectable, texture::Texture,
};

pub struct Editor {
    drag_area: DragArea,
    selection: PaintedSelectableArea,
    pub texture_to_edit: IndexedTexture,
    selection_size: [f32; 2],
    selected_texture: Texture,
}

pub enum Command {
    Selection(Rect, ColorImage),
    Apply(Pos2),
}

impl Editor {
    pub fn new(name: &str, selection_size: [f32; 2]) -> Self {
        Self {
            drag_area: DragArea::default(),
            selection: PaintedSelectableArea::new([1.0, 1.0], selection_size),
            texture_to_edit: IndexedTexture::new(format!("Texture_To_Edit_{}", name)),
            selected_texture: Texture::new(format!("Selected_Texture_{}", name)),
            selection_size,
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) -> (Response, Rect, Option<Command>) {
        let (widget_rect, widget_response) = self.drag_area.create(ui, self.texture_to_edit.size());

        self.texture_to_edit.ui(ui, widget_rect);

        let Some(action) = self.selection.ui(ui, widget_rect, &widget_response) else {return (widget_response, widget_rect, None)};
        let command = match action {
            Selectable::SelectedHovering(selection) => {
                self.selected_texture.ui(ui, selection);
                None
            }
            Selectable::Selected(selection) => Some(Command::Selection(
                selection,
                self.crop_selection(selection)
                    .expect("Creating a selection without a texture?"),
            )),
            Selectable::Clicked(position) => Some(Command::Apply(position)),
            _ => None,
        };

        (widget_response, widget_rect, command)
    }

    pub fn crop_selection(&self, selection: Rect) -> Option<ColorImage> {
        let pixel_size_selection = Rect::from_min_max(
            (selection.min.to_vec2() * Vec2::from(self.selection_size)).to_pos2(),
            (selection.max.to_vec2() * Vec2::from(self.selection_size)).to_pos2(),
        );
        self.texture_to_edit.crop(pixel_size_selection)
    }

    pub fn set_selection(
        &mut self,
        ctx: &Context,
        image_selection: ColorImage,
        rect_selection: Rect,
    ) {
        self.selected_texture.load_image(ctx, image_selection);
        self.selection.set_selection(rect_selection);
    }

    pub fn set_size(&mut self, size: [usize; 2]) {
        self.selection
            .set_sizes([size[0] as f32, size[1] as f32], self.selection_size);
    }

    pub fn load_colors(
        &mut self,
        ctx: &Context,
        indexed_colors: Vec<IndexedColor>,
        palette: &Palette,
        texture_size: [usize; 2],
    ) {
        self.texture_to_edit
            .load_colors(ctx, indexed_colors, palette, texture_size);
    }

    pub fn apply_colors(&mut self, palette: &Palette) {
        self.texture_to_edit.apply_colors(palette);
    }

    pub fn clear_selection(&mut self) {
        self.selected_texture.texture = None;
    }
}
