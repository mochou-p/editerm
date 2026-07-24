// mochou-p/editerm/src/view/editing/mod.rs

mod actions;
mod highlight;

use std::path::PathBuf;
use spliterm::{PaneView, PaneCommand, Event, PaneEvent};
use spliterm::betterm;
use betterm::styled_printer::StyledPrinter;
use betterm::terminal::{KeyboardEvent, Key, MouseEvent, MouseButtonEvent, MouseButton, CtrlableChar, ScrollEvent, ScrollDirection};
use crate::{Cursor, ViewEvent};
use crate::config::Theme;
use crate::utils::{ToWith, Utf8};
use crate::view::{Browsing, Welcome};


#[derive(Clone)]
pub struct Editing {
    path:       PathBuf,
    file:       File,
    scroll:     Vec2,
    pane_focus: bool,
    i:          Option<usize>
}

#[derive(Clone)]
pub struct File {
    clean:   bool,
    cursors: Vec<Cursor>,
    lines:   Vec<String>
}

#[derive(Clone, PartialEq)]
struct Vec2 {
    x: isize,
    y: isize
}

impl Editing {
    pub fn new(path: PathBuf, i: Option<usize>) -> Self {
        Self {
            file:       Self::read_file(&path),
            path,
            scroll:     Vec2 { x: 0, y: 0 },
            pane_focus: true,
            i
        }
    }

    fn read_file(path: &PathBuf) -> File {
        let     string = std::fs::read_to_string(path).unwrap();
        let mut lines  = string.lines().map(str::to_owned).collect::<Vec<String>>();

        if lines.is_empty() || string.ends_with('\n') {
            lines.push(String::new());
        }

        File {
            clean:   true,
            cursors: vec![Cursor { last_x: 0, x: 0, y: 0 }],
            lines
        }
    }

    fn cursor_visible_relative_position(&self) -> (isize, isize) {
        let cursor = &self.file.cursors[0];

        let x = cursor.x - self.scroll.x;
        let y = cursor.y - self.scroll.y;

        (x, y)
    }

    fn snap_to_cursor(&mut self, w: usize, h: usize) -> PaneCommand<ViewEvent, Theme> {
        let cursor     = self.file.cursors[0].clone();
        let old_scroll = self.scroll.clone();

        if cursor.y < self.scroll.y {
            self.scroll.y = cursor.y;
        } else if cursor.y > self.scroll.y + h as isize - 1 {
            self.scroll.y = cursor.y - h as isize + 1;
        }

        if cursor.x < self.scroll.x {
            self.scroll.x = cursor.x;
        } else if cursor.x > self.scroll.x + w as isize - 1 {
            self.scroll.x = cursor.x - w as isize + 1;
        }

        if self.scroll == old_scroll {
            PaneCommand::DoNothing
        } else {
            PaneCommand::RerenderMe
        }
    }

    // TODO: merge cursors (could make an outside function, and its relevant in other places too)
    fn warp_cursor(&mut self, x: u16, y: u16) -> PaneCommand<ViewEvent, Theme> {
        let old_cursor = self.file.cursors[0].clone();

        let y = {
            let line_count = self.file.lines.len() as isize;

            self.file.cursors.drain(1..);

            let cursor = &mut self.file.cursors[0];

            cursor.y = y as isize + self.scroll.y;
            cursor.x = x as isize + self.scroll.x;

            cursor.y.to_min_with(line_count - 1);
            cursor.y as usize
        };

        let line_len = self.file.lines[y].utf8_len();
        let cursor   = &mut self.file.cursors[0];

        cursor.x.to_min_with(line_len);
        cursor.last_x = x as isize + self.scroll.x;

        if *cursor == old_cursor {
            PaneCommand::DoNothing
        } else {
            PaneCommand::RerenderMe
        }
    }
}

impl PaneView<ViewEvent, Theme> for Editing {
    fn pane_clone(&self) -> Box<dyn PaneView<ViewEvent, Theme>> {
        Box::new(self.clone())
    }

    fn print_line(&self, i: usize, w: u16, _h: u16, sp: StyledPrinter, theme: &Theme) -> StyledPrinter {
        let i = i + self.scroll.y as usize;

        if i < self.file.lines.len() {
            let x = self.scroll.x;
            let y = i as isize;

            let line         = &self.file.lines[y as usize];
            let visible_line = line.utf8_range(x, x + w as isize);
            let cursor_line  = y == self.file.cursors[0].y;

            let bg = if cursor_line && self.pane_focus {
                theme.background_selected
            } else {
                theme.background
            };

            // TODO: this is wrong, colors can break on words cut by scroll.x, requires a new betterm feature
            sp.with_bg(bg, |sp| {
                self.highlight(sp, &visible_line, theme)
                    .text(" ".repeat((w as isize - visible_line.utf8_len()).max(0) as usize))
            })
        } else {
            sp.bg(theme.background_disabled, " ".repeat(w as usize))
        }
    }

    fn event(&mut self, event: Event, w: u16, h: u16) -> (PaneCommand<ViewEvent, Theme>, ViewEvent) {
        let w = w as usize;
        let h = h as usize;

        (
            match event {
                Event::Keyboard(keyboard_event) => match keyboard_event {
                    KeyboardEvent::NoModifiers(key) => match key {
                        Key::F1 => {
                            return (
                                PaneCommand::ReplaceMe(
                                    Box::new(
                                        Welcome::new(
                                            Some(self.pane_clone())
                                        )
                                    )
                                ),
                                ViewEvent::DoNothing
                            );
                        },
                        Key::Escape => {
                            return (
                                PaneCommand::ReplaceMe(
                                    Box::new(
                                        Browsing::new(
                                            Some(self.path.parent().unwrap().to_path_buf()),
                                            false,
                                            self.i
                                        )
                                    )
                                ),
                                ViewEvent::DoNothing
                            );
                        },
                        Key::ArrowLeft  => self.left           (w, h),
                        Key::ArrowRight => self.right          (w, h),
                        Key::ArrowUp    => self.up             (w, h),
                        Key::ArrowDown  => self.down           (w, h),
                        Key::Home       => self.line_start     (w, h),
                        Key::End        => self.line_end       (w, h),
                        Key::Delete     => self.erase_right    (w, h),
                        Key::Enter      => self.newline        (w, h),
                        Key::Tab        => self.tab            (w, h),
                        _               => PaneCommand::DoNothing
                    },
                    KeyboardEvent::Ctrl(key) => match key {
                        Key::ArrowLeft  => self.prev_word      (w, h),
                        Key::ArrowRight => self.next_word      (w, h),
                        Key::Home       => self.file_start     (w, h),
                        Key::End        => self.file_end       (w, h),
                        Key::Delete     => self.erase_next_word(w, h),
                        Key::ArrowUp    => self.scroll_dir     (  -5),
                        Key::ArrowDown  => self.scroll_dir     (   5),
                        _               => PaneCommand::DoNothing
                    },
                    KeyboardEvent::Alt(key) => match key {
                        Key::ArrowUp   => self.move_line_up    (w, h),
                        Key::ArrowDown => self.move_line_down  (w, h),
                        _              => PaneCommand::DoNothing
                    },
                    KeyboardEvent::Backspace                 => self.erase_left     (    w, h),
                    KeyboardEvent::CtrlBackspace             => self.erase_prev_word(    w, h),
                    KeyboardEvent::Char(ch)                  => self.character      (ch, w, h),
                    KeyboardEvent::CtrlChar(CtrlableChar::S) => { self.save(); PaneCommand::DoNothing },
                    _                                        =>                PaneCommand::DoNothing
                },
                Event::Mouse(mouse_event) => match mouse_event {
                    MouseEvent::Scroll(ScrollEvent::NoModifiers(scroll_direction)) => match scroll_direction {
                        ScrollDirection::Up  (_x, _y) => self.scroll_dir(-1),
                        ScrollDirection::Down(_x, _y) => self.scroll_dir( 1)
                    },
                    MouseEvent::Press(MouseButtonEvent::NoModifiers(MouseButton::Left(x, y))) => self.warp_cursor(x, y),
                    _                                                                         => PaneCommand::DoNothing
                },
                Event::Custom(pane_event) => {
                    match pane_event {
                        PaneEvent::FocusGained => { self.pane_focus =  true; },
                        PaneEvent::FocusLost   => { self.pane_focus = false; }
                    }

                    PaneCommand::RerenderMe
                },
                _ => PaneCommand::DoNothing
            },
            {
                let (x, y) = self.cursor_visible_relative_position();

                if x >= 0 && y >= 0 && x < w as isize && y < h as isize {
                    ViewEvent::DrawCursor(x as u16, y as u16)
                } else {
                    ViewEvent::DoNothing
                }
            }
        )
    }
}
