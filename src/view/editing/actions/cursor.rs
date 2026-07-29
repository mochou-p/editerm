// mochou-p/editerm/src/view/editing/actions/cursor.rs

use spliterm::PaneCommand;
use crate::ViewEvent;
use crate::config::Theme;
use crate::utils::{self, ToWith, Utf8, word};


impl super::super::Editing {
    pub fn line_start(&mut self, w: usize, h: usize) -> PaneCommand<ViewEvent, Theme, (), String> {
        let mut dirty = false;

        for cursor in &mut self.file.cursors {
            if cursor.x != 0 {
                cursor.x = 0;
                dirty    = true
            }

            cursor.last_x = 0;
        }

        match self.snap_to_cursor(w, h) {
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

    pub fn line_end(&mut self, w: usize, h: usize) -> PaneCommand<ViewEvent, Theme, (), String> {
        let mut dirty = false;

        for cursor in &mut self.file.cursors {
            let target = self.file.lines[cursor.y as usize].utf8_len();

            if cursor.x != target {
                cursor.x = target;
                dirty    = true;
            }

            cursor.last_x = isize::MAX;
        }

        match self.snap_to_cursor(w, h) {
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

    pub fn file_start(&mut self, w: usize, h: usize) -> PaneCommand<ViewEvent, Theme, (), String> {
        let mut dirty = false;

        for cursor in &mut self.file.cursors {
            if cursor.y != 0 {
                cursor.y = 0;

                cursor.x
                    .to_max_with(cursor.last_x)
                    .to_min_with(self.file.lines[cursor.y as usize].utf8_len());

                dirty = true;
            }
        }

        match self.snap_to_cursor(w, h) {
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

    pub fn file_end(&mut self, w: usize, h: usize) -> PaneCommand<ViewEvent, Theme, (), String> {
        let     line_count = self.file.lines.len() as isize;
        let mut dirty      = false;

        for cursor in &mut self.file.cursors {
            if cursor.y != line_count - 1 {
                cursor.y = line_count - 1;

                cursor.x
                    .to_max_with(cursor.last_x)
                    .to_min_with(self.file.lines[cursor.y as usize].utf8_len());

                dirty = true
            }
        }

        match self.snap_to_cursor(w, h) {
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

    pub fn up(&mut self, w: usize, h: usize) -> PaneCommand<ViewEvent, Theme, (), String> {
        let mut dirty = false;

        for cursor in &mut self.file.cursors {
            if cursor.y == 0 {
                if cursor.x != 0 {
                    cursor.x = 0;
                    dirty    = true;
                }

                cursor.last_x = 0;
            } else {
                cursor.y -= 1;
                cursor.x
                    .to_max_with(cursor.last_x)
                    .to_min_with(self.file.lines[cursor.y as usize].utf8_len());

                dirty = true;
            }
        }

        match self.snap_to_cursor(w, h) {
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

    pub fn down(&mut self, w: usize, h: usize) -> PaneCommand<ViewEvent, Theme, (), String> {
        let mut dirty = false;

        for cursor in &mut self.file.cursors {
            if cursor.y == (self.file.lines.len() - 1) as isize {
                let target = self.file.lines[cursor.y as usize].utf8_len();

                if cursor.x != target {
                    cursor.x = target;
                    dirty    = true;
                }

                cursor.last_x = cursor.x;
            } else {
                cursor.y += 1;
                cursor.x
                    .to_max_with(cursor.last_x)
                    .to_min_with(self.file.lines[cursor.y as usize].utf8_len());

                dirty = true;
            }
        }

        match self.snap_to_cursor(w, h) {
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

    pub fn left(&mut self, w: usize, h: usize) -> PaneCommand<ViewEvent, Theme, (), String> {
        let mut dirty = false;

        for cursor in &mut self.file.cursors {
            if cursor.x == 0 {
                if cursor.y != 0 {
                    cursor.y -= 1;
                    cursor.x  = self.file.lines[cursor.y as usize].utf8_len();
                    dirty     = true;
                }
            } else {
                cursor.x -= 1;
                dirty     = true;
            }

            cursor.last_x = cursor.x;
        }

        match self.snap_to_cursor(w, h) {
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

    pub fn right(&mut self, w: usize, h: usize) -> PaneCommand<ViewEvent, Theme, (), String> {
        let mut dirty = false;

        for cursor in &mut self.file.cursors {
            if cursor.x == self.file.lines[cursor.y as usize].utf8_len() {
                if cursor.y != (self.file.lines.len() - 1) as isize {
                    cursor.x  = 0;
                    cursor.y += 1;
                    dirty     = true;
                }
            } else {
                cursor.x += 1;
                dirty     = true;
            }

            cursor.last_x = cursor.x;
        }

        match self.snap_to_cursor(w, h) {
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

    pub fn prev_word(&mut self, w: usize, h: usize) -> PaneCommand<ViewEvent, Theme, (), String> {
        let mut dirty = false;

        for cursor in &mut self.file.cursors {
            // TODO: wait shouldnt this be a continue, and take out inner dirt
            if cursor.x == 0 {
                return self.left(w, h);
            }

            let line  = &self.file.lines[cursor.y as usize];
            let start = line.chars().nth((cursor.x - 1) as usize).unwrap();

            let end = if utils::is_alphanumericx(start) {
                word::to_left(line, cursor.x, |ch| !utils::is_alphanumericx(ch))
            } else {
                word::to_left(line, cursor.x, |ch| ch != start)
            };

            cursor.x      = end.map(|i| i+1).unwrap_or(0);
            cursor.last_x = cursor.x;
            dirty         = true;
        }

        match self.snap_to_cursor(w, h) {
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

    pub fn next_word(&mut self, w: usize, h: usize) -> PaneCommand<ViewEvent, Theme, (), String> {
        let mut dirty = false;

        for cursor in &mut self.file.cursors {
            let line = &self.file.lines[cursor.y as usize];

            // TODO: wait shouldnt this be a continue, and take out inner dirt
            if cursor.x == line.utf8_len() {
                return self.right(w, h);
            }

            let start = line.chars().nth(cursor.x as usize).unwrap();

            let end = if utils::is_alphanumericx(start) {
                word::to_right(line, cursor.x, |ch| !utils::is_alphanumericx(ch))
            } else {
                word::to_right(line, cursor.x, |ch| ch != start)
            };

            cursor.x      = end.unwrap_or(self.file.lines[cursor.y as usize].utf8_len());
            cursor.last_x = cursor.x;
            dirty         = true;
        }

        match self.snap_to_cursor(w, h) {
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
}
