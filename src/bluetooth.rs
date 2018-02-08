#![feature(const_fn)]

use core::fmt::Write;
use cortex_m_semihosting::hio;
use rtfm::Threshold;
use stm32l151::DMA1;
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

    pub fn send_report(&mut self, report: &HidReport, dma1: &mut DMA1, stdout: &mut Option<hio::HStdout>) {
        self.serial.send(MsgType::Keyboard, KeyboardOp::KeyReport as u8,
                         report.as_bytes(), dma1, stdout);
    }

    pub fn receive(message: &Message, stdout: &mut Option<hio::HStdout>) {
        match message.msg_type {
            //(2, 1) => {
                // SYSTEM Get ID
                //let data = &[4, 1, 0, 1, 2, 3, 4][..];
                //self.send(MsgType::System, 129, &data, dma, stdout, gpioa);
                //}
            MsgType::Ble => {
                match BleOp::from(message.operation)  {
                    BleOp::AckHostListQuery => {
                        //debug!(stdout, "bt host list: {:?}", message.data).ok();
                    }
                    _ => {
                        debug!(stdout, "msg: {:?} {} {:?}", message.msg_type, message.operation, message.data).ok();
                    }
                }
            },
            _ => {
                debug!(stdout, "msg: {:?} {} {:?}", message.msg_type, message.operation, message.data).ok();
            }
        }
    }
}

pub fn rx(_t: &mut Threshold, mut r: super::DMA1_CHANNEL6::Resources) {
    let stdout = &mut r.STDOUT;
    let callback = |msg: &Message| Bluetooth::receive(msg, stdout);
    r.BLUETOOTH.serial.receive(&mut r.DMA1, callback);
}

pub fn tx(_t: &mut Threshold, mut r: super::DMA1_CHANNEL7::Resources) {
    r.BLUETOOTH.serial.tx_interrupt(&mut r.DMA1);
}
