use zen::graphics::Rgb888;

pub fn rgb888s_to_rgba(colors: impl Iterator<Item = Rgb888>) -> Vec<u8> {
    colors.fold(Vec::new(), |mut pixels, color| {
        pixels.push(color.r);
        pixels.push(color.g);
        pixels.push(color.b);
        pixels.push(255);
        pixels
    })
}
