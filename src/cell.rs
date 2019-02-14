use crate::board::ResourceTable;

use termion::{style, cursor};

#[derive(Clone)]
pub enum Cell {
    Empty,
    ResourceId(u16),
    Char(char),
    Content(String),
}

impl Cell {
    pub(crate) fn add_value_to_str(&self, dst: &mut String,
                                   resources: Option<&ResourceTable>) {
        match self {
            Cell::Empty => dst.push(' '),
            Cell::Char(c) => dst.push(*c),
            Cell::ResourceId(id) => {
                if let Some(ref rt) = resources {
                    let content = &rt[id];
                    dst.push_str(&format!("{}{}", content, style::Reset));
                } else {
                    panic!("If you use Cell::ResourceId, you must add resource table to Board.");
                }
            },
            Cell::Content(content) => dst.push_str(&format!("{}{}", content, style::Reset))
        };
    }

    pub(crate) fn get_content(&self, width: usize, height: usize, x: u16, y: u16,
                              resources: Option<&ResourceTable>) -> String {
        match self {
            Cell::Empty => Cell::prepare_str_from_char(' ', width, height, x, y),
            Cell::Char(c) => Cell::prepare_str_from_char(*c, width, height, x, y),
            Cell::ResourceId(id) => {
                if let Some(ref rt) = resources {
                    let content = &rt[id];
                    Cell::prepare_str(content, width, height, x, y)
                } else {
                    panic!("If you use Cell::ResourceId, you must add resource table to Board.");
                }
            },
            Cell::Content(content) => Cell::prepare_str(content, width, height, x, y)
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
        for (i, ch) in content.chars().enumerate() {
            if ch == CSI_SGR_START {
                is_csi = true;
            } else if is_csi && ch == CSI_SGR_END {
                is_csi = false;
            } else if !is_csi {
                ch_count += 1;
                if ch_count == width {
                    res.push_str(&content[line_start..i + 1]);
                    ch_count = 0;
                    line_start = i + 1;
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
