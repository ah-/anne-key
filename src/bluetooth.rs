#![feature(const_fn)]

use core::fmt::Write;
use cortex_m_semihosting::hio;
use rtfm::Threshold;
use super::hidreport::HidReport;
use super::protocol::{Message, MsgType, BleOp, KeyboardOp};
use super::serial::Serial;
use super::serial::bluetooth_usart::BluetoothUsart;


pub struct Bluetooth<'a> {
    pub serial: Serial<'a, BluetoothUsart>,
}


impl<'a> Bluetooth<'a> {
    pub fn new(serial: Serial<'a, BluetoothUsart>) -> Bluetooth {
        Bluetooth {
            serial: serial,
        }
    }

    pub fn send_report(&mut self, report: &HidReport) {
        self.serial.send(MsgType::Keyboard,
                         KeyboardOp::KeyReport as u8,
                         report.as_bytes());
    }

    pub fn receive(message: &Message) {
        match message.msg_type {
            //(2, 1) => {
                // SYSTEM Get ID
                //let data = &[4, 1, 0, 1, 2, 3, 4][..];
                //self.send(MsgType::System, 129, &data, gpioa);
                //}
            MsgType::Ble => {
                match BleOp::from(message.operation)  {
                    BleOp::AckHostListQuery => {
                        //debug!("bt host list: {:?}", message.data).ok();
                    }
                    _ => {
                        debug!("msg: {:?} {} {:?}", message.msg_type, message.operation, message.data).ok();
                    }
                }
            },
            _ => {
                debug!("msg: {:?} {} {:?}", message.msg_type, message.operation, message.data).ok();
            }
        }
    }
}

pub fn rx(_t: &mut Threshold, mut r: super::DMA1_CHANNEL6::Resources) {
    r.BLUETOOTH.serial.receive(Bluetooth::receive);
}

pub fn tx(_t: &mut Threshold, mut r: super::DMA1_CHANNEL7::Resources) {
    r.BLUETOOTH.serial.tx_interrupt();
}
