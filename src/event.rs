use num::FromPrimitive;

pub type Xi32 = i32;
pub type Yi32 = i32;
pub type Xu32 = u32;
pub type Yu32 = u32;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    KeyInput(PressState, VKeyCode),
    MButtonInput(ClickType, MButton),
    /// Triggered when the mouse moves
    MouseMoved(Xi32, Yi32),
    /// Triggered when the mouse hovers over one point for a system-specified length
    /// of time
    MouseHover(Xi32, Yi32),
    /// Triggered when the mouse leaves the client area
    MouseLeave,
    /// Triggered when the mouse enters the client area
    MouseEnter,
    /// Triggered when the window is resized - note that this includes when the window
    /// is first created.
    Resized(ResizeType, Xu32, Xu32),
    Moved(Xi32, Yi32),
    Closed
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PressState {
    Pressed,
    /// On windows, additional virtual keypresses are generated after a key has
    /// been held down for long enough. When that happens, this state is triggered.
    Held,
    Released
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ClickType {
    Single,
    Double,
    Released
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MButton {
    Left = 0,
    Right = 1,
    Middle = 2,
    Button4 = 3,
    Button5 = 4
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ResizeType {
    /// Sent when a window is maximized
    Maximized,
    /// Sent when a window is minimized
    Minimized,
    /// Sent when the size of a window is changed by dragging at the edge of the window
    Changed
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VKeyCode {
    /// Backspace
    Back    = 0x08,
    Tab     = 0x09,
    Clear   = 0x0C,
    Enter   = 0x0D,
    Pause   = 0x13,
    Escape  = 0x1B,
    Space   = 0x20,
    PgUp    = 0x21,
    PgDn    = 0x22,
    End     = 0x23,
    Home    = 0x24,
    Select  = 0x29,
    Print   = 0x2A,
    Execute = 0x2B,
    PrntScr = 0x2C,
    Insert  = 0x2D,
    Delete  = 0x2E,
    Help    = 0x2F,

    Key0    = 0x30,
    Key1    = 0x31,
    Key2    = 0x32,
    Key3    = 0x33,
    Key4    = 0x34,
    Key5    = 0x35,
    Key6    = 0x36,
    Key7    = 0x37,
    Key8    = 0x38,
    Key9    = 0x39,

    A   = 0x41,
    B   = 0x42,
    C   = 0x43,
    D   = 0x44,
    E   = 0x45,
    F   = 0x46,
    G   = 0x47,
    H   = 0x48,
    I   = 0x49,
    J   = 0x4a,
    K   = 0x4b,
    L   = 0x4c,
    M   = 0x4d,
    N   = 0x4e,
    O   = 0x4f,
    P   = 0x50,
    Q   = 0x51,
    R   = 0x52,
    S   = 0x53,
    T   = 0x54,
    U   = 0x55,
    V   = 0x56,
    W   = 0x57,
    X   = 0x58,
    Y   = 0x59,
    Z   = 0x5a,

    /// ';:' on US standard keyboards, though it may not be this on other layouts
    Semi    = 0xBA,
    Plus    = 0xBB,
    Comma   = 0xBC,
    Minus   = 0xBD,
    /// The period key
    Dot     = 0xBE,
    /// '/?' on US standard keyboards, though it may not be this on other layouts
    Slash   = 0xBF,
    /// '`~' on US standard keyboards, though it may not be this on other layouts
    Tilde   = 0xC0,

    /// '[{' on US standard keyboards, though it may not be this on other layouts
    LBrac   = 0xDB,
    /// ']}' on US standard keyboards, though it may not be this on other layouts
    RBrac   = 0xDD,
    /// '\|' on US standard keyboards, though it may not be this on other layouts
    Pipe    = 0xDC,
    /// `"'` on US standard keyboards, though it may not be this on other layouts
    Quote   = 0xDE,

    Sleep   = 0x5F,
    Num0    = 0x60,
    Num1    = 0x61,
    Num2    = 0x62,
    Num3    = 0x63,
    Num4    = 0x64,
    Num5    = 0x65,
    Num6    = 0x66,
    Num7    = 0x67,
    Num8    = 0x68,
    Num9    = 0x69,
    NumStar = 0x6A,
    NumPlus = 0x6B,
    NumSub  = 0x6D,
    NumDot  = 0x6E,
    NumSlash= 0x6F,

    F1      = 0x70,
    F2      = 0x71,
    F3      = 0x72,
    F4      = 0x73,
    F5      = 0x74,
    F6      = 0x75,
    F7      = 0x76,
    F8      = 0x77,
    F9      = 0x78,
    F10     = 0x79,
    F11     = 0x7A,
    F12     = 0x7B,
    F13     = 0x7C,
    F14     = 0x7D,
    F15     = 0x7E,
    F16     = 0x7F,
    F17     = 0x80,
    F18     = 0x81,
    F19     = 0x82,
    F20     = 0x83,
    F21     = 0x84,
    F22     = 0x85,
    F23     = 0x86,
    F24     = 0x87,

    Num     = 0x90,
    Caps    = 0x14,
    Scroll  = 0x91,

    LShift  = 0xA0,
    RShift  = 0xA1,
    LCtrl   = 0xA2,
    RCtrl   = 0xA3,
    LAlt    = 0xA4,
    RAlt    = 0xA5,

    /// Browser back key
    BBack   = 0xA6,
    /// Browser forward key
    BFwd    = 0xA7,
    /// Browser refresh key
    BRef    = 0xA8,
    /// Browser stop key
    BStop   = 0xA9,
    /// Browser search key
    BSearch = 0xAA,
    /// Browser favorites key
    BFav    = 0xAB,
    /// Browser start/home key
    BHome   = 0xAC,

    /// Next track key
    MNTrack = 0xB0,
    /// Previous track key
    MPTrack = 0xB1, // B)
    /// Stop media key
    MStop   = 0xB2,
    /// Play/pause media key
    MPause  = 0xB3,

    

    /// Left arrow key
    LArrow  = 0x25,
    /// Up arrow key
    UArrow  = 0x26,
    /// Right arrow key
    RArrow  = 0x27,
    /// Down arrow key
    DArrow  = 0x28,


    // IME keys
    Kana    = 0x15,
    Junja   = 0x17,
    Final   = 0x18,
    Kanji   = 0x19,
    Convert = 0x1C,
    Nonconvert = 0x1D,
    Accept  = 0x1E,
    ModeChange = 0x1F,
    Process = 0xE5,



    // Come back to these
    Shift   = 0x10,
    Control = 0x11,
    Menu    = 0x12
}


impl FromPrimitive for VKeyCode {
    fn from_i64(n: i64) -> Option<VKeyCode> {
        if 0 <= n {
            VKeyCode::from_u64(n as u64)
        }

        else {
            None
        }
    }

    fn from_u64(n: u64) -> Option<VKeyCode> {
        match n {
            0x08    => Some(VKeyCode::Back   ),
            0x09    => Some(VKeyCode::Tab    ),
            0x0C    => Some(VKeyCode::Clear  ),
            0x0D    => Some(VKeyCode::Enter  ),
            0x13    => Some(VKeyCode::Pause  ),
            0x1B    => Some(VKeyCode::Escape ),
            0x20    => Some(VKeyCode::Space  ),
            0x21    => Some(VKeyCode::PgUp   ),
            0x22    => Some(VKeyCode::PgDn   ),
            0x23    => Some(VKeyCode::End    ),
            0x24    => Some(VKeyCode::Home   ),
            0x29    => Some(VKeyCode::Select ),
            0x2A    => Some(VKeyCode::Print  ),
            0x2B    => Some(VKeyCode::Execute),
            0x2C    => Some(VKeyCode::PrntScr),
            0x2D    => Some(VKeyCode::Insert ),
            0x2E    => Some(VKeyCode::Delete ),
            0x2F    => Some(VKeyCode::Help   ),
            0x30    => Some(VKeyCode::Key0   ),
            0x31    => Some(VKeyCode::Key1   ),
            0x32    => Some(VKeyCode::Key2   ),
            0x33    => Some(VKeyCode::Key3   ),
            0x34    => Some(VKeyCode::Key4   ),
            0x35    => Some(VKeyCode::Key5   ),
            0x36    => Some(VKeyCode::Key6   ),
            0x37    => Some(VKeyCode::Key7   ),
            0x38    => Some(VKeyCode::Key8   ),
            0x39    => Some(VKeyCode::Key9   ),

            0x41    => Some(VKeyCode::A),
            0x42    => Some(VKeyCode::B),
            0x43    => Some(VKeyCode::C),
            0x44    => Some(VKeyCode::D),
            0x45    => Some(VKeyCode::E),
            0x46    => Some(VKeyCode::F),
            0x47    => Some(VKeyCode::G),
            0x48    => Some(VKeyCode::H),
            0x49    => Some(VKeyCode::I),
            0x4a    => Some(VKeyCode::J),
            0x4b    => Some(VKeyCode::K),
            0x4c    => Some(VKeyCode::L),
            0x4d    => Some(VKeyCode::M),
            0x4e    => Some(VKeyCode::N),
            0x4f    => Some(VKeyCode::O),
            0x50    => Some(VKeyCode::P),
            0x51    => Some(VKeyCode::Q),
            0x52    => Some(VKeyCode::R),
            0x53    => Some(VKeyCode::S),
            0x54    => Some(VKeyCode::T),
            0x55    => Some(VKeyCode::U),
            0x56    => Some(VKeyCode::V),
            0x57    => Some(VKeyCode::W),
            0x58    => Some(VKeyCode::X),
            0x59    => Some(VKeyCode::Y),
            0x5a    => Some(VKeyCode::Z),

            0xBA    => Some(VKeyCode::Semi    ),
            0xBB    => Some(VKeyCode::Plus    ),
            0xBC    => Some(VKeyCode::Comma   ),
            0xBD    => Some(VKeyCode::Minus   ),
            0xBE    => Some(VKeyCode::Dot     ),
            0xBF    => Some(VKeyCode::Slash   ),
            0xC0    => Some(VKeyCode::Tilde   ),
            0xDB    => Some(VKeyCode::LBrac   ),
            0xDD    => Some(VKeyCode::RBrac   ),
            0xDC    => Some(VKeyCode::Pipe    ),
            0xDE    => Some(VKeyCode::Quote   ),
            0x5F    => Some(VKeyCode::Sleep   ),
            0x60    => Some(VKeyCode::Num0    ),
            0x61    => Some(VKeyCode::Num1    ),
            0x62    => Some(VKeyCode::Num2    ),
            0x63    => Some(VKeyCode::Num3    ),
            0x64    => Some(VKeyCode::Num4    ),
            0x65    => Some(VKeyCode::Num5    ),
            0x66    => Some(VKeyCode::Num6    ),
            0x67    => Some(VKeyCode::Num7    ),
            0x68    => Some(VKeyCode::Num8    ),
            0x69    => Some(VKeyCode::Num9    ),
            0x6A    => Some(VKeyCode::NumStar ),
            0x6B    => Some(VKeyCode::NumPlus ),
            0x6D    => Some(VKeyCode::NumSub  ),
            0x6E    => Some(VKeyCode::NumDot  ),
            0x6F    => Some(VKeyCode::NumSlash),
            0x70    => Some(VKeyCode::F1      ),
            0x71    => Some(VKeyCode::F2      ),
            0x72    => Some(VKeyCode::F3      ),
            0x73    => Some(VKeyCode::F4      ),
            0x74    => Some(VKeyCode::F5      ),
            0x75    => Some(VKeyCode::F6      ),
            0x76    => Some(VKeyCode::F7      ),
            0x77    => Some(VKeyCode::F8      ),
            0x78    => Some(VKeyCode::F9      ),
            0x79    => Some(VKeyCode::F10     ),
            0x7A    => Some(VKeyCode::F11     ),
            0x7B    => Some(VKeyCode::F12     ),
            0x7C    => Some(VKeyCode::F13     ),
            0x7D    => Some(VKeyCode::F14     ),
            0x7E    => Some(VKeyCode::F15     ),
            0x7F    => Some(VKeyCode::F16     ),
            0x80    => Some(VKeyCode::F17     ),
            0x81    => Some(VKeyCode::F18     ),
            0x82    => Some(VKeyCode::F19     ),
            0x83    => Some(VKeyCode::F20     ),
            0x84    => Some(VKeyCode::F21     ),
            0x85    => Some(VKeyCode::F22     ),
            0x86    => Some(VKeyCode::F23     ),
            0x87    => Some(VKeyCode::F24     ),
            0x90    => Some(VKeyCode::Num     ),
            0x14    => Some(VKeyCode::Caps    ),
            0x91    => Some(VKeyCode::Scroll  ),
            0xA0    => Some(VKeyCode::LShift  ),
            0xA1    => Some(VKeyCode::RShift  ),
            0xA2    => Some(VKeyCode::LCtrl   ),
            0xA3    => Some(VKeyCode::RCtrl   ),
            0xA4    => Some(VKeyCode::LAlt    ),
            0xA5    => Some(VKeyCode::RAlt    ),
            0xA6    => Some(VKeyCode::BBack   ),
            0xA7    => Some(VKeyCode::BFwd    ),
            0xA8    => Some(VKeyCode::BRef    ),
            0xA9    => Some(VKeyCode::BStop   ),
            0xAA    => Some(VKeyCode::BSearch ),
            0xAB    => Some(VKeyCode::BFav    ),
            0xAC    => Some(VKeyCode::BHome   ),
            0xB0    => Some(VKeyCode::MNTrack ),
            0xB1    => Some(VKeyCode::MPTrack ),
            0xB2    => Some(VKeyCode::MStop   ),
            0xB3    => Some(VKeyCode::MPause  ),
            0x25    => Some(VKeyCode::LArrow  ),
            0x26    => Some(VKeyCode::UArrow  ),
            0x27    => Some(VKeyCode::RArrow  ),
            0x28    => Some(VKeyCode::DArrow  ),
            0x15    => Some(VKeyCode::Kana    ),
            0x17    => Some(VKeyCode::Junja   ),
            0x18    => Some(VKeyCode::Final   ),
            0x19    => Some(VKeyCode::Kanji   ),
            0x1C    => Some(VKeyCode::Convert ),

            _ => None
        }
    }
}
