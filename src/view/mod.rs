// mochou-p/editerm/src/view/mod.rs

mod browsing;
mod editing;
mod save_dialog;
mod tabs;
mod welcome;

pub use {browsing::Browsing, editing::Editing, save_dialog::SaveDialog, tabs::Tabs, welcome::Welcome};

