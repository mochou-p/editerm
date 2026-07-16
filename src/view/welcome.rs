// mochou-p/editerm/src/view/welcome.rs

use spliterm::{PaneView, PaneCommand, Event};
use spliterm::betterm;
use betterm::color::rgb;
use betterm::terminal::{KeyboardEvent, Key, MouseEvent, ScrollEvent, ScrollDirection};
use betterm::styled_printer::StyledPrinter;
use crate::ViewEvent;
use crate::utils::Utf8;


pub struct Welcome {
    pub scroll: usize
}

impl PaneView<ViewEvent> for Welcome {
    fn pane_clone(&self) -> Box<dyn PaneView<ViewEvent>> {
        Box::new(Welcome { scroll: self.scroll })
    }

    fn print_line(&self, i: usize, w: u16, _h: u16, sp: StyledPrinter) -> StyledPrinter {
        let i = i + self.scroll;

        let bg  = rgb(0x1C, 0x1C, 0x1C);
        let fg  = rgb(0x80, 0x80, 0x80);
        let fgb = rgb(0xC9, 0xC9, 0xC9);
        let bfg = rgb(0x94, 0xD1, 0xFF);
        let bin = env!("CARGO_BIN_NAME");

        match i {
            1  => sp.with_bg(bg, |sp| sp.fg(bfg, fill(w, format!("  Welcome to {bin}! \\(^ヮ^)/")))),

            4  => sp.with_bg(bg, |sp| sp.fg(fgb, fill(w,         "  USAGE"))),
            5  => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w, format!("    {bin}                  = Open this welcome screen")))),
            6  => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w, format!("    {bin} <DIRECTORY_PATH> = Open a directory to browse")))),
            7  => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w, format!("    {bin} <     FILE_PATH> = Open a file      to edit")))),

            10 => sp.with_bg(bg, |sp| sp.fg(fgb, fill(w,         "  GLOBAL KEYBINDS"))),
            11 => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w, format!("    Escape                   = Exit {bin}")))),
            13 => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Alt + ArrowUp            = Focus the pane        above"))),
            14 => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Alt + ArrowDown          = Focus the pane        below"))),
            15 => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Alt + ArrowLeft          = Focus the pane to the  left"))),
            16 => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Alt + ArrowRight         = Focus the pane to the right"))),
            18 => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Ctrl + Alt + ArrowUp     = Split the focused pane   vertically and focus the    top"))),
            19 => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Ctrl + Alt + ArrowDown   = Split the focused pane   vertically and focus the bottom"))),
            20 => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Ctrl + Alt + ArrowLeft   = Split the focused pane horizontally and focus the   left"))),
            21 => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Ctrl + Alt + ArrowRight  = Split the focused pane horizontally and focus the  right"))),

            24 => sp.with_bg(bg, |sp| sp.fg(fgb, fill(w,         "  DIRECTORY KEYBINDS"))),
            25 => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    ArrowUp                  = Select the entry above"))),
            26 => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    ArrowDown                = Select the entry below"))),
            27 => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    ArrowLeft  / Backspace   = Enter the parent   directory"))),
            28 => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    ArrowRight / Enter       = Enter the selected directory or file"))),

            31 => sp.with_bg(bg, |sp| sp.fg(fgb, fill(w,         "  FILE KEYBINDS"))),
            32 => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    ArrowUp                  = Move the cursor    up"))),
            33 => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    ArrowDown                = Move the cursor  down"))),
            34 => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    ArrowLeft                = Move the cursor  left"))),
            35 => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    ArrowRight               = Move the cursor right"))),

            37 => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Ctrl + ArrowLeft         = Move the cursor to the previous separator"))),
            38 => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Ctrl + ArrowRight        = Move the cursor to the     next separator"))),

            40 => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Home                     = Move the cursor to the start of the line"))),
            41 => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    End                      = Move the cursor to the   end of the line"))),

            43 => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Ctrl + Home              = Move the cursor to the first        line"))),
            44 => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Ctrl + End               = Move the cursor to the  last        line"))),

            46 => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Backspace                = Erase character to the  left of the cursor"))),
            47 => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Delete                   = Erase character to the right of the cursor"))),

            49 => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Ctrl + Backspace         = Erase to the  left from the cursor until separator"))),
            50 => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Ctrl + Delete            = Erase to the right from the cursor until separator"))),

            52 => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Alt + ArrowUp            = Move the line above"))),
            53 => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Alt + ArrowDown          = Move the line below"))),

            55 => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Enter                    = Insert a newline"))),
            56 => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Tab                      = Insert spaces to reach the closest tabstop to the right"))),

            58 => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Ctrl + S                 = Save file"))),

            _  => sp.bg(bg, fill(w, ""))
        }
    }

    fn event(&mut self, event: Event, _w: u16, _h: u16) -> (PaneCommand<ViewEvent>, ViewEvent) {
        match event {
            Event::Keyboard(KeyboardEvent::CtrlAlt(Key::ArrowUp))
                | Event::Keyboard(KeyboardEvent::CtrlAlt(Key::ArrowDown ))
                | Event::Keyboard(KeyboardEvent::CtrlAlt(Key::ArrowLeft ))
                | Event::Keyboard(KeyboardEvent::CtrlAlt(Key::ArrowRight))
            => {
                (PaneCommand::DoNothing, ViewEvent::Welcomed)
            },
            Event::Keyboard(KeyboardEvent::NoModifiers(Key::ArrowUp)) => {
                if self.scroll != 0 {
                    self.scroll -= 1;
                    (PaneCommand::RerenderMe, ViewEvent::DoNothing)
                } else {
                    (PaneCommand::DoNothing, ViewEvent::DoNothing)
                }
            },
            Event::Keyboard(KeyboardEvent::NoModifiers(Key::ArrowDown)) => {
                self.scroll += 1;
                (PaneCommand::RerenderMe, ViewEvent::DoNothing)
            },
            Event::Mouse(MouseEvent::Scroll(ScrollEvent::NoModifiers(scroll_direction))) => {
                match scroll_direction {
                    ScrollDirection::  Up(_x, _y) => {
                        if self.scroll != 0 {
                            self.scroll -= 1;
                            (PaneCommand::RerenderMe, ViewEvent::DoNothing)
                        } else {
                            (PaneCommand::DoNothing, ViewEvent::DoNothing)
                        }
                    },
                    ScrollDirection::Down(_x, _y) => {
                        self.scroll += 1;
                        (PaneCommand::RerenderMe, ViewEvent::DoNothing)
                    }
                }
            },
            Event::Keyboard(KeyboardEvent::NoModifiers(Key::Home)) => {
                if self.scroll != 0 {
                    self.scroll = 0;
                    (PaneCommand::RerenderMe, ViewEvent::DoNothing)
                } else {
                    (PaneCommand::DoNothing, ViewEvent::DoNothing)
                }
            },
            _ => (PaneCommand::DoNothing, ViewEvent::DoNothing)
        }
    }
}

fn fill(w: u16, text: impl AsRef<str>) -> String {
    let line         = text.as_ref();
    let visible_line = line.utf8_range(0, w as isize);

    format!("{visible_line}{}", " ".repeat((w as isize - visible_line.utf8_len()).max(0) as usize))
}
