use eframe::{
    egui::{Context, Response, Ui},
    epaint::{ColorImage, Pos2, Rect, Vec2},
};
use zen::graphics::{IndexedColor, Palette, Rgb888};

use super::{
    drag_area::DragArea, indexed_texture::IndexedTexture,
    painted_selectable_area::PaintedSelectableArea, selectable_area::Selectable, texture::Texture,
};

pub struct Editor {
    drag_area: DragArea,
    selection: PaintedSelectableArea,
    texture_to_edit: IndexedTexture,
    selection_size: [f32; 2],
    selected_texture: Texture,
}

pub enum Command {
    Selection(Rect, Vec<IndexedColor>),
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
                self.crop_selection(selection),
            )),
            Selectable::Clicked(position) => Some(Command::Apply(position)),
            _ => None,
        };

        (widget_response, widget_rect, command)
    }

    pub fn edit_texture(
        &mut self,
        position: Pos2,
        selection_width: f32,
        indexed_image: &Vec<IndexedColor>,
        palette: &Palette,
    ) {
        let click_pixel_position = [
            (position.x * self.selection_size[0]) as usize,
            (position.y * self.selection_size[1]) as usize,
        ];

        let screen_width_in_pixels = self.texture_to_edit.size()[0] as usize;
        let selection_width_in_pixels = (selection_width * self.selection_size[0]) as usize;

        let new_indexed_colors = &mut self.texture_to_edit.indexed_colors;
        for (index, indexed_color) in indexed_image.iter().enumerate() {
            let x = click_pixel_position[0] + (index % selection_width_in_pixels);
            let y = click_pixel_position[1] * screen_width_in_pixels
                + (index / selection_width_in_pixels) * screen_width_in_pixels;
            new_indexed_colors[x + y] = *indexed_color;
        }

        self.texture_to_edit.apply_colors(palette);
    }

    pub fn crop_selection(&self, selection: Rect) -> Vec<IndexedColor> {
        let pixel_size_selection = Rect::from_min_max(
            (selection.min.to_vec2() * Vec2::from(self.selection_size)).to_pos2(),
            (selection.max.to_vec2() * Vec2::from(self.selection_size)).to_pos2(),
        );
        self.texture_to_edit.crop(pixel_size_selection)
    }

    pub fn set_selection(
        &mut self,
        ctx: &Context,
        indexed_image: &Vec<IndexedColor>,
        rect_selection: Rect,
        palette: &Palette,
    ) {
        self.selected_texture.load_image(
            ctx,
            indexed_color_to_color_image(
                indexed_image,
                palette,
                [
                    (rect_selection.size().x * self.selection_size[0]) as usize,
                    (rect_selection.size().y * self.selection_size[1]) as usize,
                ],
            ),
        );
        self.selection.set_selection(rect_selection);
    }

    pub fn size(&self) -> Vec2 {
        self.texture_to_edit.size()
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

pub fn indexed_color_to_color_image(
    indexed_colors: &Vec<IndexedColor>,
    palette: &Palette,
    size: [usize; 2],
) -> ColorImage {
    let colors = indexed_colors
        .iter()
        .fold(Vec::new(), |mut output, idx_color| {
            let rgb888: Rgb888 =
                palette.sub_palettes[idx_color.sub_palette].colors[idx_color.index].into();
            output.push(rgb888.r);
            output.push(rgb888.g);
            output.push(rgb888.b);
            output.push(255);
            output
        });

    ColorImage::from_rgba_unmultiplied(size, &colors)
}
