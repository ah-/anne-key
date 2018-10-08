use super::keymatrix::KeyState;
use super::protocol::{LedOp, Message, MsgType};
use super::serial::led_usart::LedUsart;
use super::serial::{Serial, Transfer};
use bluetooth::BluetoothMode;
use core::marker::Unsize;
use embedded_hal::digital::OutputPin;
use hal::gpio::gpioc::PC15;
use hal::gpio::{Input, Output};
use keycodes::KeyIndex;
use nb;
use rtfm::Threshold;
use stm32l151::SYST;

pub enum LedMode {
    _Off,
    On,
    Flash,
}

pub struct Led<BUFFER: 'static + Unsize<[u8]>> {
    pub serial: Serial<LedUsart, BUFFER>,
    pub rx_transfer: Option<Transfer<BUFFER>>,
    pub pc15: PC15<Output>,
    pub state: bool,
}

impl<BUFFER> Led<BUFFER>
where
    BUFFER: Unsize<[u8]>,
{
    pub fn new(
        mut serial: Serial<LedUsart, BUFFER>,
        rx_buffer: &'static mut BUFFER,
        pc15: PC15<Input>,
    ) -> Led<BUFFER> {
        let rx_transfer = serial.receive(rx_buffer);
        Led {
            serial,
            rx_transfer: Some(rx_transfer),
            pc15: pc15.into_output().pull_up(),
            state: false,
        }
    }

    pub fn on(&mut self) -> nb::Result<(), !> {
        self.pc15.set_high();
        Ok(())
    }

    pub fn off(&mut self) -> nb::Result<(), !> {
        self.pc15.set_low();
        self.state = false;
        Ok(())
    }

    pub fn poke(&mut self, syst: &SYST) -> nb::Result<(), !> {
        self.off()?;

        // TODO: introduce proper delay()
        let wait_until_tick = 0;
        while syst.cvr.read() > wait_until_tick {}

        self.on()?;

        while syst.cvr.read() > wait_until_tick {}

        Ok(())
    }

    pub fn toggle(&mut self) -> nb::Result<(), !> {
        self.state = !self.state;
        if self.state {
            self.theme_mode()
        } else {
            self.set_theme(0)
        }
    }

    // next_* cycles through themes/brightness/speed
    pub fn next_theme(&mut self) -> nb::Result<(), !> {
        self.serial
            .send(MsgType::Led, LedOp::ConfigCmd as u8, &[1, 0, 0])
    }

    pub fn next_brightness(&mut self) -> nb::Result<(), !> {
        self.serial
            .send(MsgType::Led, LedOp::ConfigCmd as u8, &[0, 0, 1])
    }

    pub fn next_animation_speed(&mut self) -> nb::Result<(), !> {
        self.serial
            .send(MsgType::Led, LedOp::ConfigCmd as u8, &[0, 1, 0])
    }

    pub fn set_theme(&mut self, theme: u8) -> nb::Result<(), !> {
        self.serial
            .send(MsgType::Led, LedOp::ThemeMode as u8, &[theme])
    }

    pub fn send_keys(&mut self, state: &KeyState) -> nb::Result<(), !> {
        self.serial.send(MsgType::Led, LedOp::Key as u8, state)
    }

    pub fn send_music(&mut self, keys: &[u8]) -> nb::Result<(), !> {
        self.serial.send(MsgType::Led, LedOp::Music as u8, keys)
    }

    pub fn get_theme_id(&mut self) -> nb::Result<(), !> {
        // responds with with [ThemeId]
        self.serial.send(MsgType::Led, LedOp::GetThemeId as u8, &[])
    }

    pub fn set_keys(&mut self, payload: &[u8]) -> nb::Result<(), !> {
        self.serial
            .send(MsgType::Led, LedOp::SetIndividualKeys as u8, payload)
    }

    pub fn theme_mode(&mut self) -> nb::Result<(), !> {
        self.state = true;
        self.serial.send(MsgType::Led, LedOp::ThemeMode as u8, &[])
    }

    pub fn bluetooth_mode(
        &mut self,
        saved_hosts: u8,
        connected_host: u8,
        mode: BluetoothMode,
    ) -> nb::Result<(), !> {
        let mode_color = match mode {
            BluetoothMode::Unknown => (0xff, 0, 0),
            BluetoothMode::Ble => (0, 0xff, 0),
            BluetoothMode::Legacy => (0xff, 0xff, 0),
        };

        let s1 = if (saved_hosts & 1) != 0 { 0xFF } else { 0x00 };
        let s2 = if (saved_hosts & 2) != 0 { 0xFF } else { 0x00 };
        let s3 = if (saved_hosts & 4) != 0 { 0xFF } else { 0x00 };
        let s4 = if (saved_hosts & 8) != 0 { 0xFF } else { 0x00 };

        let mut c1 = 0x00;
        let mut c2 = 0x00;
        let mut c3 = 0x00;
        let mut c4 = 0x00;
        let mut cu = 0x00;

        match connected_host {
            1 => c1 = 0xFF,
            2 => c2 = 0xFF,
            3 => c3 = 0xFF,
            4 => c4 = 0xFF,
            12 => cu = 0xFF,
            _ => {}
        }

        #[cfg_attr(rustfmt, rustfmt_skip)]
        let payload = &[0xca,
                        19, // the number of keys in this request
            KeyIndex::Escape as u8, 0xff, 0xff, 0x00, LedMode::On as u8,
            // Select host
            KeyIndex::N1 as u8,     cu, 0xff, c1, LedMode::On as u8,
            KeyIndex::N2 as u8,     cu, 0xff, c2, LedMode::On as u8,
            KeyIndex::N3 as u8,     cu, 0xff, c3, LedMode::On as u8,
            KeyIndex::N4 as u8,     cu, 0xff, c4, LedMode::On as u8,
            // Save host
            KeyIndex::Q as u8,      0x00, s1, 0xff, LedMode::On as u8,
            KeyIndex::W as u8,      0x00, s2, 0xff, LedMode::On as u8,
            KeyIndex::E as u8,      0x00, s3, 0xff, LedMode::On as u8,
            KeyIndex::R as u8,      0x00, s4, 0xff, LedMode::On as u8,
            // Delete host
            KeyIndex::A as u8,      s1, 0x00, 0x00, LedMode::On as u8,
            KeyIndex::S as u8,      s2, 0x00, 0x00, LedMode::On as u8,
            KeyIndex::D as u8,      s3, 0x00, 0x00, LedMode::On as u8,
            KeyIndex::F as u8,      s4, 0x00, 0x00, LedMode::On as u8,
            // Query host list
            KeyIndex::LCtrl as u8,  0xff, 0xff, 0xff, LedMode::On as u8,
            KeyIndex::Equal as u8,  0x00, 0xff, 0x00, LedMode::On as u8,
            KeyIndex::BSpace as u8, 0x00, 0x00, 0xff, LedMode::On as u8,
            KeyIndex::B as u8,      0x00, 0xff, 0x00, LedMode::Flash as u8,
            KeyIndex::Minus as u8,  0xff, 0x00, 0x00, LedMode::On as u8,
            KeyIndex::N0 as u8,  mode_color.0, mode_color.1, mode_color.2, LedMode::On as u8,
        ];

        self.set_keys(payload)
    }

    pub fn bluetooth_pin_mode(&mut self) -> nb::Result<(), !> {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let payload = &[0xca,
                        11, // the number of keys in this request
            KeyIndex::N1 as u8, 0x00, 0xff, 0x00, LedMode::On as u8,
            KeyIndex::N2 as u8, 0x00, 0xff, 0x00, LedMode::On as u8,
            KeyIndex::N3 as u8, 0x00, 0xff, 0x00, LedMode::On as u8,
            KeyIndex::N4 as u8, 0x00, 0xff, 0x00, LedMode::On as u8,
            KeyIndex::N5 as u8, 0x00, 0xff, 0x00, LedMode::On as u8,
            KeyIndex::N6 as u8, 0x00, 0xff, 0x00, LedMode::On as u8,
            KeyIndex::N7 as u8, 0x00, 0xff, 0x00, LedMode::On as u8,
            KeyIndex::N8 as u8, 0x00, 0xff, 0x00, LedMode::On as u8,
            KeyIndex::N9 as u8, 0x00, 0xff, 0x00, LedMode::On as u8,
            KeyIndex::N0 as u8, 0x00, 0xff, 0x00, LedMode::On as u8,
            KeyIndex::Enter as u8, 0x00, 0x00, 0xff, LedMode::On as u8,
        ];

        self.set_keys(payload)
    }

    pub fn handle_message(&mut self, message: &Message) {
        match message.msg_type {
            MsgType::Led => {
                match LedOp::from(message.operation) {
                    LedOp::AckThemeMode => {
                        // data: [theme id]
                        //debug!("Led AckThemeMode {:?}", message.data).ok();
                    }
                    LedOp::AckConfigCmd => {
                        // data: [theme id, brightness, animation speed]
                        //debug!("Led AckConfigCmd {:?}", message.data).ok();
                    }
                    LedOp::AckSetIndividualKeys => {
                        // data: [202]
                    }
                    _ => {
                        debug!(
                            "lmsg: {:?} {} {:?}",
                            message.msg_type, message.operation, message.data
                        )
                        .ok();
                    }
                }
            }
            _ => {
                debug!(
                    "lmsg: {:?} {} {:?}",
                    message.msg_type, message.operation, message.data
                )
                .ok();
            }
        }
    }

    pub fn poll(&mut self) {
        let result = self
            .rx_transfer
            .as_mut()
            .unwrap()
            .poll(&mut self.serial.usart);
        match result {
            Err(nb::Error::WouldBlock) => {}
            Err(_) => unreachable!(),
            Ok(()) => {
                let buffer = self.rx_transfer.take().unwrap().finish();

                {
                    let buffer: &mut [u8] = buffer;
                    let message = Message {
                        msg_type: MsgType::from(buffer[0]),
                        operation: buffer[2],
                        data: &buffer[3..3 + buffer[1] as usize - 1],
                    };
                    self.handle_message(&message);
                }

                self.rx_transfer = Some(self.serial.receive(buffer));
            }
        }
    }
}

pub fn rx(_t: &mut Threshold, mut r: super::DMA1_CHANNEL3::Resources) {
    r.LED.poll();
}

pub fn tx(_t: &mut Threshold, mut r: super::DMA1_CHANNEL2::Resources) {
    r.LED.serial.tx_interrupt();
}
