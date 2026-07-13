// mochou-p/editerm/src/view/browsing.rs

use std::collections::HashMap;
use std::ffi::CString;
use std::path::{Path, PathBuf};
use spliterm::{PaneView, PaneCommand};
use spliterm::betterm;
use betterm::color::{ansi, AnsiColor};
use betterm::styled_printer::StyledPrinter;
use betterm::terminal::{Event, KeyboardEvent, Key, MouseEvent, MouseButtonEvent, MouseButton};
use betterm::libc;
use super::Editing;


pub struct Browsing {
    current_dir: PathBuf,
    focused:     usize,
    focuses:     HashMap<PathBuf, usize>,
    parent:      Option<BrowserEntry>,
    dirs:        Vec<BrowserEntry>,
    files:       Vec<BrowserEntry>,
    scroll_y:    usize
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

        Self { current_dir, focused, focuses, parent, dirs, files, scroll_y: 0 }
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

    fn up(&mut self) -> PaneCommand {
        if self.focused != 0 {
            self.focused -= 1;
            PaneCommand::RerenderMe
        } else {
            PaneCommand::DoNothing
        }
    }

    fn down(&mut self) -> PaneCommand {
        if self.focused != self.dirs.len() + self.files.len() - self.parent.is_none() as usize {
            self.focused += 1;
            PaneCommand::RerenderMe
        } else {
            PaneCommand::DoNothing
        }
    }

    fn go_out(&mut self) -> PaneCommand {
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

    fn go_in(&mut self) -> PaneCommand {
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
            prefix:    Option<AnsiColor>,
            suffix:    &str,
            w:         u16
    ) -> StyledPrinter {
        if focused {
            sp = sp.push_bg(ansi::black().bright())
        } else {
            sp = sp.push_bg(ansi::black())
        }

        let mut width = w;

        if width > 4 {
            sp = sp.fg(if entry.r { ansi::green() } else { ansi::red() }, "r");
            sp = sp.fg(if entry.w { ansi::green() } else { ansi::red() }, "w");
            sp = sp.fg(if entry.x { ansi::green() } else { ansi::red() }, "x");
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

        if let Some(color) = prefix {
            sp.fg(color, final_text)
        } else {
            sp = sp.reset_fg();
            sp.text(final_text)
        }
    }

    fn print_dir(&self, sp: StyledPrinter, focused: bool, i: usize, w: u16) -> StyledPrinter {
        self.print_entry(sp, focused, &self.dirs[i], false, Some(ansi::blue()), "/", w)
    }

    fn print_file(&self, sp: StyledPrinter, focused: bool, i: usize, w: u16) -> StyledPrinter {
        self.print_entry(sp, focused, &self.files[i], false, None, "", w)
    }

    fn print_empty(&self, sp: StyledPrinter, w: u16) -> StyledPrinter {
        sp.bg(ansi::red().bright(), " ".repeat(w as usize))
    }
}

impl PaneView for Browsing {
    fn print_line(&self, i: usize, w: u16, _h: u16, sp: StyledPrinter) -> StyledPrinter {
        let mut i = i + self.scroll_y;

        if let Some(parent) = self.parent.as_ref() {
            if i == 0 {
                return self.print_entry(sp, self.focused == 0, parent, true, Some(ansi::blue()), "/", w);
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

    fn event(&mut self, event: Event, _w: u16, _h: u16) -> PaneCommand {
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
            _ => PaneCommand::DoNothing
        }
    }
}
