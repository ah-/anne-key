use crate::bluetooth::BluetoothMode;
use crate::keycodes::KeyIndex;
use crate::keymatrix::KeyState;
use crate::protocol::{LedOp, Message, MsgType};
use crate::serial::led_usart::LedUsart;
use crate::serial::{Serial, Transfer};

use core::convert::Infallible;
use core::marker::Unsize;
use embedded_hal::digital::v2::OutputPin;
use hal::gpio::gpioc::PC15;
use hal::gpio::{Input, Output};
use stm32l1::stm32l151::SYST;

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

    pub fn on(&mut self) -> nb::Result<(), Infallible> {
        self.pc15.set_high().unwrap();
        Ok(())
    }

    pub fn off(&mut self) -> nb::Result<(), Infallible> {
        self.pc15.set_low().unwrap();
        self.state = false;
        Ok(())
    }

    pub fn poke(&mut self, syst: &SYST) -> nb::Result<(), Infallible> {
        self.off()?;

        // TODO: introduce proper delay()
        let wait_until_tick = 0;
        while syst.cvr.read() > wait_until_tick {}

        self.on()?;

        while syst.cvr.read() > wait_until_tick {}

        Ok(())
    }

    pub fn toggle(&mut self) -> nb::Result<(), Infallible> {
        self.state = !self.state;
        if self.state {
            self.theme_mode()
        } else {
            self.set_theme(15)
        }
    }

    // next_* cycles through themes/brightness/speed
    pub fn next_theme(&mut self) -> nb::Result<(), Infallible> {
        self.serial
            .send(MsgType::Led, LedOp::ConfigCmd as u8, &[1, 0, 0])
    }

    pub fn next_brightness(&mut self) -> nb::Result<(), Infallible> {
        self.serial
            .send(MsgType::Led, LedOp::ConfigCmd as u8, &[0, 0, 1])
    }

    pub fn next_animation_speed(&mut self) -> nb::Result<(), Infallible> {
        self.serial
            .send(MsgType::Led, LedOp::ConfigCmd as u8, &[0, 1, 0])
    }

    pub fn set_theme(&mut self, theme: u8) -> nb::Result<(), Infallible> {
        self.serial
            .send(MsgType::Led, LedOp::ThemeMode as u8, &[theme])
    }

    pub fn send_keys(&mut self, state: &KeyState) -> nb::Result<(), Infallible> {
        self.serial.send(MsgType::Led, LedOp::Key as u8, state)
    }

    pub fn send_music(&mut self, keys: &[u8]) -> nb::Result<(), Infallible> {
        self.serial.send(MsgType::Led, LedOp::Music as u8, keys)
    }

    pub fn get_theme_id(&mut self) -> nb::Result<(), Infallible> {
        // responds with with [ThemeId]
        self.serial.send(MsgType::Led, LedOp::GetThemeId as u8, &[])
    }

    pub fn set_keys(&mut self, payload: &[u8]) -> nb::Result<(), Infallible> {
        self.serial
            .send(MsgType::Led, LedOp::SetIndividualKeys as u8, payload)
    }

    pub fn theme_mode(&mut self) -> nb::Result<(), Infallible> {
        self.state = true;
        self.serial.send(MsgType::Led, LedOp::ThemeMode as u8, &[])
    }

    pub fn bluetooth_mode(
        &mut self,
        saved_hosts: u8,
        connected_host: u8,
        mode: BluetoothMode,
        keyboard_send_usb_report: bool,
    ) -> nb::Result<(), Infallible> {
        let mode_color = match mode {
            BluetoothMode::Unknown => (0xff, 0, 0),
            BluetoothMode::Ble => (0, 0xff, 0),
            BluetoothMode::Legacy => (0xff, 0xff, 0),
        };

        let usb_mode = if keyboard_send_usb_report {
            LedMode::On
        } else {
            LedMode::Flash
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

        #[rustfmt::skip]
        let payload = &[0xca,
                        20, // the number of keys in this request
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
            KeyIndex::N5 as u8, 0xff, 0xff, 0xff, usb_mode as u8,
        ];

        self.set_keys(payload)
    }

    pub fn bluetooth_pin_mode(&mut self) -> nb::Result<(), Infallible> {
        #[rustfmt::skip]
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

    pub fn handle_message(&mut self, message: &Message<'_>) {
        match message.msg_type {
            MsgType::Led => {
                match LedOp::from(message.operation) {
                    LedOp::AckThemeMode => {
                        // data: [theme id]
                        //crate::heprintln!("Led AckThemeMode {:?}", message.data).ok();
                    }
                    LedOp::AckConfigCmd => {
                        // data: [theme id, brightness, animation speed]
                        //crate::heprintln!("Led AckConfigCmd {:?}", message.data).ok();
                    }
                    LedOp::AckSetIndividualKeys => {
                        // data: [202]
                    }
                    _ => {
                        crate::heprintln!(
                            "lmsg: {:?} {} {:?}",
                            message.msg_type,
                            message.operation,
                            message.data
                        )
                        .ok();
                    }
                }
            }
            _ => {
                crate::heprintln!(
                    "lmsg: {:?} {} {:?}",
                    message.msg_type,
                    message.operation,
                    message.data
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
