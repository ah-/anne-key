use keycodes::KeyCode;
use keycodes::KeyCode::*;

/*
  ,-----------------------------------------------------------------------------.
  |Esc   |  1|   2|   3|   4|   5|   6|   7|   8|   9|   0|   -|   = |   Backsp |
  |-----------------------------------------------------------------------------|
  |Tab    |  Q  |  W  |  E  |  R  |  T  |  Y  |  U  |  I|   O|  P|  [|  ]|  \ | |
  |-----------------------------------------------------------------------------|
  |Caps         |    A|    S|    D|    F|   G|  H|  J|  K|  L|  ;|  '|   #|Enter|
  |-----------------------------------------------------------------------------|
  |Shift      |    Z|     X|    C|     V|  B|  N|  M|  ,|  .|  /|     Shift     |
  |-----------------------------------------------------------------------------|
  |Ctrl |Meta | Alt |               Space                |Alt | Fn  | Anne |Ctrl|
  `-----------------------------------------------------------------------------'
*/


pub const DEFAULT: [KeyCode; 70] = [
    Escape,   N1,     N2,   N3, N4, N5,    N6, N7, N8,    N9,    N0,     Minus,     Equal,     BSpace,
    Tab,      Q,      W,    E,  R,  T,     Y,  U,  I,     O,     P,      LBracket,  RBracket,  BSlash,
    Capslock, A,      S,    D,  F,  G,     H,  J,  K,     L,     SColon, Quote,     NonUSHash, Enter,
    LShift,   Z,      X,    C,  V,  B,     N,  M,  Comma, Dot,   Slash,  No,        No,        RShift,
    LCtrl,    LMeta,  LAlt, No, No, Space, No, No, No,    No,    RAlt,   No,        No,        RCtrl,
];
