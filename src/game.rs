//! Main game object.

use std::io::{Read, Write};
use std::cell::RefCell;
use std::rc::{Rc, Weak};

use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;
use termion::input::{TermRead, Keys};
use termion::{cursor};
use termion::event::Key;

use crate::board::{Board, CellUpdates};
use crate::info::{Info, InfoLayout};
use crate::cursor::KeyHandleResult;

const SCREEN_TOP: usize = 1;
const SCREEN_LEFT: usize = 1;

/// Board position.
///
/// *x* (horizontal) and *y* (vertical) cell position on the board. Position is zero-based.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Position(pub usize, pub usize);

/// Game state.
#[derive(PartialEq, Eq, Copy, Clone)]
pub enum GameState {
    /// Initial game state.
    Created = 0,
    /// Game is initialized with board and information area (optional). The layout is complete.
    /// Board and information are displayed.
    Initialized,
    /// Game is started. Input from keyboard is listened.
    Started,
    /// Game is paused. All input from keyboard except resume key is ignored.
    Paused,
    /// Game is stopped. Input from keyboard is ignored.
    Stopped,
}

/// User input listener.
pub trait InputListener<R: Read, W: Write>
    where Self: Sized {
    /// This method is called when user press any key on keyboard.
    ///
    /// Since this library uses termion crate, keys from `termion::event::Key` are supported only.
    /// You can update game using `game` argument.
    fn handle_key(&mut self, key: Key, game: &mut Game<R, W, Self>);

    /// This method is called when user moved [`Cursor`]. Default implementation is empty. You
    /// don't need to implement it if you don't use [`Cursor`].
    ///
    /// The `position` is a new cursor position. You can update game using `game` argument.
    ///
    /// [`Cursor`]: ../cursor/struct.Cursor.html
    ///
    fn cursor_moved(&mut self, _position: Position, _game: &mut Game<R, W, Self>) {}
}

/// Main game object.
///
/// All interactions with the game should be done using its API.
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

    /// Creates new game object.
    ///
    /// # Arguments
    ///
    /// `input` - input stream.
    ///
    /// `output` - output stream.
    ///
    /// `listener` - user input listener.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::{self, Read, Write};
    /// use std::cell::RefCell;
    /// use std::rc::Rc;
    /// use termion::event::Key;
    /// use gameboard::{Game, InputListener};
    ///
    /// struct App {}
    ///
    /// impl<R: Read, W: Write> InputListener<R, W> for App {
    ///     fn handle_key(&mut self, key: Key, game: &mut Game<R, W, Self>) {
    ///         match key {
    ///             Key::Char('q') => game.stop(),
    ///             _ => {}
    ///         }
    ///     }
    /// }
    ///
    /// fn main() {
    ///     let stdout = io::stdout();
    ///     let stdout = stdout.lock();
    ///     let stdin = io::stdin();
    ///     let stdin = stdin.lock();
    ///
    ///     let app = Rc::new(RefCell::new(App {}));
    ///     let game = Rc::new(RefCell::new(Game::new(stdin, stdout, Rc::clone(&app))));
    /// }
    /// ```
    ///
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

    /// Creates new game object.
    ///
    /// This method is the same as [`new`] method, but for debug purposes only.
    /// The `new` method uses `termion::screen::AlternateScreen` for output, which switches to
    /// the alternate screen buffer of the terminal. When application crashes, terminal switches
    /// to the main screen buffer and all debug/crash output is wiped out. This method uses main
    /// screen buffer for output.
    ///
    /// [`new`]: #method.new
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
    /// Initializes game with board and information area (optional).
    ///
    /// This method sets layout. Board and information will be displayed on the screen.
    /// Game state will be set to `GameState::Initialized`.
    ///
    /// # Panics
    ///
    /// This method can be called in `GameState::Created` or `GameState::Stopped` states only.
    /// Panics if called in any other state.
    ///
    pub fn init(&mut self, board: Board, info: Option<Info>) {
        if self.state != GameState::Created && self.state != GameState::Stopped {
            panic!("You can initialize new or stopped game only.");
        }
        self.board = Some(board);
        self.info = info;
        self.layout();

        // Print initial screen
        if let Some(ref mut board) = self.board {
            self.output.write(board.get_border().as_bytes()).unwrap();
            if let Some(updates) = board.get_updates() {
                self.output.write(updates.as_bytes()).unwrap();
            }
        }
        if let Some(ref info) = self.info {
            self.output.write(info.get_border().as_bytes()).unwrap();
            if let Some(updates) = info.get_updates() {
                self.output.write(updates.as_bytes()).unwrap();
            }
        }
        self.output.flush().unwrap();

        self.state = GameState::Initialized;
    }

    // Layout board and information area on the screen.
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
                board.set_position(Position(b_x, b_y));
                info.set_position_and_size(Position(i_x, i_y), i_w, i_h);
            } else {
                board.set_position(Position(SCREEN_LEFT, SCREEN_TOP));
            };
        }
    }

    /// Starts listening user input.
    ///
    /// Game state will be set to `GameState::Started`.
    ///
    /// # Panics
    ///
    /// This method can be called in `GameState::Initialized` or `GameState::Stopped` states only.
    /// Panics if called in any other state. Also it panics if input listener object was dropped.
    ///
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
                    if let Some(ref mut board) = self.board {
                        // We pass key to board first. If board has cursor, it'll try to handle
                        // cursor movement and return new cursor position. Otherwise, user key
                        // handler will be called.
                        match board.handle_key(key) {
                            KeyHandleResult::NotHandled =>
                                listener.borrow_mut().handle_key(key, self),
                            KeyHandleResult::NewPosition(pos) =>
                                listener.borrow_mut().cursor_moved(pos, self),
                            KeyHandleResult::Consumed => {},
                        }
                    }
                }
                // Update screen.
                if let Some(ref mut board) = self.board {
                    if let Some(updates) = board.get_updates() {
                        self.output.write(updates.as_bytes()).unwrap();
                    }
                }
                if let Some(ref info) = self.info {
                    if let Some(updates) = info.get_updates() {
                        self.output.write(updates.as_bytes()).unwrap();
                    }
                }
                self.output.flush().unwrap();
            }
        } else {
            panic!("You cannot start game without listener. Listener was dropped.");
        };
    }

    /// Stops listening user input.
    ///
    /// Game state will be set to `GameState::Stopped`.
    ///
    /// # Panics
    ///
    /// This method can be called in `GameState::Started` state only.
    /// Panics if called in any other state.
    ///
    pub fn stop(&mut self) {
        if self.state != GameState::Started {
            panic!("You can stop started game only.");
        }
        self.state = GameState::Stopped;
    }

    /// Pauses listening user input (except resume key).
    ///
    /// Game state will be set to `GameState::Paused`. This method is added for convenience.
    /// The same functionality can easily be done on the client side.
    /// All key presses are ignored while in `GameState::Paused` state. Pressing
    /// resume key will call `handle_key`. User must call [`resume`] to get back to
    /// `GameState::Started` state.
    ///
    /// [`resume`]: #method.resume
    ///
    /// # Panics
    ///
    /// This method can be called in `GameState::Started` state only.
    /// Panics if called in any other state.
    ///
    pub fn pause(&mut self, resume_key: Key) {
        if self.state != GameState::Started {
            panic!("You can pause started game only.");
        }
        self.resume_key = Some(resume_key);
        self.state = GameState::Paused;
    }

    /// Starts listening all user input.
    ///
    /// Game state will be set to `GameState::Started`.
    ///
    /// # Panics
    ///
    /// This method can be called in `GameState::Paused` state only.
    /// Panics if called in any other state.
    ///
    pub fn resume(&mut self) {
        if self.state != GameState::Paused {
            panic!("You can resume paused game only.");
        }
        self.resume_key = None;
        self.state = GameState::Started;
    }

    /// Returns game state.
    pub fn get_state(&self) -> GameState {
        self.state
    }

    /// Updates cells content.
    ///
    /// # Panics
    ///
    /// Panics if message dialog is open.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let mut updates = CellUpdates::with_capacity(2);
    /// updates.push((Cell::Empty, Position(0, 1)));
    /// updates.push((Cell::Char('x'), Position(0, 2)));
    /// game.update_cells(updates);
    /// ```
    pub fn update_cells(&mut self, updates: CellUpdates) {
        if let Some(ref mut board) = self.board {
            board.update_cells(updates);
        }
    }

    /// Shows message dialog.
    ///
    /// This dialog can be used to ask user a questions. This dialog is modal. You can't update
    /// board cells while it is open. It can be closed by calling [`hide_message`].
    ///
    /// The dialog is displayed over the board. It will be centered automatically. It can't be
    /// larger than board: last lines will be ignored, too long lines will be truncated.
    ///
    /// *lines* is a list of strings to display in the dialog. If you want space between lines,
    /// add empty string to list.
    ///
    /// Text alignment:
    ///
    /// * Lines are left-aligned by default
    /// * Lines started with *|^|* are centered
    /// * Lines started with *|>|* are right-aligned
    ///
    /// # Implementation note
    ///
    /// This crate iterates Unicode strings as a set of [grapheme clusters] to handle characters
    /// like *g̈* correctly. When we slice strings to put them inside cells or dialogs, we expect
    /// characters to have the same width. This is not always true for some Unicode symbols. Such
    /// symbols will break layout.
    ///
    /// [`hide_message`]: #method.hide_message
    /// [grapheme clusters]: http://www.unicode.org/reports/tr29/
    ///
    /// # Examples
    ///
    /// ```no_run
    /// game.show_message(&[
    ///     "|^|Congratulations! You win!",
    ///     "",
    ///     "Press 'r' to replay.",
    ///     "Press 'q' to quit.",
    /// ]);
    /// ```
    pub fn show_message(&mut self, lines: &[&str]) {
        if let Some(ref mut board) = self.board {
            board.show_message(lines);
        }
    }

    /// Hides message dialog.
    pub fn hide_message(&mut self) {
        if let Some(ref mut board) = self.board {
            board.hide_message();
        }
    }
}
