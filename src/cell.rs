use crate::board::ResourceTable;

#[derive(Clone)]
pub enum Cell {
    Empty,
    ResourceId(u16),
    Char(char),
    Content(String),
}

impl Cell {
    pub(crate) fn add_line_to_str(&self, dst: &mut String, width: usize, line_num: usize,
                                  resources: Option<&ResourceTable>) {
        match self {
            Cell::Empty => {
                dst.push_str(&" ".repeat(width));
            },
            Cell::Char(c) => {
                if width == 1 {
                    dst.push(*c);
                } else {
                    dst.push_str(&c.to_string().repeat(width))
                }
            },
            Cell::ResourceId(id) => {
                if let Some(ref rt) = resources {
                    let content = &rt[id];
                    dst.push_str(&Cell::get_line_from_content(content, width, line_num));
                } else {
                    panic!("If you use Cell::ResourceId, you must add resource table to Board.");
                }
            },
            Cell::Content(content) => {
                dst.push_str(&Cell::get_line_from_content(content, width, line_num));
            }
        };
    }

    fn get_line_from_content(content: &str, width: usize, line_num: usize) -> String {
        let mut res = String::with_capacity(width);
        res.push_str(&content[line_num * width..(line_num + 1) * width]);
        res
    }
}
