use unicode_segmentation::UnicodeSegmentation;

pub(crate) fn get_str_range(text: &str, start: usize, end: usize) -> &str {
    let mut iter = UnicodeSegmentation::grapheme_indices(text, true);
    let (s, _) = iter.nth(start).expect("Invalid string range index.");
    match iter.nth(end - start - 1) {
        Some((e, _)) => &text[s as usize..e as usize],
        None => &text[s as usize..]
    }
}

pub(crate) fn get_str_len(text: &str) -> usize {
    UnicodeSegmentation::graphemes(text, true).count()
}
