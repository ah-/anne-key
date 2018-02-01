use core::fmt::Write;
use cortex_m_semihosting::hio;
use rtfm::Threshold;
use super::protocol::Message;
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

    pub fn receive(message: &Message, stdout: &mut Option<hio::HStdout>) {
        match (message.msg_type, message.operation) {
            _ => {
                debug!(stdout, "lmsg: {} {} {:?}", message.msg_type, message.operation, message.data).ok();
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
