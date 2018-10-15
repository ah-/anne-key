#![allow(dead_code)]

// USB HID KeyCodes
#[derive(PartialOrd, PartialEq, Copy, Clone)]
pub enum KeyCode {
    No = 0x00,
    RollOver,
    PostFail,
    Undefined,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M, // 0x10
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    N1,
    N2,
    N3, // 0x20
    N4,
    N5,
    N6,
    N7,
    N8,
    N9,
    N0,
    Enter,
    Escape,
    BSpace,
    Tab,
    Space,
    Minus,
    Equal,
    LBracket,
    RBracket,  // 0x30
    BSlash,    // \ (and |)
    NonUSHash, // Non-US # and ~ (Typically near the Enter key)
    SColon,    // ; (and :)
    Quote,     // ' and "
    Grave,     // Grave accent and tilde
    Comma,     // , and <
    Dot,       // . and >
    Slash,     // / and ?
    Capslock,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7, // 0x40
    F8,
    F9,
    F10,
    F11,
    F12,
    PScreen,
    Scrolllock,
    Pause,
    Insert,
    Home,
    PgUp,
    Delete,
    End,
    PgDown,
    Right,
    Left, // 0x50
    Down,
    Up,
    Numlock,
    KpSlash,
    KpAsterisk,
    KpMinus,
    KpPlus,
    KpEnter,
    Kp1,
    Kp2,
    Kp3,
    Kp4,
    Kp5,
    Kp6,
    Kp7,
    Kp8, // 0x60
    Kp9,
    Kp0,
    KpDot,
    NonUSBackslash, // Non-US \ and | (Typically near the Left-Shift key)
    Application,    // 0x65 - Max keycode the Bluetooth HID descriptor supports

    // Modifiers
    LCtrl = 0xE0,
    LShift,
    LAlt,
    LMeta,
    RCtrl,
    RShift,
    RAlt,
    RMeta, // 0xE7
}

impl KeyCode {
    pub fn is_modifier(self) -> bool {
        self >= KeyCode::LCtrl && self <= KeyCode::RMeta
    }

    pub fn is_normal_key(self) -> bool {
        self >= KeyCode::A && self <= KeyCode::Application
    }
}

/// Index of each physical key in the scan matrix
#[cfg_attr(rustfmt, rustfmt_skip)]
pub enum KeyIndex {
    Escape,   N1,    N2,   N3,  N4,  N5,    N6,  N7,  N8,    N9,   N0,     Minus,    Equal,    BSpace,
    Tab,      Q,     W,    E,   R,   T,     Y,   U,   I,     O,    P,      LBracket, RBracket, BSlash,
    Capslock, A,     S,    D,   F,   G,     H,   J,   K,     L,    SColon, Quote,    No1,      Enter,
    LShift,   Z,     X,    C,   V,   B,     N,   M,   Comma, Dot,  Slash,  No2,      No3,      RShift,
    LCtrl,    LMeta, LAlt, No4, No5, Space, No6, No7, No8,   No9,  RAlt,   FN,       Anne,     RCtrl
}
