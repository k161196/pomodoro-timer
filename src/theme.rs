use gpui::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThemeMode {
    Light,
    Dark,
}

impl ThemeMode {
    pub fn from_appearance(appearance: WindowAppearance) -> Self {
        match appearance {
            WindowAppearance::Dark | WindowAppearance::VibrantDark => ThemeMode::Dark,
            WindowAppearance::Light | WindowAppearance::VibrantLight => ThemeMode::Light,
        }
    }
}

pub struct Theme {
    pub background: Hsla,
    pub foreground: Hsla,
    pub border: Hsla,
    pub muted_background: Hsla,
    pub muted_foreground: Hsla,
    pub secondary: Hsla,
    pub secondary_foreground: Hsla,
}

impl Theme {
    pub fn light() -> Self {
        Self {
            background: rgb(0xffffff).into(),
            foreground: rgb(0x111827).into(),
            border: rgb(0xe5e7eb).into(),
            muted_background: rgb(0xf3f4f6).into(),
            muted_foreground: rgb(0x6b7280).into(),
            secondary: rgb(0xe5e7eb).into(),
            secondary_foreground: rgb(0x374151).into(),
        }
    }

    pub fn dark() -> Self {
        Self {
            background: rgb(0x1f2937).into(),
            foreground: rgb(0xf9fafb).into(),
            border: rgb(0x374151).into(),
            muted_background: rgb(0x374151).into(),
            muted_foreground: rgb(0x9ca3af).into(),
            secondary: rgb(0x4b5563).into(),
            secondary_foreground: rgb(0xe5e7eb).into(),
        }
    }

    pub fn from_mode(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Light => Self::light(),
            ThemeMode::Dark => Self::dark(),
        }
    }
}
