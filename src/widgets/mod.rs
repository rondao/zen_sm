use std::ops::Deref;

use eframe::{
    egui::{self, TextureId},
    epi,
};

pub mod palette;

pub struct WithTexture<T> {
    pub data: T,
    pub texture_id: Option<TextureId>,
}

impl<T> Deref for WithTexture<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

pub trait Widget {
    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        frame: &mut epi::Frame<'_>,
        size: egui::Vec2,
    ) -> egui::Response;
}
