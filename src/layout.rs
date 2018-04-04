use action::Action;
use action::Action::*;
use keycodes::KeyCode::*;
use keymatrix::{COLUMNS, ROWS};

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

pub type Layout = [Action; COLUMNS * ROWS];

pub const LAYERS: [Layout; 4] = [BASE, FN, FN2, BT];

pub const LAYER_FN: u8 = 1;
pub const LAYER_FN2: u8 = 2;
pub const LAYER_BT: u8 = 3;

// activate by indexing into LAYERS
const FN_M: Action = LayerMomentary(LAYER_FN);
const FN2_M: Action = LayerMomentary(LAYER_FN2);
const __: Action = Transparent;
const LED_NT: Action = LedNextTheme;
const LED_NB: Action = LedNextBrightness;
const LED_NAS: Action = LedNextAnimationSpeed;
const BT_ON: Action = LayerOn(LAYER_BT);

pub const BASE: Layout = layout![
    Escape   N1     N2   N3 N4 N5    N6 N7 N8    N9  N0     Minus    Equal     BSpace
    Tab      Q      W    E  R  T     Y  U  I     O   P      LBracket RBracket  BSlash
    Capslock A      S    D  F  G     H  J  K     L   SColon Quote    No        Enter
    LShift   Z      X    C  V  B     N  M  Comma Dot Slash  No       No        RShift
    LCtrl    LMeta  LAlt No No Space No No No    No  RAlt   FN_M     FN2_M     RCtrl
];

pub const FN: Layout = layout![
  Grave F1   F2   F3    F4        F5      F6     F7     F8   F9         F10    F11    F12 __
  __    __   Up   __    LedToggle LED_NAS LED_NB LED_NT Up   Scrolllock Pause  Home   End PScreen
  __    Left Down Right __        __      __     Left   Down Right      PgUp   PgDown No  __
  __    __   __   __    __        BT_ON   __     __     __   Insert     Delete No     No  __
  __    __   __   No    No        Reset   No     No     No   No         __     __     __  __
];

pub const FN2: Layout = layout![
    LedOff LedOn LED_NT LED_NAS LED_NB __ __ __ __ __ __ __ __ __
    __     __    __     __      __     __ __ __ __ __ __ __ __ __
    __     __    __     __      __     __ __ __ __ __ __ __ No __
    __     __    __     __      __     __ __ __ __ __ __ __ __ __
    __     __    __     No      No     __ No No No No __ __ __ __
];

#[cfg_attr(rustfmt, rustfmt_skip)]
pub const BT: Layout = layout![
    LayerOff(LAYER_BT) BtConnectHost(1) BtConnectHost(2) BtConnectHost(3) BtConnectHost(4) __ __ __ __ __ BtToggleLegacyMode BtOff BtBroadcast BtOn
    __ BtSaveHost(1) BtSaveHost(2) BtSaveHost(3) BtSaveHost(4) __ __ __ __ __ __ __ __ __
    __ BtDeleteHost(1) BtDeleteHost(2) BtDeleteHost(3) BtDeleteHost(4) __ __ __ __ __ __ __ No __
    __ __ __ __ __ LayerOff(LAYER_BT) __ __ __ __ __ __ __ __
    BtHostListQuery __ __ No No __ No No No No __ __ __ __
];
