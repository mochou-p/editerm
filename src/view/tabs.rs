// mochou-p/editerm/src/view/tabs.rs

use spliterm::{PaneView, PaneCommand, Event};
use spliterm::betterm;
use betterm::terminal::{KeyboardEvent, Key, CtrlableChar, MouseEvent, MouseButtonEvent, MouseButton, ScrollEvent, ScrollDirection};
use betterm::color::RgbColor;
use betterm::styled_printer::StyledPrinter;
use crate::{ViewEvent, In, Out, ColoredText};
use crate::config::Theme;
use crate::utils::Utf8;
use crate::view::Browsing;


pub struct Tabs {
    views: Vec<Box<dyn PaneView<ViewEvent, Theme, In, Out>>>,
    view:  usize,
    pages: Vec<Vec<usize>>,
    page:  usize
}

impl Tabs {
    pub fn new(views: Vec<Box<dyn PaneView<ViewEvent, Theme, In, Out>>>) -> Self {
        Self { views, view: 0, pages: vec![vec![0]], page: 0 }
    }

    fn open(
        &mut self,
        view:     Box<dyn PaneView<ViewEvent, Theme, In, Out>>,
        focus_it: bool,
        w:        usize,
        theme:    Option<Theme>
    ) -> (PaneCommand<ViewEvent, Theme, In, Out>, ViewEvent) {
        self.views.push(view);

        if focus_it {
            self.view = self.views.len() - 1;
        }

        self.redo_pages(w, true, theme);

        (PaneCommand::RerenderMe, ViewEvent::DoNothing)
    }

    fn open_default(&mut self, focus_it: bool, w: usize, theme: Option<Theme>) -> (PaneCommand<ViewEvent, Theme, In, Out>, ViewEvent) {
        self.open(Box::new(Browsing::new(None, None)), focus_it, w, theme)
    }

    fn close_i(&mut self, i: usize, w: usize, theme: Option<Theme>) -> (PaneCommand<ViewEvent, Theme, In, Out>, ViewEvent) {
        self.views.remove(i);

        if self.views.is_empty() {
            (PaneCommand::DoNothing, ViewEvent::CloseMe)
        } else {
            if self.view > i {
                self.view -= 1;
            }
            self.view = self.view.min(self.views.len() - 1);
            self.redo_pages(w, true, theme);

            (PaneCommand::RerenderMe, ViewEvent::DoNothing)
        }
    }

    fn next(&mut self, w: usize, theme: Option<Theme>) -> (PaneCommand<ViewEvent, Theme, In, Out>, ViewEvent) {
        if self.views.len() == 1 {
            (PaneCommand::DoNothing, ViewEvent::DoNothing)
        } else {
            self.view = (self.view + 1) % self.views.len();
            self.redo_pages(w, true, theme);

            (PaneCommand::RerenderMe, ViewEvent::DoNothing)
        }
    }

    fn prev(&mut self, w: usize, theme: Option<Theme>) -> (PaneCommand<ViewEvent, Theme, In, Out>, ViewEvent) {
        if self.views.len() == 1 {
            (PaneCommand::DoNothing, ViewEvent::DoNothing)
        } else {
            let i     = (self.view as isize - 1).rem_euclid(self.views.len() as isize);
            self.view = i as usize;
            self.redo_pages(w, true, theme);

            (PaneCommand::RerenderMe, ViewEvent::DoNothing)
        }
    }

    fn nth(&mut self, i: usize, w: usize, theme: Option<Theme>) -> (PaneCommand<ViewEvent, Theme, In, Out>, ViewEvent) {
        let i = i.min(self.views.len() - 1);

        if self.view == i {
            (PaneCommand::DoNothing, ViewEvent::DoNothing)
        } else {
            self.view = i;
            self.redo_pages(w, true, theme);

            (PaneCommand::RerenderMe, ViewEvent::DoNothing)
        }
    }

    fn redo_pages(&mut self, w: usize, go_to_it: bool, theme: Option<Theme>) {
        let mut page_x = 0;
        let mut page   = 0;

        self.pages    = vec![vec![]];
        let   pos_len = self.page_pos().len();

        for i in 0..self.views.len() {
            let name_len = self.views[i].custom(theme.clone()).text.utf8_len() as usize + 2;

            if page_x + name_len + (3 * (i != self.views.len() - 1) as usize) + pos_len > w {
                self.pages.push(vec![]);
                page_x  = 3;
                page   += 1;
            }

            if go_to_it && i == self.view {
                self.page = page;
            }

            self.pages[page].push(i);

            page_x += name_len;
        }
    }

    fn mouse(&mut self, mouse_event: &MouseEvent, w: usize, theme: Option<Theme>) -> (PaneCommand<ViewEvent, Theme, In, Out>, ViewEvent) {
        match mouse_event {
            MouseEvent::Scroll(ScrollEvent::NoModifiers(ScrollDirection::Up(_x, _y))) => {
                return self.prev(w, theme);
            },
            MouseEvent::Scroll(ScrollEvent::NoModifiers(ScrollDirection::Down(_x, _y))) => {
                return self.next(w, theme);
            },
            MouseEvent::Press(MouseButtonEvent::NoModifiers(MouseButton::Left(x, _y))) => {
                let         x = *x as usize;
                let mut tab_x = 0;

                if self.pages.len() != 1 {
                    tab_x += self.page_pos().len();
                }

                if self.page != 0 {
                    if x < tab_x + 3 {
                        self.page -= 1;
                        return (PaneCommand::RerenderMe, ViewEvent::DoNothing);
                    }

                    tab_x += 3;
                }

                if self.page != self.pages.len() - 1 {
                    if x >= w - 3 && x < w {
                        self.page += 1;
                        return (PaneCommand::RerenderMe, ViewEvent::DoNothing);
                    }
                }

                for i in &self.pages[self.page] {
                    let view  = &self.views[*i];
                    let tab_w = view.custom(theme.clone()).text.utf8_len() as usize + 2;

                    if x >= tab_x && x < tab_x + tab_w {
                        if *i == self.view {
                            break;
                        } else {
                            self.view = *i;
                            return (PaneCommand::RerenderMe, ViewEvent::DoNothing);
                        }
                    }

                    tab_x += tab_w;
                }
            },
            MouseEvent::Press(MouseButtonEvent::NoModifiers(MouseButton::Middle(x, _y))) => {
                let         x = *x as usize;
                let mut tab_x = 0;

                if self.pages.len() != 1 {
                    tab_x += self.page_pos().len();
                }

                if self.page != 0 {
                    if x < tab_x + 3 {
                        return (PaneCommand::DoNothing, ViewEvent::DoNothing);
                    }

                    tab_x += 3;
                }

                if self.page != self.pages.len() - 1 {
                    if x >= w - 3 && x < w {
                        return (PaneCommand::DoNothing, ViewEvent::DoNothing);
                    }
                }

                for i in &self.pages[self.page] {
                    let view  = &self.views[*i];
                    let tab_w = view.custom(theme.clone()).text.utf8_len() as usize + 2;

                    if x >= tab_x && x < tab_x + tab_w {
                        return self.close_i(*i, w, theme);
                    }

                    tab_x += tab_w;
                }
            },
            _ => ()
        }

        (PaneCommand::DoNothing, ViewEvent::DoNothing)
    }

    fn page_pos(&self) -> String {
        format!(" {}/{} ", self.page + 1, self.pages.len())
    }

    fn left_arrow_bg(&self, theme: &Theme) -> RgbColor {
        if self.view < self.pages[self.page][0] {
            theme.background_selected
        } else {
            theme.background
        }
    }

    fn right_arrow_bg(&self, theme: &Theme) -> RgbColor {
        if self.view > self.pages[self.page][self.pages[self.page].len() - 1] {
            theme.background_selected
        } else {
            theme.background
        }
    }

    fn top_bar(&self, mut w: usize, mut sp: StyledPrinter, theme: &Theme) -> StyledPrinter {
        let mut printed_w = 0;

        if self.pages.len() != 1 {
            let pos    = self.page_pos();
            sp         = sp.with_bg(theme.background_disabled, |sp| sp.fg(theme.foreground_disabled, &pos));
            printed_w += pos.len();
        }

        if self.page != 0 {
            sp = sp.with_bg(self.left_arrow_bg(theme), |sp| {
                sp
                    .fg(theme.background_disabled, "▏")
                    .fg(theme.foreground_disabled, "< ")
            });

            printed_w += 3;
        }

        let not_last = self.page != self.pages.len() - 1;
        if  not_last {
            w -= 3;
        }

        for i in &self.pages[self.page] {
            let ColoredText { fg, text } = self.views[*i].custom(Some(theme.clone()));

            sp = sp.with_bg(if *i == self.view { theme.background_selected } else { theme.background }, |sp| {
                if self.pages.len() != 1 {
                    sp
                        .fg(theme.background_disabled, "▏")
                        .fg(fg,                        format!("{text} "))
                } else {
                    sp.fg(fg, format!(" {text} "))
                }
            });

            printed_w += text.utf8_len() as usize + 2;
        }

        sp = sp.bg(theme.background_disabled, " ".repeat(w - printed_w));

        if not_last {
            sp.with_bg(self.right_arrow_bg(theme), |sp| {
                sp
                    .fg(theme.background_disabled, "▏")
                    .fg(theme.foreground_disabled, "> ")
            })
        } else {
            sp
        }
    }
}

impl PaneView<ViewEvent, Theme, In, Out> for Tabs {
    fn pane_clone(&self) -> Box<dyn PaneView<ViewEvent, Theme, In, Out>> {
        let views = self.views.iter().map(|view| view.pane_clone()).collect();

        Box::new(Tabs { views, view: self.view, pages: self.pages.clone(), page: self.page })
    }

    fn print_line(&mut self, i: usize, w: u16, h: u16, sp: StyledPrinter, theme: &Theme) -> StyledPrinter {
        if i == 0 {
            self.top_bar(w as usize, sp, theme)
        } else {
            self.views[self.view].print_line(i - 1, w, h - 1, sp, theme)
        }
    }

    fn event(&mut self, mut event: Event, w: u16, h: u16) -> (PaneCommand<ViewEvent, Theme, In, Out>, ViewEvent) {
        let w = w as usize;

        match &mut event {
            Event::Keyboard(KeyboardEvent::Ctrl     (Key::Tab)) => { return self.next(w, None); },
            Event::Keyboard(KeyboardEvent::CtrlShift(Key::Tab)) => { return self.prev(w, None); },
            Event::Keyboard(KeyboardEvent::CtrlChar (      ch)) => match ch {
                CtrlableChar::T => { return self.open_default(           true, w, None); },
                CtrlableChar::W => { return self.     close_i(self.view,       w, None); },
                _               => ()
            },
            Event::Keyboard(KeyboardEvent::AltChar(ch)) => match ch {
                '1' => { return self.nth(0, w, None); },
                '2' => { return self.nth(1, w, None); },
                '3' => { return self.nth(2, w, None); },
                '4' => { return self.nth(3, w, None); },
                '5' => { return self.nth(4, w, None); },
                '6' => { return self.nth(5, w, None); },
                '7' => { return self.nth(6, w, None); },
                '8' => { return self.nth(7, w, None); },
                '9' => { return self.nth(8, w, None); },
                '0' => { return self.nth(9, w, None); },
                _   => ()
            },
            Event::Mouse(mouse_event) => {
                let (_x, y) = mouse_event.cell();

                if y == 0 {
                    return self.mouse(mouse_event, w, None);
                } else {
                    mouse_event.correct_by(0, 1);
                }
            },
            _ => ()
        }

        let (pane_command, mut view_event) = self.views[self.view].event(event, w as u16, h - 1);

        view_event = {
            match view_event {
                ViewEvent::CloseMe          => { return self.close_i(self.view,        w, None); },
                ViewEvent::DrawCursor(x, y) => ViewEvent::DrawCursor(x, y + 1),
                ViewEvent::Open(view)       => { return self.   open(     view, false, w, None); },
                original                    => original
            }
        };

        if let PaneCommand::ReplaceMe(replacement) = pane_command {
            self.views[self.view] = replacement;
            self.redo_pages(w, true, None);
            return (PaneCommand::RerenderMe, ViewEvent::DoNothing);
        }

        if matches!(pane_command, PaneCommand::RerenderMe) {
            // TODO: but a real tabname changed event would be better :D
            self.redo_pages(w, false, None);
        }

        (pane_command, view_event)
    }
}
