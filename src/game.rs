use std::io::{Read, Write};
use std::cell::RefCell;
use std::rc::{Rc, Weak};

use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;
use termion::input::{TermRead, Keys};
use termion::{cursor};
use termion::event::Key;

use crate::board::Board;
use crate::info::{Info, InfoLayout};

const SCREEN_TOP: u16 = 1;
const SCREEN_LEFT: u16 = 1;

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum GameState {
    Created = 0,
    Initialized,
    Started,
    Paused,
    Stopped,
}

pub trait InputListener<R: Read, W: Write>
    where Self: Sized {
    fn handle_key(&mut self, key: Key, game: &mut Game<R, W, Self>);
}

pub struct Game<R: Read, W: Write, L: InputListener<R, W>> {
    board: Option<Board>,
    info: Option<Info>,
    state: GameState,
    input: Keys<R>,
    output: W,
    listener: Weak<RefCell<L>>,
    resume_key: Option<Key>
}


impl<R: Read, W: Write, L: InputListener<R, W>> Drop for Game<R, W, L> {
    fn drop(&mut self) {
        write!(self.output, "{}", cursor::Show).unwrap();
        self.output.flush().unwrap();
    }
}

impl<R: Read, W: Write, L> Game<R, AlternateScreen<RawTerminal<W>>, L>
    where L: InputListener<R, AlternateScreen<RawTerminal<W>>> {

    pub fn new(input: R, output: W, listener: Rc<RefCell<L>>) -> Self {
        let mut alt_screen = AlternateScreen::from(output.into_raw_mode().unwrap());
        write!(alt_screen, "{}", cursor::Hide).unwrap();
        alt_screen.flush().unwrap();

        Game {
            input: input.keys(),
            output: alt_screen,
            listener: Rc::downgrade(&listener),
            board: None,
            info: None,
            state: GameState::Created,
            resume_key: None
        }
    }
}

impl<R: Read, W: Write, L> Game<R, RawTerminal<W>, L>
    where L: InputListener<R, RawTerminal<W>> {

    pub fn new_dbg(input: R, output: W, listener: Rc<RefCell<L>>) -> Self {
        let mut screen = output.into_raw_mode().unwrap();
        write!(screen, "{}", cursor::Hide).unwrap();
        screen.flush().unwrap();

        Game {
            input: input.keys(),
            output: screen,
            listener: Rc::downgrade(&listener),
            board: None,
            info: None,
            state: GameState::Created,
            resume_key: None
        }
    }
}

impl<R: Read, W: Write, L: InputListener<R, W>> Game<R, W, L> {
    pub fn init(&mut self, board: Board, info: Option<Info>) {
        if self.state != GameState::Created && self.state != GameState::Stopped {
            panic!("You can initialize new or stopped game only.");
        }
        self.board = Some(board);
        self.info = info;
        self.layout();

        // Print initial screen
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

        if let Some(listener) = self.listener.upgrade() {
            while self.state == GameState::Started || self.state == GameState::Paused {
                let key = match self.input.next() {
                    None => break,
                    Some(res) => match res {
                        Err(_) => continue,
                        Ok(c) => c
                    }
                };
                if self.state == GameState::Paused {
                    if let Some(resume_key) = self.resume_key {
                        if key == resume_key {
                            // In 'Paused' state we call key handler only if resume key is
                            // pressed. User should call resume().
                            listener.borrow_mut().handle_key(key, self);
                        }
                    }
                } else {
                    listener.borrow_mut().handle_key(key, self);
                }
            }
        } else {
            panic!("You cannot start game without listener. Listener was dropped.");
        };
    }

    pub fn stop(&mut self) {
        if self.state != GameState::Started {
            panic!("You can stop started game only.");
        }
        self.state = GameState::Stopped;
    }

    pub fn pause(&mut self, resume_key: Key) {
        if self.state != GameState::Started {
            panic!("You can pause started game only.");
        }
        self.resume_key = Some(resume_key);
        self.state = GameState::Paused;
    }

    pub fn resume(&mut self) {
        if self.state != GameState::Paused {
            panic!("You can resume paused game only.");
        }
        self.resume_key = None;
        self.state = GameState::Started;
    }

    pub fn get_state(&self) -> GameState {
        self.state
    }
}
