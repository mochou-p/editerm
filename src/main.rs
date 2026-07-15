// mochou-p/editerm/src/main.rs

//mod config;
mod utils;
mod view;

use std::io::Write as _;
use std::panic::{set_hook, take_hook, catch_unwind, AssertUnwindSafe};
use std::sync::OnceLock;
use spliterm::Screen;
use spliterm::betterm;
use betterm::cursor;
use betterm::{reset, color::{ansi, fg}};
use betterm::terminal::{CtrlableChar, RawTerminal, Event, KeyboardEvent, Key, size};
use view::Browsing;


static PANIC_LOCATION: OnceLock<String> = OnceLock::new();
static PANIC_PAYLOAD:  OnceLock<String> = OnceLock::new();

fn main() {
    let was_ok = {
        Editor::new().run()
    };

    if !was_ok {
        eprintln!(
            "{}{} crashed! panic info:{}\n{}{}",
            fg(ansi::red().bright()),
            env!("CARGO_CRATE_NAME"),
            reset::fg(),
            PANIC_LOCATION.get().unwrap_or(&String::new()),
            PANIC_PAYLOAD .get().unwrap()
        );
    }
}

struct Editor {
    terminal: RawTerminal,
    screen:   Screen<ViewEvent>
}

#[derive(Default)]
pub enum ViewEvent {
    #[default]
    DoNothing,
    DrawCursor(u16, u16)
}

#[derive(Default, Clone)]
pub struct Cursor {
    last_x: isize,
    x:      isize,
    y:      isize
}

impl PartialEq<Self> for Cursor {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Editor {
    fn new() -> Self {
        let terminal        = RawTerminal::new(std::io::stdin(), std::io::stdout());
        let (width, height) = size(terminal.output.as_raw_fd());
        let screen          = Screen::new(0, 0, width, height, Browsing::new());

        Self { terminal, screen }
    }

    fn run(mut self) -> bool {
        set_hook(Box::new(|panic_info| {
            if let Some(location) = panic_info.location() {
                let _ = PANIC_LOCATION.set(
                    format!(
                        "{}:{}:{}:\n",
                        location.file(), location.line(), location.column()
                    )
                );
            }

            let payload = panic_info.payload();

            let _ = PANIC_PAYLOAD.set(
                if let Some(small_string) = payload.downcast_ref::<&str>() {
                    String::from(*small_string)
                } else if let Some(big_string) = payload.downcast_ref::<String>() {
                    big_string.clone()
                } else {
                    String::from("anonymous panic")
                }
            );
        }));

        let result = catch_unwind(AssertUnwindSafe(|| {
            self.inner_run();
        }));

        let _ = take_hook();

        result.is_ok()
    }

    fn inner_run(&mut self) {
        self.screen.render_all(&mut self.terminal);
        self.terminal.flush().unwrap();

        loop {
            let event = self.terminal.blocking_event();

            if let Event::Keyboard(keyboard_event) = &event {
                match keyboard_event {
                    KeyboardEvent::NoModifiers(Key::Escape) => {
                        break;
                    },
                    KeyboardEvent::CtrlChar(CtrlableChar::O) => {
                        self.screen.panes.vertical_split(Browsing::new());
                        self.screen.render_all(&mut self.terminal);
                        continue;
                    },
                    _ => ()
                }
            }

            if let Some(ViewEvent::DrawCursor(x, y)) = self.screen.event(event, &mut self.terminal) {
                self.terminal.write_all(
                    format!(
                        "{}{}",
                        cursor::goto(
                            x + 1 + self.screen.panes.focused_x().unwrap(),
                            y + 1 + self.screen.panes.focused_y().unwrap()
                        ),
                        "\x1b[?25h", // show cursor
                    ).as_bytes()
                ).unwrap();
            } else {
                self.terminal.write_all(
                    b"\x1b[?25l" // hide cursor
                ).unwrap();
            }

            self.terminal.flush().unwrap();
        }
    }
}
