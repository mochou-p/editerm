// mochou-p/editerm/src/view/mod.rs

mod browsing;
mod editing;
mod multiple_opens;
mod save_dialog;
mod tabs;
mod welcome;

pub use {
    browsing::Browsing,
    editing::Editing,
    multiple_opens::MultipleOpens,
    save_dialog::SaveDialog,
    tabs::Tabs,
    welcome::Welcome
};
