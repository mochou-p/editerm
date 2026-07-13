// mochou-p/editerm/src/view/editing/actions/file.rs

impl super::super::Editing {
    pub fn save(&mut self) {
        if self.file.clean {
            return;
        }

        // NOTE: 7 -> '\a' -> BEL
        // TODO: write!(editor.stdout, "{}", 7 as char).unwrap();

        let writee = if
            self.file.lines.len() == 1
            &&
            self.file.lines[0].is_empty()
        {
            String::new()
        } else {
            self.file.lines.join("\n")
        };

        std::fs::write(&self.path, writee).unwrap();
        self.file.clean = true;
    }
}
