// mochou-p/editerm/src/view/browsing.rs

use std::collections::HashMap;
use std::ffi::CString;
use std::path::{Path, PathBuf};
use spliterm::{PaneView, PaneCommand, Event, PaneEvent};
use spliterm::betterm;
use betterm::color::rgb;
use betterm::styled_printer::StyledPrinter;
use betterm::terminal::{KeyboardEvent, Key, MouseEvent, MouseButtonEvent, MouseButton, ScrollEvent, ScrollDirection, HoverEvent};
use betterm::libc;
use super::Editing;
use crate::{ViewEvent, In, Out, ColoredText};
use crate::config::Theme;
use crate::view::Welcome;


#[derive(Clone)]
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

#[derive(Clone)]
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
    pub fn new(path: Option<PathBuf>, i: Option<usize>) -> Self {
        let current_dir = path.unwrap_or_else(|| std::env::current_dir().unwrap());
        let current_dir = current_dir.canonicalize().unwrap();

        let (parent, dirs, files) = Self::load(&current_dir);
        // TODO: make this a Path not a number, since dirs can change between visits
        let focused               = i.unwrap_or(0);
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

    fn snap_to_cursor(&mut self, h: usize) -> PaneCommand<ViewEvent, Theme, In, Out> {
        if self.focused < self.scroll_y {
            self.scroll_y = self.focused;
            PaneCommand::RerenderMe
        } else if self.focused > self.scroll_y + h - 1 {
            self.scroll_y = self.focused - h + 1;
            PaneCommand::RerenderMe
        } else {
            PaneCommand::DoNothing
        }
    }

    fn up(&mut self, h: usize) -> PaneCommand<ViewEvent, Theme, In, Out> {
        if self.focused != 0 {
            self.focused -= 1;
            self.snap_to_cursor(h);
            PaneCommand::RerenderMe
        } else {
            PaneCommand::DoNothing
        }
    }

    fn down(&mut self, h: usize) -> PaneCommand<ViewEvent, Theme, In, Out> {
        if self.focused != self.dirs.len() + self.files.len() - self.parent.is_none() as usize {
            self.focused += 1;
            self.snap_to_cursor(h);
            PaneCommand::RerenderMe
        } else {
            PaneCommand::DoNothing
        }
    }

    fn _go_out(this: &mut Self, parent: BrowserEntry, h: usize) {
        let old_dir = this.current_dir.clone();
        this.focuses.insert(old_dir.clone(), this.focused);

        this.current_dir                     = parent.path;
        (this.parent, this.dirs, this.files) = Self::load(&this.current_dir);

        this.focused = this.focuses.get(&this.current_dir)
            .map_or_else(
                || {
                    this.dirs
                        .iter()
                        .position(|entry| entry.path == old_dir)
                        .unwrap()
                    + this.parent.is_some() as usize
                },
                |i| *i
            );

        this.snap_to_cursor(h);
    }

    fn go_out(&mut self, h: usize, newtab: bool) -> (PaneCommand<ViewEvent, Theme, In, Out>, ViewEvent) {
        self.scroll_y = 0;

        let Some(parent) = self.parent.clone() else { return (PaneCommand::DoNothing, ViewEvent::DoNothing); };

        if newtab {
            let mut newone = self.clone();
            Self::_go_out(&mut newone, parent, h);

            (PaneCommand::DoNothing, ViewEvent::Open(Box::new(newone)))
        } else {
            Self::_go_out(self, parent, h);

            (PaneCommand::RerenderMe, ViewEvent::DoNothing)
        }
    }

    fn _go_in_dir(this: &mut Self, i: usize, h: usize) {
        this.focuses.insert(this.current_dir.clone(), this.focused);

        this.current_dir                     = this.dirs.remove(i).path;
        (this.parent, this.dirs, this.files) = Self::load(&this.current_dir);

        this.focused = this.focuses.get(&this.current_dir).map_or_else(|| 0, |i| *i);

        this.snap_to_cursor(h);
    }

    fn go_in(&mut self, h: usize, newtab: bool) -> (PaneCommand<ViewEvent, Theme, In, Out>, ViewEvent) {
        self.scroll_y = 0;

        let mut i = self.focused;

        if self.parent.is_some() {
            if i == 0 {
                return self.go_out(h, newtab);
            }

            i -= 1;
        }

        if i < self.dirs.len() {
            if newtab {
                let mut newone = self.clone();
                Self::_go_in_dir(&mut newone, i, h);

                (PaneCommand::DoNothing, ViewEvent::Open(Box::new(newone)))
            } else {
                Self::_go_in_dir(self, i, h);

                (PaneCommand::RerenderMe, ViewEvent::DoNothing)
            }
        } else {
            i -= self.dirs.len() + self.parent.is_some() as usize - 1;

            let j       = self.parent.is_some() as usize + self.dirs.len() + i;
            let editing = Editing::new(self.files[i].path.clone(), Some(j));

            if newtab {
                (PaneCommand::DoNothing, ViewEvent::Open(editing))
            } else {
                // TODO: Tabs could do the DrawCursor but idk
                (PaneCommand::ReplaceMe(editing), ViewEvent::DrawCursor(0, 0))
            }
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
            w:         u16,
            theme:     &Theme
    ) -> StyledPrinter {
        if focused && self.pane_focus {
            sp = sp.push_bg(theme.background_selected)
        } else {
            sp = sp.push_bg(theme.background)
        }

        let mut width = w;

        if width > 4 {
            sp = sp.fg(if entry.r { theme.green } else { theme.red }, if entry.r { "r" } else { "-" });
            sp = sp.fg(if entry.w { theme.green } else { theme.red }, if entry.w { "w" } else { "-" });
            sp = sp.fg(if entry.x { theme.green } else { theme.red }, if entry.x { "x" } else { "-" });
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

        sp.fg(if is_dir { theme.blue } else { theme.foreground }, final_text)
    }

    fn print_dir(&self, sp: StyledPrinter, focused: bool, i: usize, w: u16, theme: &Theme) -> StyledPrinter {
        self.print_entry(sp, focused, &self.dirs[i], false, true, "/", w, theme)
    }

    fn print_file(&self, sp: StyledPrinter, focused: bool, i: usize, w: u16, theme: &Theme) -> StyledPrinter {
        self.print_entry(sp, focused, &self.files[i], false, false, "", w, theme)
    }

    fn print_empty(&self, sp: StyledPrinter, w: u16, theme: &Theme) -> StyledPrinter {
        sp.bg(theme.background_disabled, " ".repeat(w as usize))
    }

    fn scroll_dir(&mut self, direction: isize) -> PaneCommand<ViewEvent, Theme, In, Out> {
        match direction {
            -1 => self.scroll_up(),
            1  => self.scroll_down(),
            _  => PaneCommand::DoNothing
        }
    }

    fn scroll_down(&mut self) -> PaneCommand<ViewEvent, Theme, In, Out> {
        if self.scroll_y != self.entry_count() - 1 {
            self.scroll_y += 1;
            PaneCommand::RerenderMe
        } else {
            PaneCommand::DoNothing
        }
    }

    fn scroll_up(&mut self) -> PaneCommand<ViewEvent, Theme, In, Out> {
        if self.scroll_y != 0 {
            self.scroll_y -= 1;
            PaneCommand::RerenderMe
        } else {
            PaneCommand::DoNothing
        }
    }

    fn dir_start(&mut self, h: usize) -> PaneCommand<ViewEvent, Theme, In, Out> {
        let mut dirty = false;

        if self.focused != 0 {
            self.focused = 0;
            dirty        = true;
        }

        match self.snap_to_cursor(h) {
            PaneCommand::DoNothing => {
                if dirty {
                    PaneCommand::RerenderMe
                } else {
                    PaneCommand::DoNothing
                }
            },
            other => other
        }
    }

    pub fn dir_end(&mut self, h: usize) -> PaneCommand<ViewEvent, Theme, In, Out> {
        let     last_line = self.entry_count() - 1;
        let mut dirty     = false;

        if self.focused != last_line {
            self.focused = last_line;
            dirty        = true;
        }

        match self.snap_to_cursor(h) {
            PaneCommand::DoNothing => {
                if dirty {
                    PaneCommand::RerenderMe
                } else {
                    PaneCommand::DoNothing
                }
            },
            other => other
        }
    }

    fn open(&mut self, h: usize, y: u16, newtab: bool) -> (PaneCommand<ViewEvent, Theme, In, Out>, ViewEvent) {
        let y = y as usize + self.scroll_y;

        if y < self.entry_count() {
            self.focused = y;
            self.go_in(h, newtab)
        } else {
            (PaneCommand::DoNothing, ViewEvent::DoNothing)
        }
    }
}

impl PaneView<ViewEvent, Theme, In, Out> for Browsing {
    fn custom(&self, theme: In) -> Out {
        let s    = self.current_dir.display().to_string();
        let text = format!("{}/", &s[s.rfind('/').unwrap()+1..]);

        if let Some(theme) = theme {
            ColoredText { fg: theme.blue,       text }
        } else {
            ColoredText { fg: rgb(255, 0, 255), text }
        }
    }

    fn pane_clone(&self) -> Box<dyn PaneView<ViewEvent, Theme, In, Out>> {
        Box::new(self.clone())
    }

    fn print_line(&mut self, i: usize, w: u16, _h: u16, sp: StyledPrinter, theme: &Theme) -> StyledPrinter {
        let mut i = i + self.scroll_y;

        if let Some(parent) = self.parent.as_ref() {
            if i == 0 {
                return self.print_entry(sp, self.focused == 0, parent, true, true, "/", w, theme);
            } else {
                i -= 1;
            }
        }

        let focused = self.focused == i + self.parent.is_some() as usize;

        if i < self.dirs.len() {
            return self.print_dir(sp, focused, i, w, theme);
        }
        i -= self.dirs.len();

        if i < self.files.len() {
            return self.print_file(sp, focused, i, w, theme);
        }

        self.print_empty(sp, w, theme)
    }

    fn event(&mut self, event: Event, _w: u16, h: u16) -> (PaneCommand<ViewEvent, Theme, In, Out>, ViewEvent) {
        let h = h as usize;

        (
            match event {
                Event::Keyboard(KeyboardEvent::NoModifiers(key)) => match key {
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
                    Key::Escape                  => { return (PaneCommand::DoNothing,  ViewEvent::CloseMe); },
                    Key::ArrowUp                 =>          self.up       (h       ),
                    Key::ArrowDown               =>          self.down     (h       ),
                    Key::ArrowLeft               => { return self.go_out   (h, false); },
                    Key::ArrowRight | Key::Enter => { return self.go_in    (h, false); },
                    Key::Home                    =>          self.dir_start(h       ),
                    Key::End                     =>          self.dir_end  (h       ),
                    _                            => PaneCommand::DoNothing
                },
                Event::Keyboard(KeyboardEvent::Ctrl(key)) => match key {
                    Key::ArrowLeft               => { return self.go_out   (h,  true); },
                    Key::ArrowRight | Key::Enter => { return self.go_in    (h,  true); },
                    _                            => PaneCommand::DoNothing
                },
                Event::Keyboard(KeyboardEvent::    Backspace) => { return self.go_out(h, false); },
                Event::Keyboard(KeyboardEvent::CtrlBackspace) => { return self.go_out(h,  true); },
                Event::Mouse(MouseEvent::Hover(HoverEvent::NoModifiers(_x, y) | HoverEvent::Ctrl(_x, y))) => {
                    let y = y as usize + self.scroll_y;

                    if y < self.entry_count() {
                        if self.focused != y {
                            self.focused = y;
                            PaneCommand::RerenderMe
                        } else {
                            PaneCommand::DoNothing
                        }
                    } else {
                        PaneCommand::DoNothing
                    }
                },
                Event::Mouse(MouseEvent::Press(MouseButtonEvent::NoModifiers(MouseButton::Left(_x, y)))) => {
                    return self.open(h, y, false);
                },
                Event::Mouse(MouseEvent::Press(MouseButtonEvent::Ctrl(MouseButton::Left(_x, y)) | MouseButtonEvent::NoModifiers(MouseButton::Middle(_x, y)))) => {
                    return self.open(h, y,  true);
                },
                Event::Mouse(MouseEvent::Scroll(ScrollEvent::NoModifiers(scroll_direction))) => match scroll_direction {
                    ScrollDirection::Up  (_x, _y) => self.scroll_dir(-1),
                    ScrollDirection::Down(_x, _y) => self.scroll_dir( 1)
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
