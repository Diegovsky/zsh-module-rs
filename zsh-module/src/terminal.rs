//! Stuff that is related to the actual terminal

// /// TODO: Find out if these actually update
// pub static LINES: u16 = zsh_sys::LINES;
// pub static COLUMNS: u16 = zsh_sys::COLUMNS;

// /// The window size, as reported by zsh
// pub struct WindowSize {
//     /// `$LINES`
//     pub lines: u16,
//     /// `$COLUMNS`
//     pub columns: u16,
//     /// TODO: Figure out what this is
//     pub x_pixels: u16,
//     /// TODO: Figure out what this is
//     pub y_pixels: u16,
// }
// impl WindowSize {
//     pub fn get() -> Self {
//         let win_size = zsys::LINES;
//     }
// }

pub type Color = zsh_sys::color_rgb;
