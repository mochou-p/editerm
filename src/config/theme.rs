// mochou-p/editerm/src/config/theme.rs

use spliterm::betterm;
use betterm::color::{rgb, RgbColor};


#[allow(dead_code)]
#[derive(Clone)]
pub struct Theme {
    pub pane_separator:      RgbColor,
    pub background_disabled: RgbColor,
    pub background:          RgbColor,
    pub background_selected: RgbColor,
    pub foreground_disabled: RgbColor,
    pub foreground:          RgbColor,
    pub red:                 RgbColor,
    pub yellow:              RgbColor,
    pub green:               RgbColor,
    pub cyan:                RgbColor,
    pub blue:                RgbColor,
    pub magenta:             RgbColor
}

impl Theme {
    pub fn all() -> Vec<Self> {
        vec![
            Theme::catppuccin_mocha(),
            Theme::catppuccin_latte(),
            Theme::dark(),
            Theme::light()
        ]
    }

    // https://catppuccin.com/palette/
    pub fn catppuccin_mocha() -> Self {
        Self {
            pane_separator:      rgb(0x11, 0x11, 0x1b),
            background_disabled: rgb(0x18, 0x18, 0x25),
            background:          rgb(0x1e, 0x1e, 0x2e),
            background_selected: rgb(0x31, 0x32, 0x44),
            foreground_disabled: rgb(0x93, 0x99, 0xb2),
            foreground:          rgb(0xcd, 0xd6, 0xf4),
            red:                 rgb(0xf3, 0x8b, 0xa8),
            yellow:              rgb(0xf9, 0xe2, 0xaf),
            green:               rgb(0xa6, 0xe3, 0xa1),
            cyan:                rgb(0x89, 0xdc, 0xeb),
            blue:                rgb(0x89, 0xb4, 0xfa),
            magenta:             rgb(0xcb, 0xa6, 0xf7)
        }
    }

    // https://catppuccin.com/palette/
    pub fn catppuccin_latte() -> Self {
        Self {
            pane_separator:      rgb(0x9c, 0xa0, 0xb0),
            background_disabled: rgb(0xbc, 0xc0, 0xcc),
            background:          rgb(0xdc, 0xe0, 0xe8),
            background_selected: rgb(0xef, 0xf1, 0xf5),
            foreground_disabled: rgb(0x7c, 0x7f, 0x93),
            foreground:          rgb(0x4c, 0x4f, 0x69),
            red:                 rgb(0xd2, 0x0f, 0x39),
            yellow:              rgb(0xdf, 0x8e, 0x1d),
            green:               rgb(0x40, 0xa0, 0x2b),
            cyan:                rgb(0x04, 0xa5, 0xe5),
            blue:                rgb(0x1e, 0x66, 0xf5),
            magenta:             rgb(0x88, 0x39, 0xef)
        }
    }

    pub fn dark() -> Self {
        Self {
            pane_separator:      rgb(0x00, 0x00, 0x00),
            background_disabled: rgb(0x12, 0x12, 0x12),
            background:          rgb(0x1a, 0x1a, 0x1a),
            background_selected: rgb(0x24, 0x24, 0x24),
            foreground_disabled: rgb(0xd9, 0xd9, 0xd9),
            foreground:          rgb(0xff, 0xff, 0xff),
            red:                 rgb(0xff, 0x4f, 0x4d),
            yellow:              rgb(0xff, 0xff, 0x4d),
            green:               rgb(0x4d, 0xff, 0x4d),
            cyan:                rgb(0x4d, 0xff, 0xff),
            blue:                rgb(0x4d, 0x4d, 0xff),
            magenta:             rgb(0xff, 0x4d, 0xff)
        }
    }

    pub fn light() -> Self {
        Self {
            pane_separator:      rgb(0xc2, 0xc2, 0xc2),
            background_disabled: rgb(0xd6, 0xd6, 0xd6),
            background:          rgb(0xeb, 0xeb, 0xeb),
            background_selected: rgb(0xff, 0xff, 0xff),
            foreground_disabled: rgb(0x26, 0x26, 0x26),
            foreground:          rgb(0x00, 0x00, 0x00),
            red:                 rgb(0xcc, 0x00, 0x00),
            yellow:              rgb(0xcc, 0xcc, 0x00),
            green:               rgb(0x00, 0xcc, 0x00),
            cyan:                rgb(0x00, 0xcc, 0xcc),
            blue:                rgb(0x00, 0x00, 0xcc),
            magenta:             rgb(0xcc, 0x00, 0xcc)
        }
    }
}
