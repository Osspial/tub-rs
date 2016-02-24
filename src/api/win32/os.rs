use user32;
use std::mem;
use winapi;

pub fn set_cursor_pos(x: i32, y: i32) {
    unsafe {
        user32::SetCursorPos(x, y);
    }
}

pub fn get_cursor_pos() -> (i32, i32) {
    unsafe {
        let mut point = mem::zeroed();
        user32::GetCursorPos(&mut point);

        (point.x as i32, point.y as i32)
    }
}

#[allow(dead_code)]
pub fn get_highlight_color() -> (u8, u8, u8) {
    color_from_dword(unsafe{ user32::GetSysColor(winapi::COLOR_HIGHLIGHT) })
}

#[allow(dead_code)]
pub fn get_inner_highlight_color() -> (u8, u8, u8) {
    color_from_dword(unsafe{ user32::GetSysColor(winapi::COLOR_HIGHLIGHTTEXT) })
}

#[allow(dead_code)]
pub fn get_background_color() -> (u8, u8, u8) {
    color_from_dword(unsafe{ user32::GetSysColor(winapi::COLOR_BACKGROUND) })
}

#[allow(dead_code)]
fn color_from_dword(color: winapi::DWORD) -> (u8, u8, u8) {
    ((color & 0xFF) as u8, (color >> 8 & 0xFF) as u8, (color >> 16) as u8)
}