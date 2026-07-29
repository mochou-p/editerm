// mochou-p/editerm/src/view/save_dialog.rs

use spliterm::{PaneView, PaneCommand, Event};
use spliterm::betterm;
use betterm::color::rgb;
use betterm::terminal::{KeyboardEvent, Key, MouseEvent, HoverEvent, MouseButtonEvent, MouseButton};
use betterm::styled_printer::StyledPrinter;
use crate::ViewEvent;
use crate::{In, Out, ColoredText};
use crate::config::Theme;
use crate::view::{Editing, Browsing};
use crate::utils::Utf8;


#[derive(Clone)]
pub struct SaveDialog {
    editing: Editing,
    yes:     bool,
    yes_btn: (usize, usize),
    no_btn:  (usize, usize)
}

impl SaveDialog {
    pub fn new(editing: Editing) -> Self {
        Self { editing, yes: true, yes_btn: (0, 0), no_btn: (0, 0) }
    }

    fn btns_hovered(&mut self, mid: usize, x: u16, y: u16, pass: bool) -> (bool, bool) {
        let (x, y) = (x as usize, y as usize);

        if y == mid + 1 {
            if x >= self.yes_btn.0 && x < self.yes_btn.1 {
                if pass || !self.yes {
                    self.yes = true;
                    return (true, false);
                }
            } else if x >= self.no_btn.0 && x < self.no_btn.1 {
                if pass || self.yes {
                    self.yes = false;
                    return (false, true);
                }
            }
        }

        (false, false)
    }
}

impl PaneView<ViewEvent, Theme, In, Out> for SaveDialog {
    fn custom(&self, theme: In) -> Out {
        if let Some(theme) = theme {
            ColoredText { fg: theme.yellow,     text: String::from("save?") }
        } else {
            ColoredText { fg: rgb(255, 0, 255), text: String::from("save?") }
        }
    }

    fn pane_clone(&self) -> Box<dyn PaneView<ViewEvent, Theme, In, Out>> {
        Box::new(self.clone())
    }

    fn print_line(&mut self, i: usize, w: u16, h: u16, sp: StyledPrinter, theme: &Theme) -> StyledPrinter {
        let mid = h as usize / 2;
        let   w = w as usize;

        if i == mid - 2 {
            sp.with_bg(theme.background, |sp| {
                let filename = self.editing.path.display().to_string();
                let len      = filename.utf8_len() as usize;
                let pad      = w - len;
                let lpad     = pad / 2;
                let rpad     = pad - lpad;

                sp.fg(theme.foreground_disabled, format!(
                    "{}{filename}{}",
                    " ".repeat(lpad),
                    " ".repeat(rpad)
                ))
            })
        } else if i == mid - 1 {
            sp.with_bg(theme.background, |sp| {
                let text = "Cannot close a modified file, save it?";
                let len  = text.len();
                let pad  = w - len;
                let lpad = pad / 2;
                let rpad = pad - lpad;

                sp.fg(theme.foreground, format!(
                    "{}{text}{}",
                    " ".repeat(lpad),
                    " ".repeat(rpad)
                ))
            })
        } else if i == mid + 1 {
            sp.with_bg(theme.background, |sp| {
                let        left = " Yes ";
                let       space = "  ";
                let       right = " No ";
                let         len = left.len() + space.len() + right.len();
                let         pad = w - len;
                let        lpad = pad / 2;
                let        rpad = pad - lpad;

                self.yes_btn = (lpad,                   lpad + left.len());
                self. no_btn = (w - rpad - right.len(), w - rpad         );

                let  left_color = if  self.yes {
                    (theme.background_disabled, theme.green              )
                } else {
                    (theme.foreground,          theme.background_selected)
                };
                let right_color = if !self.yes {
                    (theme.background_disabled, theme.red                )
                } else {
                    (theme.foreground,          theme.background_selected)
                };

                sp
                    .text(" ".repeat(lpad))
                    .with_bg( left_color.1, |sp| sp.fg( left_color.0,  left))
                    .text(space)
                    .with_bg(right_color.1, |sp| sp.fg(right_color.0, right))
                    .text(" ".repeat(rpad))
            })
        } else {
            sp.bg(theme.background, " ".repeat(w))
        }
    }

    fn event(&mut self, event: Event, _w: u16, h: u16) -> (PaneCommand<ViewEvent, Theme, In, Out>, ViewEvent) {
        let mid = h as usize / 2;

        match event {
            Event::Keyboard(KeyboardEvent::NoModifiers(key)) => match key {
                Key::ArrowLeft => {
                    if !self.yes {
                        self.yes = true;
                        (PaneCommand::RerenderMe, ViewEvent::DoNothing)
                    } else {
                        (PaneCommand::DoNothing, ViewEvent::DoNothing)
                    }
                },
                Key::ArrowRight => {
                    if self.yes {
                        self.yes = false;
                        (PaneCommand::RerenderMe, ViewEvent::DoNothing)
                    } else {
                        (PaneCommand::DoNothing, ViewEvent::DoNothing)
                    }
                },
                Key::Enter => {
                    if self.yes {
                        self.editing.save();
                    }

                    (
                        PaneCommand::ReplaceMe(
                            Box::new(
                                Browsing::new(
                                    Some(self.editing.path.parent().unwrap().to_path_buf()),
                                    self.editing.i
                                )
                            )
                        ),
                        ViewEvent::DoNothing
                    )
                },
                Key::Escape => (PaneCommand::ReplaceMe(Box::new(self.editing.clone())), ViewEvent::DoNothing),
                _           => (PaneCommand::DoNothing,                                 ViewEvent::DoNothing)
            },
            Event::Mouse(mouse_event) => match mouse_event {
                MouseEvent::Hover(HoverEvent::NoModifiers(x, y)) => {
                    let btns = self.btns_hovered(mid, x, y, false);

                    if btns.0 || btns.1 {
                        (PaneCommand::RerenderMe, ViewEvent::DoNothing)
                    } else {
                        (PaneCommand::DoNothing,  ViewEvent::DoNothing)
                    }
                },
                MouseEvent::Press(MouseButtonEvent::NoModifiers(MouseButton::Left(x, y))) => {
                    let btns = self.btns_hovered(mid, x, y, true);

                    if btns.0 || btns.1 {
                        if btns.0 {
                            self.editing.save();
                        }

                        (
                            PaneCommand::ReplaceMe(
                                Box::new(
                                    Browsing::new(
                                        Some(self.editing.path.parent().unwrap().to_path_buf()),
                                        self.editing.i
                                    )
                                )
                            ),
                            ViewEvent::DoNothing
                        )
                    } else {
                        (PaneCommand::DoNothing,  ViewEvent::DoNothing)
                    }
                },
                _ => (PaneCommand::DoNothing, ViewEvent::DoNothing)
            }
            _ => (PaneCommand::DoNothing, ViewEvent::DoNothing)
        }
    }
}
