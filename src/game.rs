use std::io::{Read, Write};

use termion::{clear, cursor, style};

use crate::board::Board;
use crate::info::{Info, InfoLayout};

const SCREEN_TOP: u16 = 1;
const SCREEN_LEFT: u16 = 1;

#[derive(PartialEq, Eq, Clone)]
pub enum GameState {
    Created = 0,
    Initialized,
    Started,
    Paused,
    Stopped,
}

pub struct Game<R, W: Write> {
    board: Option<Board>,
    info: Option<Info>,
    state: GameState,
    input: R,
    output: W,
}

impl<R, W: Write> Drop for Game<R, W> {
    fn drop(&mut self) {
        // Restore terminal defaults.
        write!(
            self.output,
            "{}{}{}",
            clear::All,
            style::Reset,
            cursor::Goto(1, 1)
        )
        .unwrap();
    }
}

impl<R: Read, W: Write> Game<R, W> {
    pub fn new(output: W, input: R) -> Game<R, W> {
        Game {
            output,
            input,
            board: None,
            info: None,
            state: GameState::Created,
        }
    }

    pub fn init(&mut self, board: Board, info: Option<Info>) {
        if self.state != GameState::Created && self.state != GameState::Stopped {
            panic!("You can initialize new or stopped game only.");
        }
        self.board = Some(board);
        self.info = info;
        self.layout();

        // Print initial screen
        write!(self.output, "{}", clear::All).unwrap();
        if let Some(ref board) = self.board {
            self.output.write(board.get_content().as_bytes()).unwrap();
        }
        if let Some(ref info) = self.info {
            self.output.write(info.get_content().as_bytes()).unwrap();
        }
        self.output.flush().unwrap();

        self.state = GameState::Initialized;
    }

    fn layout(&mut self) {
        if let Some(ref mut board) = self.board {
            if let Some(ref mut info) = self.info {
                let (b_w, b_h) = (board.get_width(), board.get_height());
                let (mut i_w, mut i_h) = (board.get_width(), board.get_height());
                let i_size = info.get_size();
                let (b_x, b_y, i_x, i_y) = match info.get_layout() {
                    InfoLayout::Left => {
                        i_w = i_size;
                        (i_w + 1, SCREEN_TOP, SCREEN_LEFT, SCREEN_TOP)
                    }
                    InfoLayout::Right => {
                        i_w = i_size;
                        (SCREEN_LEFT, SCREEN_TOP, b_w + 1, SCREEN_TOP)
                    }
                    InfoLayout::Top => {
                        i_h = i_size;
                        (SCREEN_LEFT, i_h + 1, SCREEN_LEFT, SCREEN_TOP)
                    }
                    InfoLayout::Bottom => {
                        i_h = i_size;
                        (SCREEN_LEFT, SCREEN_TOP, SCREEN_LEFT, b_h + 1)
                    }
                };
                board.set_position(b_x, b_y);
                info.set_position_and_size(i_x, i_y, i_w, i_h);
            } else {
                board.set_position(SCREEN_LEFT, SCREEN_TOP);
            };
        }
    }

    pub fn start(&mut self) {
        if self.state != GameState::Initialized && self.state != GameState::Stopped {
            panic!("You can start initialized or stopped game only.");
        }
        self.state = GameState::Started;
    }

    pub fn stop(&mut self) {
        if self.state != GameState::Started {
            panic!("You can stop started game only.");
        }
        self.state = GameState::Stopped;
    }

    pub fn pause(&mut self) {
        if self.state != GameState::Started {
            panic!("You can pause started game only.");
        }
        self.state = GameState::Paused;
    }

    pub fn resume(&mut self) {
        if self.state != GameState::Paused {
            panic!("You can resume paused game only.");
        }
        self.state = GameState::Started;
    }

    pub fn get_state(&self) -> GameState {
        self.state.clone()
    }
}
