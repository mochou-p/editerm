// mochou-p/editerm/src/view/welcome.rs

use spliterm::{PaneView, PaneCommand, Event};
use spliterm::betterm;
use betterm::terminal::{KeyboardEvent, Key, MouseEvent, ScrollEvent, ScrollDirection};
use betterm::styled_printer::StyledPrinter;
use crate::ViewEvent;
use crate::config::Theme;
use crate::utils::Utf8;
use crate::view::Browsing;


pub struct Welcome {
    scroll:   usize,
    ancestor: Option<Box<dyn PaneView<ViewEvent, Theme>>>
}

impl Welcome {
    pub fn new(ancestor: Option<Box<dyn PaneView<ViewEvent, Theme>>>) -> Self {
        Self { scroll: 0, ancestor }
    }
}

impl PaneView<ViewEvent, Theme> for Welcome {
    fn pane_clone(&self) -> Box<dyn PaneView<ViewEvent, Theme>> {
        let ancestor = self.ancestor.as_ref().map(|ancestor| ancestor.pane_clone());

        Box::new(Welcome { scroll: self.scroll, ancestor })
    }

    fn print_line(&mut self, i: usize, w: u16, _h: u16, sp: StyledPrinter, theme: &Theme) -> StyledPrinter {
        let i = i + self.scroll;

        let bg  = theme.background;
        let fg  = theme.foreground_disabled;
        let fgb = theme.foreground;
        let cfg = theme.cyan;
        let yfg = theme.yellow;
        let bin = env!("CARGO_BIN_NAME");

        let state_txt = if self.ancestor.is_some() {
            "  Escape to go back"
        } else {
            "  Escape to exit, Enter to open current directory"
        };

        match i {
            0    => sp.     bg(bg,                 fill(w,         ""                                                                                         )),
            1    => sp.with_bg(bg, |sp| sp.fg(cfg, fill(w, format!("  Welcome to {bin}! \\(^ヮ^)/"                                                          )))),
            2    => sp.with_bg(bg, |sp| sp.fg(yfg, fill(w,            state_txt                                                                              ))),
            3    => sp.     bg(bg,                 fill(w,         ""                                                                                         )),
            4    => sp.     bg(bg,                 fill(w,         ""                                                                                         )),
            5    => sp.with_bg(bg, |sp| sp.fg(fgb, fill(w,         "  USAGE"                                                                                 ))),
            6    => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w, format!("    {bin}                  = Open this help screen"                                     )))),
            7    => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w, format!("    {bin} <DIRECTORY_PATH> = Open a directory to browse"                                )))),
            8    => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w, format!("    {bin}      <FILE_PATH> = Open a file      to edit"                                  )))),
            9    => sp.     bg(bg,                 fill(w,         ""                                                                                         )),
            10   => sp.     bg(bg,                 fill(w,         ""                                                                                         )),
            11   => sp.with_bg(bg, |sp| sp.fg(fgb, fill(w,         "  GLOBAL KEYBINDS"                                                                       ))),
            12   => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    F1                       = Open the help screen"                                     ))),
            13   => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Ctrl + T                 = Cycle color themes"                                       ))),
            14   => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w, format!("    Ctrl + Escape            = Exit {bin}"                                              )))),
            15   => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Alt  + ArrowUp           = Focus the pane        above"                              ))),
            16   => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Alt  + ArrowDown         = Focus the pane        below"                              ))),
            17   => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Alt  + ArrowLeft         = Focus the pane to the  left"                              ))),
            18   => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Alt  + ArrowRight        = Focus the pane to the right"                              ))),
            19   => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Ctrl + Alt + ArrowUp     = Split the focused pane   vertically and focus the    top" ))),
            20   => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Ctrl + Alt + ArrowDown   = Split the focused pane   vertically and focus the bottom" ))),
            21   => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Ctrl + Alt + ArrowLeft   = Split the focused pane horizontally and focus the   left" ))),
            22   => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Ctrl + Alt + ArrowRight  = Split the focused pane horizontally and focus the  right" ))),
            23   => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Ctrl + Alt + Backspace   = Close the focused pane"                                   ))),
            24   => sp.     bg(bg,                 fill(w,         ""                                                                                         )),
            25   => sp.     bg(bg,                 fill(w,         ""                                                                                         )),
            26   => sp.with_bg(bg, |sp| sp.fg(fgb, fill(w,         "  DIRECTORY KEYBINDS"                                                                    ))),
            27   => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    ArrowUp                  = Select the entry above"                                   ))),
            28   => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    ArrowUp                  = Select the entry above"                                   ))),
            29   => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Home                     = Select the first entry"                                   ))),
            30   => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    End                      = Select the last  entry"                                   ))),
            31   => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    ArrowLeft  / Backspace   = Enter the parent   directory"                             ))),
            32   => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    ArrowRight / Enter       = Enter the selected directory or file"                     ))),
            33   => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Escape                   = Close this view"                                          ))),
            34   => sp.     bg(bg,                 fill(w,         ""                                                                                         )),
            35   => sp.     bg(bg,                 fill(w,         ""                                                                                         )),
            36   => sp.with_bg(bg, |sp| sp.fg(fgb, fill(w,         "  FILE KEYBINDS"                                                                         ))),
            37   => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    ArrowUp                  = Move the cursor    up"                                    ))),
            38   => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    ArrowDown                = Move the cursor  down"                                    ))),
            39   => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    ArrowLeft                = Move the cursor  left"                                    ))),
            40   => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    ArrowRight               = Move the cursor right"                                    ))),
            41   => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Ctrl + ArrowLeft         = Move the cursor to the previous separator"                ))),
            42   => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Ctrl + ArrowRight        = Move the cursor to the     next separator"                ))),
            43   => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Home                     = Move the cursor to the start of the line"                 ))),
            44   => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    End                      = Move the cursor to the   end of the line"                 ))),
            45   => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Ctrl + Home              = Move the cursor to the first        line"                 ))),
            46   => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Ctrl + End               = Move the cursor to the  last        line"                 ))),
            47   => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Backspace                = Erase character to the  left of the cursor"               ))),
            48   => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Delete                   = Erase character to the right of the cursor"               ))),
            49   => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Ctrl + Backspace         = Erase to the  left from the cursor until separator"       ))),
            50   => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Ctrl + Delete            = Erase to the right from the cursor until separator"       ))),
            51   => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Ctrl + ArrowUp           = Scroll 5    lines    up"                                  ))),
            52   => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Ctrl + ArrowDown         = Scroll 5    lines  down"                                  ))),
            53   => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    PageUp                   = Scroll full height   up"                                  ))),
            54   => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    PageDown                 = Scroll full height down"                                  ))),
            55   => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Enter                    = Insert a newline"                                         ))),
            56   => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Tab                      = Insert spaces to reach the closest tabstop to the right"  ))),
            57   => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Ctrl + S                 = Save file"                                                ))),
            58   => sp.with_bg(bg, |sp| sp.fg(fg,  fill(w,         "    Escape                   = Browse the parent directory"                              ))),
            59.. => sp.     bg(bg,                 fill(w,         ""                                                                                         ))
        }
    }

    fn event(&mut self, event: Event, _w: u16, _h: u16) -> (PaneCommand<ViewEvent, Theme>, ViewEvent) {
        match event {
            Event::Keyboard(KeyboardEvent::NoModifiers(Key::Enter)) => {
                if self.ancestor.is_none() {
                    (PaneCommand::ReplaceMe(Box::new(Browsing::new(None, false, None))), ViewEvent::DoNothing)
                } else {
                    (PaneCommand::DoNothing, ViewEvent::DoNothing)
                }
            },
            Event::Keyboard(KeyboardEvent::NoModifiers(Key::Escape)) => {
                if let Some(ancestor) = self.ancestor.take() {
                    (PaneCommand::ReplaceMe(ancestor), ViewEvent::DoNothing)
                } else {
                    (PaneCommand::DoNothing, ViewEvent::CloseMe)
                }
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
                    ScrollDirection::Up(_x, _y) => {
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
