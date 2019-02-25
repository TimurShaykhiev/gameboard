use std::rc::Rc;

use unicode_segmentation::UnicodeSegmentation;
use termion::{style, cursor, color};

use crate::board::ResourceTable;

const RESOURCE_TABLE_ERR_MSG: &'static str =
    "If you use Cell::ResourceId, you must add resource table to Board.";

#[derive(Clone)]
pub enum Cell {
    Empty,
    ResourceId(u16),
    Char(char),
    Content(String),
}

impl Cell {
    pub(crate) fn add_value_to_str(&self, dst: &mut String,
                                   resources: Rc<Option<ResourceTable>>) {
        match self {
            Cell::Empty => dst.push(' '),
            Cell::Char(c) => dst.push(*c),
            Cell::ResourceId(id) => {
                if let Some(rt) = resources.as_ref() {
                    let content = &rt[id];
                    dst.push_str(&format!("{}{}", content, style::Reset));
                } else {
                    panic!(RESOURCE_TABLE_ERR_MSG);
                }
            },
            Cell::Content(content) => dst.push_str(&format!("{}{}", content, style::Reset))
        };
    }

    pub(crate) fn get_content(&self, width: usize, height: usize, x: u16, y: u16,
                              resources: Rc<Option<ResourceTable>>) -> String {
        match self {
            Cell::Empty => Cell::prepare_str_from_char(' ', width, height, x, y),
            Cell::Char(c) => Cell::prepare_str_from_char(*c, width, height, x, y),
            Cell::ResourceId(id) => {
                if let Some(rt) = resources.as_ref() {
                    let content = &rt[id];
                    Cell::prepare_str(content, width, height, x, y)
                } else {
                    panic!(RESOURCE_TABLE_ERR_MSG);
                }
            },
            Cell::Content(content) => Cell::prepare_str(content, width, height, x, y)
        }
    }

    pub(crate) fn with_bg_color(&self, width: usize, height: usize,
                                resources: Rc<Option<ResourceTable>>,
                                bg_color: color::Rgb) -> Cell {
        match self {
            Cell::Empty =>
                Cell::Content(
                    format!("{}{}", color::Bg(bg_color), ' '.to_string().repeat(width * height))),
            Cell::Char(c) =>
                Cell::Content(
                    format!("{}{}", color::Bg(bg_color), (*c).to_string().repeat(width * height))),
            Cell::ResourceId(id) => {
                if let Some(rt) = resources.as_ref() {
                    let content = &rt[id];
                    Cell::Content(format!("{}{}", color::Bg(bg_color), content))
                } else {
                    panic!(RESOURCE_TABLE_ERR_MSG);
                }
            },
            Cell::Content(content) => Cell::Content(format!("{}{}", color::Bg(bg_color), content))
        }
    }

    // Fill cell with char and add Goto sequences.
    fn prepare_str_from_char(content: char, width: usize, height: usize,
                             x: u16, y: u16) -> String {
        let mut y = y;
        let mut res = String::with_capacity(width * height * 2);
        for _ in 0..height {
            res.push_str(
                &format!("{}{}", cursor::Goto(x, y), content.to_string().repeat(width)));
            y += 1;
        }
        res
    }

    // Split cell content string into lines and add Goto sequences. Add style reset at the end.
    fn prepare_str(content: &str, width: usize, height: usize, x: u16, y: u16) -> String {
        const CSI_SGR_START: char = '\x1b';
        const CSI_SGR_END: char = 'm';

        let mut res = String::with_capacity(content.len() * 2);
        // Set cursor to cell top left corner
        res.push_str(&cursor::Goto(x, y).to_string());

        let mut line_start = 0;
        let mut ch_count = 0;
        let mut is_csi = false;
        let mut y = y;
        let mut height = height;
        for (i, ch) in UnicodeSegmentation::grapheme_indices(content, true) {
            if ch.as_bytes()[0] as char == CSI_SGR_START {
                is_csi = true;
            } else if is_csi && ch.as_bytes()[0] as char == CSI_SGR_END {
                is_csi = false;
            } else if !is_csi {
                ch_count += 1;
                if ch_count == width {
                    res.push_str(&content[line_start..i + ch.len()]);
                    ch_count = 0;
                    line_start = i + ch.len();
                    y += 1;
                    height -= 1;
                    if height > 0 {
                        res.push_str(&cursor::Goto(x, y).to_string());
                    } else {
                        break;
                    }
                }
            }
        }
        // Reset all styles at the end
        res.push_str(&style::Reset.to_string());
        res
    }
}
