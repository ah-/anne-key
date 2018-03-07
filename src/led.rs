use core::fmt::Write;
use cortex_m_semihosting::hio;
use embedded_hal::digital::OutputPin;
use rtfm::Threshold;
use hal::gpio::{Input, Output};
use hal::gpio::gpioc::PC15;
use nb;
use super::protocol::{Message, MsgType, LedOp};
use super::serial::{Serial, Transfer};
use super::serial::led_usart::LedUsart;
use super::keymatrix::{KeyState, to_packed_bits};


pub struct Led<'a> {
    pub serial: Serial<'a, LedUsart>,
    pub rx_transfer: Option<Transfer<[u8; 0x20]>>,
    pub pc15: PC15<Output>,
    pub state: bool
}

impl<'a> Led<'a> {
    pub fn new(mut serial: Serial<'a, LedUsart>, rx_buffer: &'static mut[u8; 0x20], pc15: PC15<Input>) -> Led<'a> {
        let rx_transfer = serial.receive(rx_buffer);
        Led {
            serial: serial,
            rx_transfer: Some(rx_transfer),
            pc15: pc15.into_output().pull_up(),
            state: true
        }
    }

    pub fn on(&mut self) -> nb::Result<(), !> {
        self.pc15.set_high();
        Ok(())
    }

    pub fn off(&mut self) -> nb::Result<(), !> {
        self.pc15.set_low();
        Ok(())
    }

    pub fn toggle(&mut self) -> nb::Result<(), !> {
        if(!self.state){
            self.pc15.set_high();
	} else {
	    self.pc15.set_low();
        }
	self.state = !self.state;
	Ok(())
    }

    // next_* cycles through themes/brightness/speed
    pub fn next_theme(&mut self) -> nb::Result<(), !> {
        self.serial.send(MsgType::Led, LedOp::ConfigCmd as u8, &[1, 0, 0])
    }

    pub fn next_brightness(&mut self) -> nb::Result<(), !> {
        self.serial.send(MsgType::Led, LedOp::ConfigCmd as u8, &[0, 0, 1])
    }

    pub fn next_animation_speed(&mut self) -> nb::Result<(), !> {
        self.serial.send(MsgType::Led, LedOp::ConfigCmd as u8, &[0, 1, 0])
    }

    pub fn set_theme(&mut self, theme: u8) -> nb::Result<(), !> {
        self.serial.send(MsgType::Led, LedOp::ThemeMode as u8, &[theme])
    }

    pub fn send_keys(&mut self, state: &KeyState) -> nb::Result<(), !> {
        let packed = to_packed_bits(state);
        self.serial.send(MsgType::Led, LedOp::Key as u8, &packed.bytes)
    }

    pub fn send_music(&mut self, keys: &[u8]) -> nb::Result<(), !> {
        self.serial.send(MsgType::Led, LedOp::Music as u8, keys)
    }

    pub fn get_theme_id(&mut self) -> nb::Result<(), !> {
        // responds with with [ThemeId]
        self.serial.send(MsgType::Led, LedOp::GetThemeId as u8, &[])
    }

    pub fn bluetooth_mode(&mut self) -> nb::Result<(), !> {
        let payload = [0xca, 0x0a,
            0x00, 0xff, 0xff, 0x00, 0x01, //key:Esc color:Y
            0x01, 0x00, 0xff, 0x00, 0x02, //key:1   color:G mode:flash
            0x02, 0xff, 0x00, 0x00, 0x01, //key:2   color:R
            0x03, 0xff, 0x00, 0x00, 0x01, //key:3   color:R
            0x04, 0xff, 0x00, 0x00, 0x01, //key:4   color:R
            0x0c, 0x00, 0xff, 0x00, 0x01, //key:+   color:G
            0x2f, 0x00, 0xff, 0x00, 0x02, //key:B   color:G mode:flash
            0x0b, 0xff, 0x00, 0x00, 0x01, //key:-   color:R
            0x0a, 0x00, 0xff, 0x00, 0x01, //key:0   color:G
            0x1d, 0x00, 0xff, 0x00, 0x01, //key:A   color:G
        ];

        self.serial.send(MsgType::Led, LedOp::SetIndividualKeys as u8, &payload)
    }

    pub fn handle_message(&mut self, message: &Message) {
        match message.msg_type {
            MsgType::Led => {
                match LedOp::from(message.operation) {
                    LedOp::AckThemeMode => {
                        // data: [theme id]
                        //debug!("Led AckThemeMode {:?}", message.data).ok();
                    },
                    LedOp::AckConfigCmd => {
                        // data: [theme id, brightness, animation speed]
                        //debug!("Led AckConfigCmd {:?}", message.data).ok();
                    },
                    _ => {
                        debug!("lmsg: {:?} {} {:?}", message.msg_type, message.operation, message.data).ok();
                    }
                }
            },
            _ => {
                debug!("lmsg: {:?} {} {:?}", message.msg_type, message.operation, message.data).ok();
            }
        }
    }

    pub fn poll(&mut self) {
        let result = self.rx_transfer.as_mut().unwrap().poll(&mut self.serial.usart);
        match result {
            Err(nb::Error::WouldBlock) => {},
            Err(_) => panic!("led rx error"),
            Ok(()) => {
                let buffer = self.rx_transfer.take().unwrap().finish();

                {
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
