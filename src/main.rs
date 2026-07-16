// mochou-p/editerm/src/main.rs

//mod config;
mod utils;
mod view;

use std::io::Write as _;
use std::os::fd::AsRawFd;
use std::path::PathBuf;
use std::panic::{set_hook, take_hook, catch_unwind, AssertUnwindSafe};
use std::sync::OnceLock;
use spliterm::{Screen, FocusDirection};
use spliterm::betterm;
use betterm::cursor;
use betterm::{reset, color::{ansi, fg}};
use betterm::terminal::{RawTerminal, Event, MouseEvent, HoverEvent, KeyboardEvent, Key, size};
use view::{Browsing, Editing, Welcome};


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
    screen:   Screen<ViewEvent>,
    x:        u16,
    y:        u16,
    width:    u16,
    height:   u16,
    welcomed: bool
}

#[derive(Default, Clone)]
pub enum ViewEvent {
    #[default]
    DoNothing,
    DrawCursor(u16, u16),
    Welcomed
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
        let output             = std::io::stdout();
        let (x, y)             = (0, 0);
        let (width, height)    = size(output.as_raw_fd());
        let (screen, welcomed) = Self::screen_from_args(x, y, width, height);
        let terminal           = RawTerminal::new(std::io::stdin(), output);

        Self { terminal, screen, x, y, width, height, welcomed }
    }

    fn screen_from_args(x: u16, y: u16, width: u16, height: u16) -> (Screen<ViewEvent>, bool) {
        let args = std::env::args().collect::<Vec<String>>();

        match args.len() {
            1 => (Screen::new(0, 0, width, height, Welcome { scroll: 0 }), false),
            2 => {
                let path = PathBuf::from(&args[1]);

                if path.is_dir() {
                    (Screen::new(x, y, width, height, Browsing::new(Some(path))), true)
                } else if path.is_file() {
                    (Screen::new(x, y, width, height,  Editing::new(     path )), true)
                } else {
                    panic!("invalid path")
                }
            },
            _ => panic!(
                "invalid arguments\nusage: {}{}\n       {} <PATH>{}",
                fg(ansi::green()),
                args[0],
                args[0],
                reset::fg()
            )
        }
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
                    KeyboardEvent::Alt(Key::ArrowUp) => {
                        if self.welcomed {
                            if let Some((x, y)) = self.screen.panes.focus_direction(FocusDirection::Up) {
                                let (x, y) = {
                                    if
                                        x <   self.x                as i32
                                        ||
                                        x >= (self.x + self.width)  as i32
                                        ||
                                        y <   self.y                as i32
                                        ||
                                        y >= (self.y + self.height) as i32
                                    {
                                        continue;
                                    } else {
                                        (x as u16, y as u16)
                                    }
                                };

                                self.screen.event(
                                    Event::Mouse(MouseEvent::Hover(HoverEvent::NoModifiers(x, y))),
                                    &mut self.terminal,
                                    false
                                );
                                self.screen.render_all(&mut self.terminal);
                                self.terminal.flush().unwrap();
                            }
                            continue;
                        }
                    },
                    KeyboardEvent::Alt(Key::ArrowDown) => {
                        if self.welcomed {
                            if let Some((x, y)) = self.screen.panes.focus_direction(FocusDirection::Down) {
                                let (x, y) = {
                                    if
                                        x <   self.x                as i32
                                        ||
                                        x >= (self.x + self.width)  as i32
                                        ||
                                        y <   self.y                as i32
                                        ||
                                        y >= (self.y + self.height) as i32
                                    {
                                        continue;
                                    } else {
                                        (x as u16, y as u16)
                                    }
                                };

                                self.screen.event(
                                    Event::Mouse(MouseEvent::Hover(HoverEvent::NoModifiers(x, y))),
                                    &mut self.terminal,
                                    false
                                );
                                self.screen.render_all(&mut self.terminal);
                                self.terminal.flush().unwrap();
                            }
                            continue;
                        }
                    },
                    KeyboardEvent::Alt(Key::ArrowLeft) => {
                        if self.welcomed {
                            if let Some((x, y)) = self.screen.panes.focus_direction(FocusDirection::Left) {
                                let (x, y) = {
                                    if
                                        x <   self.x                as i32
                                        ||
                                        x >= (self.x + self.width)  as i32
                                        ||
                                        y <   self.y                as i32
                                        ||
                                        y >= (self.y + self.height) as i32
                                    {
                                        continue;
                                    } else {
                                        (x as u16, y as u16)
                                    }
                                };

                                self.screen.event(
                                    Event::Mouse(MouseEvent::Hover(HoverEvent::NoModifiers(x, y))),
                                    &mut self.terminal,
                                    false
                                );
                                self.screen.render_all(&mut self.terminal);
                                self.terminal.flush().unwrap();
                            }
                            continue;
                        }
                    },
                    KeyboardEvent::Alt(Key::ArrowRight) => {
                        if self.welcomed {
                            if let Some((x, y)) = self.screen.panes.focus_direction(FocusDirection::Right) {
                                let (x, y) = {
                                    if
                                        x <   self.x                as i32
                                        ||
                                        x >= (self.x + self.width)  as i32
                                        ||
                                        y <   self.y                as i32
                                        ||
                                        y >= (self.y + self.height) as i32
                                    {
                                        continue;
                                    } else {
                                        (x as u16, y as u16)
                                    }
                                };

                                self.screen.event(
                                    Event::Mouse(MouseEvent::Hover(HoverEvent::NoModifiers(x, y))),
                                    &mut self.terminal,
                                    false
                                );
                                self.screen.render_all(&mut self.terminal);
                                self.terminal.flush().unwrap();
                            }
                            continue;
                        }
                    },
                    KeyboardEvent::CtrlAlt(Key::ArrowUp) => {
                        if self.welcomed {
                            self.screen.panes.vertical_split(Browsing::new(None), true);
                            self.screen.render_all(&mut self.terminal);
                            self.terminal.flush().unwrap();
                            continue;
                        }
                    },
                    KeyboardEvent::CtrlAlt(Key::ArrowDown) => {
                        if self.welcomed {
                            self.screen.panes.vertical_split(Browsing::new(None), false);
                            self.screen.render_all(&mut self.terminal);
                            self.terminal.flush().unwrap();
                            continue;
                        }
                    },
                    KeyboardEvent::CtrlAlt(Key::ArrowLeft) => {
                        if self.welcomed {
                            self.screen.panes.horizontal_split(Browsing::new(None), true);
                            self.screen.render_all(&mut self.terminal);
                            self.terminal.flush().unwrap();
                            continue;
                        }
                    },
                    KeyboardEvent::CtrlAlt(Key::ArrowRight) => {
                        if self.welcomed {
                            self.screen.panes.horizontal_split(Browsing::new(None), false);
                            self.screen.render_all(&mut self.terminal);
                            self.terminal.flush().unwrap();
                            continue;
                        }
                    },
                    /*
                    KeyboardEvent::CtrlAltBackspace => {
                        if self.welcomed {
                            self.screen.panes.collapse();
                            self.screen.render_all(&mut self.terminal);
                            self.terminal.flush().unwrap();
                            continue;
                        }
                    },
                    */
                    _ => ()
                }
            }

            let custom = self.screen.event(event, &mut self.terminal, true);
            self.custom_event(custom);
            self.terminal.flush().unwrap();
        }
    }

    fn custom_event(&mut self, event: Option<ViewEvent>) {
        match event {
            Some(ViewEvent::DrawCursor(x, y)) => {
                self.terminal.write_all(
                    format!(
                        "{}{}",
                        cursor::goto(
                            x + 1 + self.screen.panes.focused_x().unwrap(),
                            y + 1 + self.screen.panes.focused_y().unwrap()
                        ),
                        "\x1b[?25h" // show cursor
                    ).as_bytes()
                ).unwrap();
            },
            Some(ViewEvent::Welcomed) => {
                self.screen = Screen::new(0, 0, self.width, self.height, Browsing::new(None));
                self.screen.render_all(&mut self.terminal);
                self.welcomed = true;
            },
            _ => {
                self.terminal.write_all(
                    b"\x1b[?25l" // hide cursor
                ).unwrap();
            }
        }
    }
}
