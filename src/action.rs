use keycodes::KeyCode;

#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq)]
pub enum Action {
    Nop,
    Transparent,

    Key(KeyCode), // = 0x10

    LayerMomentary(u8), // = 0x20,
    LayerToggle(u8),
    LayerOn(u8),
    LayerOff(u8),

    LedOn, // = 0x30,
    LedOff,
    LedToggle,
    LedNextTheme,
    LedNextBrightness,
    LedNextAnimationSpeed,
    LedTheme(u8),

    //Bluetooth = 0x40,
    BtOn,
    BtOff,
    BtSaveHost(u8),
    BtConnectHost(u8),
    BtDeleteHost(u8),
    BtBroadcast,
    BtCompatibilityMode(bool),
    BtToggleCompatibilityMode,
    BtHostListQuery, // TODO: remove? this shouldn't really be here
}

// Allow auto-conversion of KeyCodes to Action for nicer layout formatting
// and drop commas
macro_rules! layout {
    ( $( $e: expr )* ) => {
        [
            $(
                $e.to_action(),
            )*
        ]
    };
}

impl KeyCode {
    pub const fn to_action(self) -> Action {
        Action::Key(self)
    }
}

impl Action {
    pub const fn to_action(self) -> Action {
        self
    }
}
