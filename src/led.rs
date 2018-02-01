use core::fmt::Write;
use cortex_m_semihosting::hio;
use rtfm::Threshold;
use stm32l151::{DMA1, GPIOA};
use super::protocol::{Message, MsgType, LedOp};
use super::serial::Serial;
use super::serial::led_usart::LedUsart;


pub struct Led<'a> {
    pub serial: Serial<'a, LedUsart>,
}

impl<'a> Led<'a> {
    pub fn new(serial: Serial<'a, LedUsart>) -> Led {
        Led {
            serial: serial,
        }
    }

    pub fn send_something(&mut self, dma1: &mut DMA1, stdout: &mut Option<hio::HStdout>, gpioa: &mut GPIOA) {
        self.serial.send(MsgType::LedStyle, LedOp::ThemeMode as u8,
                         &[], dma1, stdout, gpioa);
    }

    pub fn receive(message: &Message, stdout: &mut Option<hio::HStdout>) {
        match (message.msg_type, message.operation) {
            _ => {
                debug!(stdout, "lmsg: {:?} {} {:?}", message.msg_type, message.operation, message.data).ok();
            }
        }
    }
}

pub fn rx(_t: &mut Threshold, mut r: super::DMA1_CHANNEL3::Resources) {
    let stdout = &mut r.STDOUT;
    let callback = |msg: &Message| Led::receive(msg, stdout);
    r.LED.serial.receive(&mut r.DMA1, &mut r.GPIOA, callback);
}

pub fn tx(_t: &mut Threshold, mut r: super::DMA1_CHANNEL2::Resources) {
    r.LED.serial.tx_interrupt(&mut r.DMA1);
}
