use super::keyboard::KeyState;

pub struct HidReport{pub bytes: [u8; 8]}

impl HidReport {
    pub fn from_key_state(state: &KeyState) -> HidReport {
        let mut bytes = [0; 8];

        if state[0] {
            bytes[2] = 0x4;
        } else if state[1] {
            bytes[2] = 0x5;
        } else if state[2] {
            bytes[2] = 0x6;
        } else if state[3] {
            bytes[2] = 0x7;
        } else if state[4] {
            bytes[2] = 0x8;
        } else {
            let pressed = state.into_iter().filter(|s| **s).count();
            if pressed > 0 {
                bytes[2] = 0x9;
            }
        }

        HidReport{bytes}
    }
}
