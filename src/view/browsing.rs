// mochou-p/editerm/src/view/browsing.rs

use std::collections::HashMap;
use std::ffi::CString;
use std::path::{Path, PathBuf};
use spliterm::{PaneView, PaneCommand, Event, PaneEvent};
use spliterm::betterm;
use betterm::color::rgb;
use betterm::styled_printer::StyledPrinter;
use betterm::terminal::{KeyboardEvent, Key, MouseEvent, MouseButtonEvent, MouseButton};
use betterm::libc;
use super::Editing;
use crate::ViewEvent;


pub struct Browsing {
    current_dir: PathBuf,
    focused:     usize,
    focuses:     HashMap<PathBuf, usize>,
    parent:      Option<BrowserEntry>,
    dirs:        Vec<BrowserEntry>,
    files:       Vec<BrowserEntry>,
    scroll_y:    usize,
    pane_focus:  bool
}

struct BrowserEntry {
    r:    bool,
    w:    bool,
    x:    bool,
    path: PathBuf
}

impl From<PathBuf> for BrowserEntry {
    fn from(value: PathBuf) -> Self {
        use std::os::unix::ffi::OsStrExt as _;

        let cstring = CString::new(value.as_os_str().as_bytes()).unwrap();
        let ptr     = cstring.as_ptr();

        let r = unsafe { libc::access(ptr, libc::R_OK) == 0 };
        let w = unsafe { libc::access(ptr, libc::W_OK) == 0 };
        let x = unsafe { libc::access(ptr, libc::X_OK) == 0 };

        Self { r, w, x, path: value }
    }
}

impl Browsing {
    pub fn new() -> Self {
        let current_dir           = std::env::current_dir().unwrap();
        let (parent, dirs, files) = Self::load(&current_dir);
        let focused               = 0;
        let focuses               = HashMap::from([(current_dir.clone(), focused)]);

        Self { current_dir, focused, focuses, parent, dirs, files, scroll_y: 0, pane_focus: true }
    }

    fn load(path: &Path) -> (Option<BrowserEntry>, Vec<BrowserEntry>, Vec<BrowserEntry>) {
        let parent = path.parent().map(|parent| BrowserEntry::from(parent.to_path_buf()));

        let mut dirs  = Vec::new();
        let mut files = Vec::new();

        for entry in path.read_dir().unwrap() {
            let entry = entry.unwrap();

            if entry.metadata().unwrap().is_dir() {
                dirs.push(BrowserEntry::from(entry.path()));
            } else if entry.metadata().unwrap().is_file() {
                files.push(BrowserEntry::from(entry.path()));
            }
        }

        dirs .sort_by_key(|entry| entry.path.clone());
        files.sort_by_key(|entry| entry.path.clone());

        (parent, dirs, files)
    }

    fn entry_count(&self) -> usize {
        self.parent.is_some() as usize + self.dirs.len() + self.files.len()
    }

    fn up(&mut self) -> PaneCommand<ViewEvent> {
        if self.focused != 0 {
            self.focused -= 1;
            PaneCommand::RerenderMe
        } else {
            PaneCommand::DoNothing
        }
    }

    fn down(&mut self) -> PaneCommand<ViewEvent> {
        if self.focused != self.dirs.len() + self.files.len() - self.parent.is_none() as usize {
            self.focused += 1;
            PaneCommand::RerenderMe
        } else {
            PaneCommand::DoNothing
        }
    }

    fn go_out(&mut self) -> PaneCommand<ViewEvent> {
        let Some(parent) = self.parent.take() else { return PaneCommand::DoNothing; };

        let old_dir = self.current_dir.clone();
        self.focuses.insert(old_dir.clone(), self.focused);

        self.current_dir                     = parent.path;
        (self.parent, self.dirs, self.files) = Self::load(&self.current_dir);

        self.focused = self.focuses.get(&self.current_dir)
            .map_or_else(
                | | {
                    self.dirs
                        .iter()
                        .position(|entry| entry.path == old_dir)
                        .unwrap()
                    + self.parent.is_some() as usize
                },
                |i| *i
            );

        PaneCommand::RerenderMe
    }

    fn go_in(&mut self) -> PaneCommand<ViewEvent> {
        let mut i = self.focused;

        if self.parent.is_some() {
            if i == 0 {
                return self.go_out();
            }

            i -= 1;
        }

        if i < self.dirs.len() {
            self.focuses.insert(self.current_dir.clone(), self.focused);

            self.current_dir                     = self.dirs.remove(i).path;
            (self.parent, self.dirs, self.files) = Self::load(&self.current_dir);

            self.focused = self.focuses.get(&self.current_dir).map_or_else(|| 0, |i| *i);

            PaneCommand::RerenderMe
        } else {
            i -= self.dirs.len() + self.parent.is_some() as usize - 1;

            let editing = Editing::new(self.files[i].path.clone());
            PaneCommand::ReplaceMe(Box::new(editing))
        }
    }

    fn print_entry(
        &self,
        mut sp:        StyledPrinter,
            focused:   bool,
            entry:     &BrowserEntry,
            is_parent: bool,
            is_dir:    bool,
            suffix:    &str,
            w:         u16
    ) -> StyledPrinter {
        if focused {
            sp = sp.push_bg(if self.pane_focus { rgb(0x40, 0x40, 0x40) } else { rgb(0x24, 0x24, 0x24) })
        } else {
            sp = sp.push_bg(if self.pane_focus { rgb(0x2B, 0x2B, 0x2B) } else { rgb(0x17, 0x17, 0x17) })
        }

        let mut width = w;

        if width > 4 {
            let green = if self.pane_focus { rgb(0x9E, 0xF5, 0x9E) } else { rgb(0x61, 0x99, 0x61) };
            let red   = if self.pane_focus { rgb(0xFF, 0x94, 0x94) } else { rgb(0xA0, 0x5B, 0x5B) };

            sp = sp.fg(if entry.r { green } else { red }, if entry.r { "r" } else { "-" });
            sp = sp.fg(if entry.w { green } else { red }, if entry.w { "w" } else { "-" });
            sp = sp.fg(if entry.x { green } else { red }, if entry.x { "x" } else { "-" });
            sp = sp.text(" ");

            width -= 4;
        }

        let path = if is_parent {
            String::from("..")
        } else {
            entry.path.file_name().unwrap().to_string_lossy().to_string()
        };

        let         text = format!("{path}{suffix}");
        let visible_text = &text[..(width as usize).min(text.len())];
        let   final_text = format!("{visible_text}{}", " ".repeat(w as usize - visible_text.len() - 4));

        if is_dir {
            let idk = if focused { rgb(0x94, 0xD1, 0xFF) } else { rgb(0x76, 0xA4, 0xC6) };
            let lol = if focused { rgb(0x5B, 0x82, 0xA0) } else { rgb(0x47, 0x65, 0x7B) };

            sp.fg(if self.pane_focus { idk } else { lol }, final_text)
        } else {
            let lmao = if focused { rgb(0xC9, 0xC9, 0xC9) } else { rgb(0x80, 0x80, 0x80) };
            let rofl = if focused { rgb(0x7D, 0x7D, 0x7D) } else { rgb(0x4E, 0x4E, 0x4E) };

            sp.fg(if self.pane_focus { lmao } else { rofl }, final_text)
        }
    }

    fn print_dir(&self, sp: StyledPrinter, focused: bool, i: usize, w: u16) -> StyledPrinter {
        self.print_entry(sp, focused, &self.dirs[i], false, true, "/", w)
    }

    fn print_file(&self, sp: StyledPrinter, focused: bool, i: usize, w: u16) -> StyledPrinter {
        self.print_entry(sp, focused, &self.files[i], false, false, "", w)
    }

    fn print_empty(&self, sp: StyledPrinter, w: u16) -> StyledPrinter {
        sp.bg(
            if self.pane_focus {
                rgb(0x1C, 0x1C, 0x1C)
            } else {
                rgb(0x0D, 0x0D, 0x0D)
            },
            " ".repeat(w as usize)
        )
    }
}

impl PaneView<ViewEvent> for Browsing {
    fn print_line(&self, i: usize, w: u16, _h: u16, sp: StyledPrinter) -> StyledPrinter {
        let mut i = i + self.scroll_y;

        if let Some(parent) = self.parent.as_ref() {
            if i == 0 {
                return self.print_entry(sp, self.focused == 0, parent, true, true, "/", w);
            } else {
                i -= 1;
            }
        }

        let focused = self.focused == i + self.parent.is_some() as usize;

        if i < self.dirs.len() {
            return self.print_dir(sp, focused, i, w);
        }
        i -= self.dirs.len();

        if i < self.files.len() {
            return self.print_file(sp, focused, i, w);
        }

        self.print_empty(sp, w)
    }

    fn event(&mut self, event: Event, _w: u16, _h: u16) -> (PaneCommand<ViewEvent>, ViewEvent) {
        (
            match event {
                Event::Keyboard(KeyboardEvent::NoModifiers(key)) => match key {
                    Key::ArrowUp                 => self.up(),
                    Key::ArrowDown               => self.down(),
                    Key::ArrowLeft               => self.go_out(),
                    Key::ArrowRight | Key::Enter => self.go_in(),
                    _                            => PaneCommand::DoNothing
                },
                Event::Keyboard(KeyboardEvent::Backspace) => {
                    self.go_out()
                },
                Event::Mouse(MouseEvent::Press(MouseButtonEvent::NoModifiers(MouseButton::Left(_x, y)))) => {
                    let y = y as usize;

                    if y < self.entry_count() {
                        self.focused = y;
                        self.go_in()
                    } else {
                        PaneCommand::DoNothing
                    }
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
            ViewEvent::DoNothing
        )
    }
}
