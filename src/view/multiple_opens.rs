// mochou-p/editerm/src/view/multiple_opens.rs

use std::path::PathBuf;
use spliterm::{PaneView, PaneCommand, Event};
use spliterm::betterm;
use betterm::color::rgb;
use betterm::terminal::{KeyboardEvent, Key, MouseEvent, MouseButtonEvent, MouseButton};
use betterm::styled_printer::StyledPrinter;
use crate::{ViewEvent, In, Out, ColoredText};
use crate::config::Theme;
use crate::utils::Utf8;


#[derive(Clone)]
pub struct MultipleOpens {
    path: PathBuf
}

impl MultipleOpens {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl PaneView<ViewEvent, Theme, In, Out> for MultipleOpens {
    fn custom(&self, theme: In) -> Out {
        if let Some(theme) = theme {
            ColoredText { fg: theme.red,        text: String::from("file locked") }
        } else {
            ColoredText { fg: rgb(255, 0, 255), text: String::from("file locked") }
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
                let filename = self.path.display().to_string();
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
                let text = "Cannot open file, because it is already open.";
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
                let text = " Ok ";
                let  len = text.len();
                let  pad = w - len;
                let lpad = pad / 2;
                let rpad = pad - lpad;

                sp
                    .text(" ".repeat(lpad))
                    .with_bg(theme.cyan, |sp| sp.fg(theme.background_disabled, text))
                    .text(" ".repeat(rpad))
            })
        } else {
            sp.bg(theme.background, " ".repeat(w))
        }
    }

    fn event(&mut self, event: Event, w: u16, h: u16) -> (PaneCommand<ViewEvent, Theme, In, Out>, ViewEvent) {
        let mid = h as usize / 2;

        match event {
            Event::Keyboard(KeyboardEvent::NoModifiers(Key::Enter | Key::Escape)) => {
                (PaneCommand::DoNothing, ViewEvent::CloseMe)
            },
            Event::Mouse(MouseEvent::Press(MouseButtonEvent::NoModifiers(MouseButton::Left(x, y)))) => {
                let (x, y) = (x as usize, y as usize);
                let len    = 4;
                let lpad   = (w as usize - len) / 2;

                if y == mid + 1 && x >= lpad && x < lpad + len {
                    (PaneCommand::DoNothing, ViewEvent::CloseMe)
                } else {
                    (PaneCommand::DoNothing, ViewEvent::DoNothing)
                }
            },
            _ => (PaneCommand::DoNothing, ViewEvent::DoNothing)
        }
    }
}
