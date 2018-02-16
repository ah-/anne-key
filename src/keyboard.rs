use action::Action;
use bluetooth::Bluetooth;
use hidreport::HidReport;
use keycodes::KeyCode;
use keymatrix::KeyState;
use layout::DEFAULT;
use layout::TEST;
use led::Led;

pub struct Keyboard {
    // TODO: instead of counting keys just store full previous packed snapshot and compare that
    // or might not even need that after switching to wakeup only handling?
    num_pressed_keys: usize,
    layers: Layers,
}

impl Keyboard {
    pub const fn new() -> Keyboard {
        Keyboard {
            num_pressed_keys: 0,
            layers: Layers::new(),
        }
    }

    pub fn process(&mut self, state: &KeyState, bluetooth: &mut Bluetooth, led: &mut Led) {
        let pressed = state.into_iter().filter(|s| **s).count();
        if pressed != self.num_pressed_keys {
            self.num_pressed_keys = pressed;

            let mut hid = HidProcessor::new();

            let layout = &TEST;

            for (key, pressed) in state.iter().enumerate() {
                if *pressed {
                    let action = &layout[key];
                    hid.process(action);
                    led.process(action);
                    self.layers.process(action);
                }
            }

            bluetooth.send_report(&hid.report);
            led.send_keys(state);
            self.layers.finish();
        }
    }
}

trait EventProcessor {
    fn process(&mut self, action: &Action);
    fn finish(&mut self) {}
}

struct Layers {
    current: u8,
    next: u8,
}

impl Layers {
    const fn new() -> Layers {
        Layers {
            // TODO: array of layers -> enabled or not
            current: 0,
            next: 0,
        }
    }
}

impl EventProcessor for Layers {
    fn process(&mut self, action: &Action) {
        match action {
            &Action::LayerMomentary(layer) => { self.next = layer },
            &Action::LayerToggle(layer) => { self.next = layer }, // TODO
            &Action::LayerOn(layer) => { self.next = layer }, // TODO
            &Action::LayerOff(layer) => { self.next = layer }, // TODO
            _ => {}
        }
    }

    fn finish(&mut self) {
        self.current = self.next;
        self.next = 0;
    }
}

struct HidProcessor {
    pub report: HidReport,
    i: usize,
}

impl HidProcessor {
    fn new() -> HidProcessor {
        HidProcessor {
            report: HidReport::new(),
            i: 0,
        }
    }
}

impl EventProcessor for HidProcessor {
    fn process(&mut self, action: &Action) {
        match action {
            &Action::Key(code) => {
                if code.is_modifier() {
                    self.report.modifiers |= 1 << (code as u8 - KeyCode::LCtrl as u8);
                } else if code.is_normal_key() && self.i < self.report.keys.len() {
                    self.report.keys[self.i] = code as u8;
                    self.i += 1;
                }
            },
            _ => {}
        }
    }
}

impl<'a> EventProcessor for Led<'a> {
    fn process(&mut self, action: &Action) {
        match action {
            &Action::LedOn => self.on(),
            &Action::LedOff => self.off(),
            &Action::LedNextTheme => self.next_theme(),
            &Action::LedNextBrightness => self.next_brightness(),
            &Action::LedNextAnimationSpeed => self.next_animation_speed(),
            &Action::LedTheme(theme_id) => self.set_theme(theme_id),
            _ => {}
        }
    }
}
