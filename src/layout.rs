use action::Action;
use action::Action::*;
use keycodes::KeyCode;
use keycodes::KeyCode::*;

/*
  ,-----------------------------------------------------------------------------.
  |Esc   |  1|   2|   3|   4|   5|   6|   7|   8|   9|   0|   -|   = |   Backsp |
  |-----------------------------------------------------------------------------|
  |Tab    |  Q  |  W  |  E  |  R  |  T  |  Y  |  U  |  I|   O|  P|  [|  ]|  \ ] |
  |-----------------------------------------------------------------------------|
  |Caps         |    A|    S|    D|    F|   G|  H|  J|  K|  L|  ;|  '|   #|Enter|
  |-----------------------------------------------------------------------------|
  |Shift      |    Z|     X|    C|     V|  B|  N|  M|  ,|  .|  /|     Shift     |
  |-----------------------------------------------------------------------------|
  |Ctrl |Meta | Alt |               Space                |Alt | Fn  | Anne |Ctrl|
  `-----------------------------------------------------------------------------'
*/

pub type Layout = [Action; 70];

const LON1: Action = LayerOn(1);
const LOFF1: Action = LayerOff(1);
const __: Action = Transparent;

pub const BASE: Layout = layout![
    Escape   N1     N2   N3 N4 N5    N6 N7 N8    N9  N0     Minus    Equal     BSpace
    Tab      Q      W    E  R  T     Y  U  I     O   P      LBracket RBracket  BSlash
    Capslock A      S    D  F  G     H  J  K     L   SColon Quote    NonUSHash Enter
    LShift   Z      X    C  V  B     N  M  Comma Dot Slash  No       No        RShift
    LCtrl    LMeta  LAlt No No Space No No No    No  RAlt   LON1     LOFF1     RCtrl
];

pub const FN1: Layout = layout![
    LedOff LedOn LedNextTheme LedNextAnimationSpeed LedNextBrightness __ __ __ __ __ __ __ __ __
    Q  LedTheme(0) LedTheme(1) LedTheme(2) LedTheme(14) LedTheme(17) LedTheme(18) __ __ __ __ __ __ __
    __ __ __ __ __ __ __ __ __ __ __ __ __ __
    __ __ __ __ __ __ __ __ __ __ __ __ __ __
    LayerMomentary(1) LayerToggle(1) LayerOn(1) LayerOff(1) __ __ __ __ __ __ __ __ __ __
];

pub const NONE: Layout = [Nop; 70];
