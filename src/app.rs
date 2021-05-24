use std::fs;

use eframe::{
    egui::{self, Vec2},
    epi,
};
use zen::graphics;

use crate::widgets;

#[derive(Default)]
struct Rom {
    palette: graphics::Palette,
}

#[derive(Default)]
pub struct ZenSM {
    rom: Option<Rom>,
    palette: Option<widgets::Palette>,
}

impl epi::App for ZenSM {
    fn name(&self) -> &str {
        "Zen SM"
    }

    fn setup(&mut self, _ctx: &egui::CtxRef) {
        let palette = zen::graphics::palette::from_bytes(
            &fs::read("/home/rondao/dev/rust/snes_data/Crateria.tpl").unwrap(),
        )
        .unwrap();

        self.rom = Some(Rom { palette });
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        if let Some(rom) = &mut self.rom {
            if self.palette.is_none() {
                let mut widget_palette = widgets::Palette::default();
                widget_palette.load_texture(frame, &rom.palette);
                self.palette = Some(widget_palette);
            }

            if let Some(palette) = &mut self.palette {
                egui::Area::new(1).show(ctx, |ui| {
                    let response = palette.ui(
                        ui,
                        &mut rom.palette,
                        Vec2 {
                            x: 600_f32,
                            y: 600_f32,
                        },
                    );
                    if response.changed() {
                        palette.load_texture(frame, &rom.palette);
                    }
                });
            }
        } else {
            egui::CentralPanel::default().show(ctx, |ui| ui.label("Load a ROM."));
        }
    }
}
