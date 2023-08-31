use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use eframe::epaint::ColorImage;
use zen::super_metroid::{level_data::BlockType, tile_table::BLOCK_SIZE};

#[derive(Default, Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct BtsTile {
    pub block_type: BlockType,
    pub bts_block: u8,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    async fn fetch_image(s: &str) -> wasm_bindgen::JsValue;
}

#[cfg(target_arch = "wasm32")]
pub fn load_bts_icons(editor_assets: Arc<Mutex<HashMap<BtsTile, ColorImage>>>) {
    wasm_bindgen_futures::spawn_local(async move {
        let path = "/images/1_13.png";

        let js_buffer: js_sys::Uint8Array = fetch_image(path).await.into();

        let mut buffer = vec![0; js_buffer.length() as usize];
        js_buffer.copy_to(&mut buffer);

        let result = image::load_from_memory(&buffer);
        if let Ok(image) = result {
            let size = [image.width() as _, image.height() as _];
            let image_buffer = image.to_rgba8();
            let pixels = image_buffer.as_flat_samples();

            let file_name = path.split("/").last().unwrap().replace(".png", "");
            let mut splitted_file_name = file_name.split("_");

            let bts_tile = BtsTile {
                block_type: u8::from_str_radix(splitted_file_name.next().unwrap(), 16)
                    .unwrap()
                    .into(),
                bts_block: u8::from_str_radix(splitted_file_name.next().unwrap(), 16).unwrap(),
            };

            let bts_icon = ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());

            if bts_tile.block_type == BlockType::Slope {
                editor_assets.lock().unwrap().insert(
                    BtsTile {
                        block_type: bts_tile.block_type,
                        bts_block: bts_tile.bts_block | 0b01_0_00000,
                    },
                    image_mirror_horizontally(&bts_icon),
                );
                editor_assets.lock().unwrap().insert(
                    BtsTile {
                        block_type: bts_tile.block_type,
                        bts_block: bts_tile.bts_block | 0b10_0_00000,
                    },
                    image_mirror_vertically(&bts_icon),
                );
                editor_assets.lock().unwrap().insert(
                    BtsTile {
                        block_type: bts_tile.block_type,
                        bts_block: bts_tile.bts_block | 0b11_0_00000,
                    },
                    image_mirror_horizontally(&image_mirror_vertically(&bts_icon)),
                );
            }
            editor_assets.lock().unwrap().insert(bts_tile, bts_icon);
        }
    });
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_bts_icons(editor_assets: Arc<Mutex<HashMap<BtsTile, ColorImage>>>) {
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

        if bts_tile.block_type == BlockType::Slope {
            editor_assets.lock().unwrap().insert(
                BtsTile {
                    block_type: bts_tile.block_type,
                    bts_block: bts_tile.bts_block | 0b01_0_00000,
                },
                image_mirror_horizontally(&bts_icon),
            );
            editor_assets.lock().unwrap().insert(
                BtsTile {
                    block_type: bts_tile.block_type,
                    bts_block: bts_tile.bts_block | 0b10_0_00000,
                },
                image_mirror_vertically(&bts_icon),
            );
            editor_assets.lock().unwrap().insert(
                BtsTile {
                    block_type: bts_tile.block_type,
                    bts_block: bts_tile.bts_block | 0b11_0_00000,
                },
                image_mirror_horizontally(&image_mirror_vertically(&bts_icon)),
            );
        }

        editor_assets.lock().unwrap().insert(bts_tile, bts_icon);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn load_image_from_path(path: &std::path::Path) -> Result<ColorImage, image::ImageError> {
    let image = image::io::Reader::open(path)?.decode()?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    Ok(ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()))
}

fn image_mirror_horizontally(image: &ColorImage) -> ColorImage {
    let mut output = image.clone();
    for (row_number, row_pixels) in image.pixels.chunks(BLOCK_SIZE).enumerate() {
        for (index, pixel) in row_pixels.iter().rev().enumerate() {
            output.pixels[row_number * BLOCK_SIZE + index] = *pixel;
        }
    }
    output
}

fn image_mirror_vertically(image: &ColorImage) -> ColorImage {
    let mut output = image.clone();
    for (row_number, row_pixels) in image.pixels.chunks(BLOCK_SIZE).enumerate() {
        for (index, pixel) in row_pixels.iter().enumerate() {
            output.pixels[(BLOCK_SIZE - row_number - 1) * BLOCK_SIZE + index] = *pixel;
        }
    }
    output
}
