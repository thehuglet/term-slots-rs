pub fn iter_some<T>(vec: &[Option<T>]) -> impl Iterator<Item = (usize, &T)> {
    vec.iter()
        .enumerate()
        .filter_map(|(i, maybe)| maybe.as_ref().map(|item| (i, item)))
}

pub fn iter_some_mut<T>(vec: &mut [Option<T>]) -> impl Iterator<Item = (usize, &mut T)> {
    vec.iter_mut()
        .enumerate()
        .filter_map(|(i, maybe)| maybe.as_mut().map(|item| (i, item)))
}
