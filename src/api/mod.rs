pub mod win32;
pub mod wgl;

fn osstr<'a>(s: &'a str) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    OsStr::new(s).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>()
}