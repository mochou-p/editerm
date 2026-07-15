// mochou-p/editerm/src/view/editing/actions/view.rs

use spliterm::PaneCommand;
use crate::ViewEvent;


impl super::super::Editing {
    pub fn scroll_dir(&mut self, direction: isize) -> PaneCommand<ViewEvent> {
        match direction {
            -1 => self.scroll_up(),
            1  => self.scroll_down(),
            _  => PaneCommand::DoNothing
        }
    }

    fn scroll_down(&mut self) -> PaneCommand<ViewEvent> {
        if self.scroll.y != (self.file.lines.len() - 1) as isize {
            self.scroll.y += 1;
            PaneCommand::RerenderMe
        } else {
            PaneCommand::DoNothing
        }
    }

    fn scroll_up(&mut self) -> PaneCommand<ViewEvent> {
        if self.scroll.y != 0 {
            self.scroll.y -= 1;
            PaneCommand::RerenderMe
        } else {
            PaneCommand::DoNothing
        }
    }
}
