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
