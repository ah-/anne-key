use keycodes::KeyCode;

#[derive(Copy, Clone)]
pub enum Action {
    Nop,
    Trans,

    Key(KeyCode), // = 0x10

    LayerMomentary(u8), // = 0x20,
    LayerToggle(u8),
    LayerOn(u8),
    LayerOff(u8),

    LedOn, // = 0x30,
    LedOff,
    LedNextTheme,
    LedNextBrightness,
    LedNextAnimationSpeed,
    LedTheme(u8),

    //Bluetooth = 0x40,
}
