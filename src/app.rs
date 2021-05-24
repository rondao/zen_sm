use std::{
    borrow::{Borrow, BorrowMut},
    fs,
};

use eframe::{
    egui::{self, Vec2},
    epi,
};
use zen::graphics::Palette;

use crate::widgets::palette::Widget;

#[derive(Default)]
struct Rom {
    palette: Palette,
}

static mut ROM: Option<Rom> = None;

#[derive(Default)]
pub struct ZenSM<'a> {
    palette: Option<Widget<'a, Palette>>,
}

impl epi::App for ZenSM<'_> {
    fn name(&self) -> &str {
        "Zen SM"
    }

    fn setup(&mut self, _ctx: &egui::CtxRef) {
        let palette = zen::graphics::palette::from_bytes(
            &fs::read("/home/rondao/dev/rust/snes_data/Crateria.tpl").unwrap(),
        )
        .unwrap();

        unsafe {
            ROM = Some(Rom { palette });
            if let Some(rom) = ROM.borrow() {
                self.palette = Some(Widget {
                    palette: &rom.palette,
                    texture_id: None,
                });
            };
        };
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        if let Some(palette) = self.palette.borrow_mut() {
            egui::Area::new(1).show(ctx, |ui| {
                palette.ui(
                    ui,
                    frame,
                    Vec2 {
                        x: 600_f32,
                        y: 600_f32,
                    },
                );
            });
        } else {
            egui::CentralPanel::default().show(ctx, |ui| ui.label("Load a ROM."));
        }
    }
}
