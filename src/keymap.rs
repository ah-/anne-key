use core::slice;
use keyboard::KeyState;
use layout::DEFAULT;
use keycodes::KeyCode;

#[repr(packed)]
pub struct HidReport {
    pub modifiers: u8,
    pub _unused: u8,
    pub keys: [u8; 6]
}

impl HidReport {
    pub fn from_key_state(state: &KeyState) -> HidReport {
        let layout = &DEFAULT;

        let mut modifiers: u8 = 0;
        let mut keys: [u8; 6] = [0; 6];
        let mut i: usize = 0;

        for (key, pressed) in state.iter().enumerate() {
            if *pressed {
                let code = &layout[key];

                if code.is_modifier() {
                    modifiers |= 1 << (*code as u8 - KeyCode::LCtrl as u8);
                } else if code.is_normal_key() {
                    if i < keys.len() {
                        keys[i] = *code as u8;
                        i += 1;
                    }
                }
            }
        }

        HidReport {
            modifiers: modifiers,
            _unused: 0,
            keys: keys,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe { 
            let p : *const HidReport = self;
            slice::from_raw_parts(p as *const u8, 8)
        }
    }
}
