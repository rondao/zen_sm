use std::fs;

use eframe::{
    egui::{self, Vec2},
    epi,
};
use zen::graphics::Palette;

use crate::widgets::{Widget, WithTexture};

pub struct ZenSM {
    palette: WithTexture<Palette>,
}

impl Default for ZenSM {
    fn default() -> Self {
        let palette = zen::graphics::palette::from_bytes(
            &fs::read("/home/rondao/dev/rust/snes_data/Crateria.tpl").unwrap(),
        )
        .unwrap();

        Self {
            palette: WithTexture {
                data: palette,
                texture_id: None,
            },
        }
    }
}

impl epi::App for ZenSM {
    fn name(&self) -> &str {
        "Zen SM"
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        if self.palette.texture_id.is_none() {
            self.palette.load_texture(frame);
        }

        egui::Area::new(1).show(ctx, |ui| {
            let response = self.palette.ui(
                ui,
                frame,
                Vec2 {
                    x: 600_f32,
                    y: 600_f32,
                },
            );
        });
    }
}
