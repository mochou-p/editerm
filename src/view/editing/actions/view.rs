// mochou-p/editerm/src/view/editing/actions/view.rs

use spliterm::PaneCommand;
use crate::{ViewEvent, In, Out};
use crate::config::Theme;


impl super::super::Editing {
    pub fn scroll_dir(&mut self, direction: isize) -> PaneCommand<ViewEvent, Theme, In, Out> {
        match direction {
            ..0 => self.scroll_up  (-direction),
            1.. => self.scroll_down( direction),
            0   => PaneCommand::DoNothing
        }
    }

    fn scroll_down(&mut self, amount: isize) -> PaneCommand<ViewEvent, Theme, In, Out> {
        let last_line = (self.file.lines.len() - 1) as isize;

        if self.scroll.y != last_line {
            self.scroll.y = (self.scroll.y + amount).min(last_line);
            PaneCommand::RerenderMe
        } else {
            PaneCommand::DoNothing
        }
    }

    fn scroll_up(&mut self, amount: isize) -> PaneCommand<ViewEvent, Theme, In, Out> {
        if self.scroll.y != 0 {
            self.scroll.y = (self.scroll.y - amount).max(0);
            PaneCommand::RerenderMe
        } else {
            PaneCommand::DoNothing
        }
    }
}
