use crate::action::Action;
use crate::bluetooth::Bluetooth;
use crate::debug::UnwrapLog;
use crate::hidreport::HidReport;
use crate::keycodes::KeyCode;
use crate::keymatrix::{KeyState, COLUMNS, ROWS};
use crate::layout::LAYERS;
use crate::layout::LAYER_BT;
use crate::led::Led;
use crate::usb::Usb;
use bit_field::{BitArray, BitField};
use core::marker::Unsize;
use stm32l1::stm32l151::SCB;

pub struct Keyboard {
    layers: Layers,
    previous_state: KeyState,
    pub send_usb_report: bool,
}

impl Keyboard {
    pub const fn new() -> Keyboard {
        Keyboard {
            layers: Layers::new(),
            previous_state: [0; 9],
            send_usb_report: true,
        }
    }

    /// Get the action for `key`.

    /// The top non-Transparent action at index `key` amongst the
    /// currently active layers is returned.
    fn get_action(&self, key: usize) -> Action {
        let mut action = Action::Transparent;

        for i in (0..LAYERS.len()).rev() {
            if self.layers.current.get_bit(i) {
                action = LAYERS[i][key];
            }
            if action != Action::Transparent {
                break;
            }
        }

        action
    }

    pub fn process<BUFFER>(
        &mut self,
        state: &KeyState,
        bluetooth: &mut Bluetooth<BUFFER>,
        led: &mut Led<BUFFER>,
        usb: &mut Usb,
    ) where
        BUFFER: Unsize<[u8]>,
    {
        // TODO: might not even need this check after switching to wakeup only handling?
        if &self.previous_state != state {
            let mut hid = HidProcessor::default();

            for key in 0..COLUMNS * ROWS {
                let pressed = state.get_bit(key);
                let changed = self.previous_state.get_bit(key) != pressed;

                // Only handle currently pressed and changed keys to
                // cut down on processing time.
                if pressed || changed {
                    let action = self.get_action(key);
                    if pressed && Action::Reset == action {
                        crate::heprintln!("system reset").ok();
                        SCB::system_reset2()
                    }
                    if pressed && Action::UsbToggle == action {
                        self.send_usb_report = !self.send_usb_report;
                        crate::heprintln!("send_usb_report: {:?}", self.send_usb_report).ok();
                    }
                    hid.process(&action, pressed, changed);
                    led.process(&action, pressed, changed);
                    bluetooth.process(&action, pressed, changed);
                    self.layers.process(&action, pressed, changed);
                }
            }

            let bt_layer_current: bool = self.bluetooth_mode_enabled();
            let bt_layer_next: bool = self.layers.next.get_bit(LAYER_BT as usize);
            if bt_layer_next && !bt_layer_current {
                bluetooth.update_led(led, self.send_usb_report).log_error();
            } else if bt_layer_current && !bt_layer_next {
                led.theme_mode().log_error();
            }

            self.layers.finish();

            bluetooth.send_report(&hid.report).log_error();
            led.send_keys(state).log_error();
            if self.send_usb_report {
                usb.update_report(&hid.report);
            }

            self.previous_state = *state;
        }
    }

    pub fn bluetooth_mode_enabled(&self) -> bool {
        self.layers.current.get_bit(LAYER_BT as usize)
    }

    pub fn disable_bluetooth_mode(&mut self) {
        self.layers.current.set_bit(LAYER_BT as usize, false);
    }
}

trait EventProcessor {
    fn process(&mut self, action: &Action, pressed: bool, changed: bool);
    fn finish(&mut self) {}
}

/// Bit-field of the currently active layers, indexed by position in
/// [`layout::LAYERS`].
struct Layers {
    current: u8,
    /// Active layers after action processing is finished
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
            match (*action, pressed) {
                (Action::LayerMomentary(layer), _) => self.next.set_bit(layer as usize, pressed),
                (Action::LayerToggle(layer), true) => {
                    let current = self.next.get_bit(layer as usize);
                    self.next.set_bit(layer as usize, !current)
                }
                (Action::LayerOn(layer), true) => self.next.set_bit(layer as usize, true),
                (Action::LayerOff(layer), true) => self.next.set_bit(layer as usize, false),
                _ => &mut self.next,
            };
        }
    }

    fn finish(&mut self) {
        self.current = self.next;
    }
}

#[derive(Default)]
struct HidProcessor {
    pub report: HidReport,
    /// Number of normal keys to be sent in `report`
    i: usize,
}

impl EventProcessor for HidProcessor {
    fn process(&mut self, action: &Action, pressed: bool, _changed: bool) {
        if pressed {
            if let Action::Key(code) = *action {
                if code.is_modifier() {
                    self.report
                        .modifiers
                        .set_bit(code as usize - KeyCode::LCtrl as usize, true);
                } else if code.is_normal_key() && self.i < self.report.keys.len() {
                    self.report.keys[self.i] = code as u8;
                    self.i += 1;
                }
            }
        }
    }
}

impl<BUFFER> EventProcessor for Led<BUFFER>
where
    BUFFER: Unsize<[u8]>,
{
    fn process(&mut self, action: &Action, pressed: bool, changed: bool) {
        if changed && pressed {
            let result = match *action {
                Action::LedOn => self.on(),
                Action::LedOff => self.off(),
                Action::LedToggle => self.toggle(),
                Action::LedNextTheme => self.next_theme(),
                Action::LedNextBrightness => self.next_brightness(),
                Action::LedNextAnimationSpeed => self.next_animation_speed(),
                Action::LedTheme(theme_id) => self.set_theme(theme_id),
                _ => Ok(()),
            };
            result.log_error()
        }
    }
}

impl<BUFFER> EventProcessor for Bluetooth<BUFFER>
where
    BUFFER: Unsize<[u8]>,
{
    fn process(&mut self, action: &Action, pressed: bool, changed: bool) {
        if changed && pressed {
            let result = match *action {
                Action::BtOn => self.on(),
                Action::BtOff => self.off(),
                Action::BtSaveHost(host) => self.save_host(host),
                Action::BtConnectHost(host) => self.connect_host(host),
                Action::BtDeleteHost(host) => self.delete_host(host),
                Action::BtBroadcast => self.broadcast(),
                Action::BtLegacyMode(on) => self.enable_legacy_mode(on),
                Action::BtToggleLegacyMode => self.toggle_legacy_mode(),
                Action::BtHostListQuery => self.host_list_query(),
                _ => Ok(()),
            };
            result.log_error()
        }
    }
}
