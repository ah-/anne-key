use action::Action;
use bluetooth::Bluetooth;
use debug::UnwrapLog;
use hidreport::HidReport;
use keycodes::KeyCode;
use keymatrix::KeyState;
use layout::LAYERS;
use led::Led;

pub struct Keyboard {
    layers: Layers,
    previous_state: KeyState, // TODO: use packed state here
}


fn eq(sa: &KeyState, sb: &KeyState) -> bool {
    sa.iter().zip(sb.iter()).all(|(a,b)| a == b)
}

impl Keyboard {
    pub const fn new() -> Keyboard {
        Keyboard {
            layers: Layers::new(),
            previous_state: [false; 70],
        }
    }

    fn get_action(&self, key: usize) -> Action {
        let mut action = Action::Transparent;

        for i in (0..LAYERS.len()).rev() {
            if self.layers.current & (1 << i) != 0 {
                action = LAYERS[i][key];
            }
            if action != Action::Transparent {
                break;
            }
        }

        action
    }

    pub fn process(&mut self, state: &KeyState, bluetooth: &mut Bluetooth, led: &mut Led) {
        // TODO: might not even need this check after switching to wakeup only handling?
        if !eq(&self.previous_state, state) {
            let mut hid = HidProcessor::new();

            for (key, pressed) in state.iter().enumerate() {
                let changed = self.previous_state[key] != *pressed;

                // Only handle currently pressed and changed keys to
                // cut down on processing time.
                if *pressed || changed {
                    let action = self.get_action(key);
                    hid.process(&action, *pressed, changed);
                    led.process(&action, *pressed, changed);
                    bluetooth.process(&action, *pressed, changed);
                    self.layers.process(&action, *pressed, changed);
                }
            }

            // TODO: need to comment this out for now for setup msgs to go through
            // probably needs a buffer / or not send this when setup got sent
            // or not send this if nothing changed
            bluetooth.send_report(&hid.report).log_error();
            led.send_keys(state).log_error();
            self.layers.finish();

            self.previous_state = *state;
        }
    }
}

trait EventProcessor {
    fn process(&mut self, action: &Action, pressed: bool, changed: bool);
    fn finish(&mut self) {}
}

struct Layers {
    current: u8,
    next: u8,
}

impl Layers {
    const fn new() -> Layers {
        Layers {
            current: 0b1,
            next: 0b1,
        }
    }
}

impl EventProcessor for Layers {
    fn process(&mut self, action: &Action, pressed: bool, changed: bool) {
        if changed {
            match (action, pressed) {
                (&Action::LayerMomentary(layer), true) => { self.next |= 1 << layer },
                (&Action::LayerMomentary(layer), false) => { self.next &= !(1 << layer) },
                (&Action::LayerToggle(layer), true) => { self.next ^= 1 << layer },
                (&Action::LayerOn(layer), true) => { self.next |= 1 << layer },
                (&Action::LayerOff(layer), true) => { self.next &= !(1 << layer)},
                _ => {}
            }
        }
    }

    fn finish(&mut self) {
        self.current = self.next;
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
    fn process(&mut self, action: &Action, pressed: bool, _changed: bool) {
        if pressed {
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
}

impl<'a> EventProcessor for Led<'a> {
    fn process(&mut self, action: &Action, pressed: bool, changed: bool) {
        if changed && pressed {
            let result = match action {
                &Action::LedOn => self.on(),
                &Action::LedOff => self.off(),
                &Action::LedNT => self.next_theme(),
                &Action::LedNB => self.next_brightness(),
                &Action::LedNAS => self.next_animation_speed(),
                &Action::LedTheme(theme_id) => self.set_theme(theme_id),
                _ => Ok(())
            };
            result.log_error()
        }
    }
}

impl<'a> EventProcessor for Bluetooth<'a> {
    fn process(&mut self, action: &Action, pressed: bool, changed: bool) {
        if changed && pressed {
            let result = match action {
                &Action::BtOn => self.on(),
                &Action::BtOff => self.off(),
                &Action::BtSaveHost(host) => self.save_host(host),
                &Action::BtConnectHost(host) => self.connect_host(host),
                &Action::BtDeleteHost(host) => self.delete_host(host),
                &Action::BtBroadcast => self.broadcast(),
                &Action::BtCompatibilityMode(on) => self.enable_compatibility_mode(on),
                &Action::BtHostListQuery => self.host_list_query(),
                _ => Ok(())
            };
            result.log_error()
        }
    }
}
