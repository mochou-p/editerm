// mochou-p/editerm/src/view/editing/actions/typing.rs

use spliterm::PaneCommand;
use crate::utils::{self, Utf8, Utf8Mut, word};


impl super::super::Editing {
    pub fn newline(&mut self, w: usize, h: usize) -> PaneCommand {
        self.file.clean = false;

        for cursor in &mut self.file.cursors {
            let trail = self.file.lines[cursor.y as usize].utf8_split_off(cursor.x);

            cursor.x       = 0;
            cursor.last_x  = cursor.x;
            cursor.y      += 1;

            self.file.lines.insert(cursor.y as usize, trail);
        }

        self.snap_to_cursor(w, h);

        PaneCommand::RerenderMe
    }

    pub fn tab(&mut self, w: usize, h: usize) -> PaneCommand {
        self.file.clean = false;

        for cursor in &mut self.file.cursors {
            let count = 4 - (cursor.x as usize % 4);

            self.file.lines[cursor.y as usize].utf8_insert_str(cursor.x, &(" ".repeat(count)));

            cursor.x += count as isize;
        }

        self.snap_to_cursor(w, h);

        PaneCommand::RerenderMe
    }

    pub fn character(&mut self, ch: char, w: usize, h: usize) -> PaneCommand {
        self.file.clean = false;

        for cursor in &mut self.file.cursors {
            self.file.lines[cursor.y as usize].utf8_insert(cursor.x, ch);

            cursor.x      += 1;
            cursor.last_x  = cursor.x;
        }

        self.snap_to_cursor(w, h);

        PaneCommand::RerenderMe
    }

    pub fn erase_left(&mut self, w: usize, h: usize) -> PaneCommand {
        let mut dirty = false;

        for cursor in &mut self.file.cursors {
            if cursor.x == 0 {
                if cursor.y != 0 {
                    let line = self.file.lines.remove(cursor.y as usize);

                    cursor.y -= 1;
                    cursor.x  = self.file.lines[cursor.y as usize].utf8_len();

                    self.file.lines[cursor.y as usize].push_str(&line);
                    self.file.clean = false;
                    dirty           = true;
                }
            } else {
                cursor.x -= 1;

                self.file.lines[cursor.y as usize].utf8_remove(cursor.x);
                self.file.clean = false;
                dirty           = true;
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

    pub fn erase_right(&mut self, w: usize, h: usize) -> PaneCommand {
        let mut dirty = false;

        for cursor in &mut self.file.cursors {
            if cursor.x == self.file.lines[cursor.y as usize].utf8_len() {
                if cursor.y != (self.file.lines.len() - 1) as isize {
                    let line = self.file.lines.remove((cursor.y + 1) as usize);

                    self.file.lines[cursor.y as usize].push_str(&line);
                    self.file.clean = false;
                    dirty           = true;
                }
            } else {
                self.file.lines[cursor.y as usize].utf8_remove(cursor.x);
                self.file.clean = false;
                dirty           = true;
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

    pub fn move_line_up(&mut self, w: usize, h: usize) -> PaneCommand {
        let mut dirty = false;

        for cursor in &mut self.file.cursors {
            if cursor.y != 0 {
                self.file.lines.swap(cursor.y as usize, (cursor.y - 1) as usize);
                cursor.y -= 1;

                self.file.clean = false;
                dirty           = true;
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

    pub fn move_line_down(&mut self, w: usize, h: usize) -> PaneCommand {
        let mut dirty = false;

        for cursor in &mut self.file.cursors {
            if cursor.y != (self.file.lines.len() - 1) as isize {
                self.file.lines.swap(cursor.y as usize, (cursor.y + 1) as usize);
                cursor.y += 1;

                self.file.clean = false;
                dirty           = true;
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

    pub fn erase_prev_word(&mut self, w: usize, h: usize) -> PaneCommand {
        let mut dirty = false;

        for cursor in &mut self.file.cursors {
            // TODO: wait shouldnt this be a continue, and take out inner dirt
            if cursor.x == 0 {
                return self.erase_left(w, h);
            }

            let line  = &mut self.file.lines[cursor.y as usize];
            let start = line.chars().nth((cursor.x - 1) as usize).unwrap();

            let end = if utils::is_alphanumericx(start) {
                word::to_left(line, cursor.x, |ch| !utils::is_alphanumericx(ch))
            } else {
                word::to_left(line, cursor.x, |ch| ch != start)
            };

            let old_x     = cursor.x;
            cursor.x      = end.map(|i| i+1).unwrap_or(0);
            cursor.last_x = cursor.x;

            line.utf8_drain(cursor.x, old_x);
            dirty = true;
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

    pub fn erase_next_word(&mut self, w: usize, h: usize) -> PaneCommand {
        let mut dirty = false;

        for cursor in &mut self.file.cursors {
            let line = &self.file.lines[cursor.y as usize];

            // TODO: wait shouldnt this be a continue, and take out inner dirt
            if cursor.x == line.utf8_len() {
                return self.erase_right(w, h);
            }

            let line  = &mut self.file.lines[cursor.y as usize];
            let start = line.chars().nth(cursor.x as usize).unwrap();

            let end = if utils::is_alphanumericx(start) {
                word::to_right(line, cursor.x, |ch| !utils::is_alphanumericx(ch))
            } else {
                word::to_right(line, cursor.x, |ch| ch != start)
            };

            cursor.last_x = cursor.x;

            line.utf8_drain(cursor.x, end.unwrap_or(line.utf8_len()));
            dirty = true;
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
