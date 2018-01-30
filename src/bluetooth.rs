#![feature(const_fn)]

use core::fmt::Write;
use cortex_m_semihosting::hio;
use rtfm::Threshold;
use stm32l151::{DMA1, GPIOA};
use super::keymap::HidReport;
use super::protocol::{MsgType, KeyboardOperation};
use super::serial::{Message, Serial};


pub struct Bluetooth<'a> {
    pub serial: Serial<'a>,
}


impl<'a> Bluetooth<'a> {
    pub fn new(serial: Serial) -> Bluetooth {
        Bluetooth {
            serial: serial,
        }
    }

    pub fn send_report(&mut self, report: &HidReport, dma1: &DMA1, stdout: &mut Option<hio::HStdout>, gpioa: &GPIOA) {
        self.serial.send(MsgType::Keyboard, KeyboardOperation::KeyReport as u8,
                  &report.as_bytes(), &dma1, stdout, &gpioa);
    }

    pub fn receive(message: &Message, stdout: &mut Option<hio::HStdout>) {
        match (message.msg_type, message.operation) {
            //(2, 1) => {
                // SYSTEM Get ID
                //let data = &[4, 1, 0, 1, 2, 3, 4][..];
                //self.send(MsgType::System, 129, &data, dma, stdout, gpioa);
            //}
            _ => {
                debug!(stdout, "msg: {} {} {:?}", message.msg_type, message.operation, message.data).ok();
            }
        }
    }
}

pub fn rx(_t: &mut Threshold, mut r: super::DMA1_CHANNEL6::Resources) {
    let stdout = &mut r.STDOUT;
    let callback = |msg: &Message| Bluetooth::receive(msg, stdout);
    r.BLUETOOTH.serial.receive(&mut r.DMA1, &mut r.GPIOA, callback);
}

pub fn tx(_t: &mut Threshold, mut r: super::DMA1_CHANNEL7::Resources) {
    r.BLUETOOTH.serial.tx_interrupt(&mut r.DMA1);
}
