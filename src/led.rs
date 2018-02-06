use core::fmt::Write;
use cortex_m_semihosting::hio;
use rtfm::Threshold;
use stm32l151::{DMA1, GPIOA};
use super::protocol::{Message, MsgType, LedOp};
use super::serial::Serial;
use super::serial::led_usart::LedUsart;
use keyboard::KeyState;


pub struct Led<'a> {
    pub serial: Serial<'a, LedUsart>,
}

impl<'a> Led<'a> {
    pub fn new(serial: Serial<'a, LedUsart>) -> Led {
        Led {
            serial: serial,
        }
    }

    pub fn send_something(&mut self, dma1: &mut DMA1, stdout: &mut Option<hio::HStdout>, gpioa: &mut GPIOA, state: &KeyState) {
        if state[0] {
            // returns AckThemeMode []
            self.serial.send(MsgType::Led, LedOp::ThemeMode as u8,
                             &[], dma1, stdout, gpioa);
        }
        if state[1] {
            // returns AckConfigCmd with [ThemeId]
            self.serial.send(MsgType::Led, LedOp::GetThemeId as u8,
                             &[0], dma1, stdout, gpioa);
        }
        if state[2] {
            self.serial.send(MsgType::Led, LedOp::ConfigCmd as u8,
                             &[1, 0, 0, 0], dma1, stdout, gpioa);
        }
        if state[3] {
            self.serial.send(MsgType::Led, LedOp::ConfigCmd as u8,
                             &[0, 1, 0, 0], dma1, stdout, gpioa);
        }
        if state[4] {
            self.serial.send(MsgType::Led, LedOp::ConfigCmd as u8,
                             &[0, 0, 1, 0], dma1, stdout, gpioa);
        }
        if state[5] {
            self.serial.send(MsgType::Led, LedOp::ConfigCmd as u8,
                             &[0, 0, 0, 1], dma1, stdout, gpioa);
        }
        if state[15] {
            self.serial.send(MsgType::Led, LedOp::ThemeMode as u8,
                             &[0], dma1, stdout, gpioa);
        }
        if state[16] {
            self.serial.send(MsgType::Led, LedOp::ThemeMode as u8,
                             &[1], dma1, stdout, gpioa);
        }
        if state[17] {
            self.serial.send(MsgType::Led, LedOp::ThemeMode as u8,
                             &[2], dma1, stdout, gpioa);
        }
        if state[18] {
            self.serial.send(MsgType::Led, LedOp::ThemeMode as u8,
                             &[3], dma1, stdout, gpioa);
        }
        if state[19] {
            self.serial.send(MsgType::Led, LedOp::ThemeMode as u8,
                             &[14], dma1, stdout, gpioa);
        }
        if state[20] {
            self.serial.send(MsgType::Led, LedOp::ThemeMode as u8,
                             &[18], dma1, stdout, gpioa);
        }
        if state[21] {
            self.serial.send(MsgType::Led, LedOp::ThemeMode as u8,
                             &[17], dma1, stdout, gpioa);
        }
        if state[22] {
            // sends O
            self.serial.send(MsgType::Led, LedOp::Key as u8,
                             &[0,0,0,1,0,0,0,0,0], dma1, stdout, gpioa);
        }
    }

    pub fn receive(message: &Message, stdout: &mut Option<hio::HStdout>) {
        match message.msg_type {
            MsgType::Led => {
                match LedOp::from(message.operation) {
                    LedOp::AckThemeMode => {
                        debug!(stdout, "Led AckThemeMode {:?}", message.data).ok();
                    },
                    LedOp::AckConfigCmd => {
                        // theme id, brightness, ???
                        debug!(stdout, "Led AckConfigCmd {:?}", message.data).ok();
                    },
                    _ => {
                        debug!(stdout, "lmsg: {:?} {} {:?}", message.msg_type, message.operation, message.data).ok();
                    }
                }
            },
            _ => {
                debug!(stdout, "lmsg: {:?} {} {:?}", message.msg_type, message.operation, message.data).ok();
            }
        }
    }
}

pub fn rx(_t: &mut Threshold, mut r: super::DMA1_CHANNEL3::Resources) {
    let stdout: &mut Option<hio::HStdout> = &mut r.STDOUT;
    let callback = |msg: &Message| Led::receive(msg, stdout);
    r.LED.serial.receive(&mut r.DMA1, &mut r.GPIOA, callback);
}

pub fn tx(_t: &mut Threshold, mut r: super::DMA1_CHANNEL2::Resources) {
    let stdout: &mut Option<hio::HStdout> = &mut r.STDOUT;
    r.LED.serial.tx_interrupt(&mut r.DMA1);
}
