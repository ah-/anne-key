use action::Action;
use action::Action::*;
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

pub const LAYERS: [Layout; 4] = [BASE, FN, LED, BT];

// activate by indexing into LAYERS
const FN_M: Action = LayerMomentary(1);
const LED_M: Action = LayerMomentary(2);
const BT_T: Action = LayerToggle(3);
const __: Action = Transparent;

pub const BASE: Layout = layout![
    Escape   N1     N2   N3 N4 N5    N6 N7 N8    N9  N0     Minus    Equal     BSpace
    Tab      Q      W    E  R  T     Y  U  I     O   P      LBracket RBracket  BSlash
    Capslock A      S    D  F  G     H  J  K     L   SColon Quote    No        Enter
    LShift   Z      X    C  V  B     N  M  Comma Dot Slash  No       No        RShift
    LCtrl    LMeta  LAlt No No Space No No No    No  RAlt   FN_M     BT_T      LED_M
];

pub const FN: Layout = layout![
  Grave F1   F2   F3    F4 F5 F6 F7   F8   F9         F10   F11    F12    __
  __    __   Up   __    __ __ __ __   Up   Scrolllock Pause Home   End    PScreen
  __    Left Down Right __ __ __ Left Down Right      PgUp  PgDown No     __
  __    __   __   __    __ __ __ __   __   __         __    Insert Delete __
  __    __   __   No    No __ No No   No   No         __    __     __     __
];

pub const LED: Layout = layout![
    LedOff LedOn LedNextTheme LedNextAnimationSpeed LedNextBrightness __ __ __ __ __ __ __ __ __
    __ LedTheme(0) LedTheme(1) LedTheme(2) LedTheme(14) LedTheme(17) LedTheme(18) __ __ __ __ __ __ __
    __ __ __ __ __ __ __ __ __ __ __ __ No __
    __ __ __ __ __ __ __ __ __ __ __ __ __ __
    __ __ __ No No __ No No No No __ __ __ __
];

pub const BT: Layout = layout![
    BtOff BtOn BtConnectHost(0) BtConnectHost(1) BtConnectHost(2) BtConnectHost(3) __ __ __ __ __ __ __ __
    BtBroadcast BtSaveHost(0) BtSaveHost(1) BtSaveHost(2) BtSaveHost(3) __ __ __ __ __ __ __ __ __
    BtCompatibilityMode(true) BtDeleteHost(0) BtDeleteHost(1) BtDeleteHost(2) BtDeleteHost(3) __ __ __ __ __ __ __ No __
    BtCompatibilityMode(false) __ __ __ __ __ __ __ __ __ __ __ __ __
    LedNextTheme BtHostListQuery __ No No __ No No No No __ __ __ __
];
