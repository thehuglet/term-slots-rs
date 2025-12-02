#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn lerp_hue(a: f32, b: f32, t: f32) -> f32 {
    let diff = (b - a).abs();

    if diff > 180.0 {
        if a < b {
            (a + 360.0) * (1.0 - t) + b * t
        } else {
            a * (1.0 - t) + (b + 360.0) * t
        }
    } else {
        a * (1.0 - t) + b * t
    }
    .rem_euclid(360.0)
}

pub fn iter_some<T>(vec: &[Option<T>]) -> impl Iterator<Item = (usize, &T)> {
    vec.iter()
        .enumerate()
        .filter_map(|(i, maybe)| maybe.as_ref().map(|item| (i, item)))
}

pub fn iter_some_mut<T>(vec: &mut [Option<T>]) -> impl Iterator<Item = (usize, &T)> {
    vec.iter_mut()
        .enumerate()
        .filter_map(|(i, maybe)| maybe.as_ref().map(|item| (i, item)))
}

pub fn center_text_unicode(text: String, width: usize) -> String {
    use unicode_width::UnicodeWidthStr;

    let text_width = text.width();
    if text_width >= width {
        return text.to_string();
    }

    let left_padding = (width - text_width) / 2;
    let right_padding = width - text_width - left_padding;

    format!(
        "{}{}{}",
        " ".repeat(left_padding),
        text,
        " ".repeat(right_padding)
    )
}
