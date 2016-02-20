use user32;
use std::mem;

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