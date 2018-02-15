use bluetooth::Bluetooth;
use hidreport::HidReport;
use keycodes::KeyCode;
use keymatrix::KeyState;
use layout::DEFAULT;
use led::Led;

pub struct Keyboard {
    // TODO: instead of counting keys just store full previous packed snapshot and compare that
    // or might not even need that after switching to wakeup only handling?
    num_pressed_keys: usize,
}

impl Keyboard {
    pub const fn new() -> Keyboard { Keyboard { num_pressed_keys: 0 } }

    pub fn process(&mut self, state: &KeyState, bluetooth: &mut Bluetooth, led: &mut Led) {
        let pressed = state.into_iter().filter(|s| **s).count();
        if pressed != self.num_pressed_keys {
            self.num_pressed_keys = pressed;

            let mut hid = HidProcessor::new();

            let layout = &DEFAULT;

            for (key, pressed) in state.iter().enumerate() {
                if *pressed {
                    let code = &layout[key];
                    hid.process(*code);
                }
            }

            bluetooth.send_report(&hid.report);
            test_led(led, state);
            led.send_keys(state);
        }
    }
}


#[repr(packed)]
struct HidProcessor {
    pub report: HidReport,
    i: usize,
}

impl HidProcessor {
    fn new() -> HidProcessor {
        HidProcessor {
            report: HidReport::new(),
            i: 0
        }
    }

    fn process(&mut self, code: KeyCode) {
        if code.is_modifier() {
            self.report.modifiers |= 1 << (code as u8 - KeyCode::LCtrl as u8);
        } else if code.is_normal_key() && self.i < self.report.keys.len() {
            self.report.keys[self.i] = code as u8;
            self.i += 1;
        }
    }
}

fn test_led(led: &mut Led, state: &KeyState) {
    if state[0] {
        led.off();
    }
    if state[1] {
        led.on();
    }
    if state[2] {
        led.next_theme();
    }
    if state[3] {
        led.next_brightness();
    }
    if state[4] {
        led.next_animation_speed();
    }
    if state[15] {
        led.set_theme(0);
    }
    if state[16] {
        led.set_theme(1);
    }
    if state[17] {
        led.set_theme(2);
    }
    if state[18] {
        led.set_theme(3);
    }
    if state[19] {
        led.set_theme(14);
    }
    if state[20] {
        led.set_theme(17);
    }
    if state[21] {
        led.set_theme(18);
    }
    if state[22] {
        led.send_keys(state);
    }
    if state[23] {
        led.send_music(&[1,2,3,4,5,6,7,8,9]);
    }
}

