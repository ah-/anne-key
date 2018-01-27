#![feature(const_fn)]

use core::fmt::Write;
use cortex_m_semihosting::hio;
use rtfm::Threshold;
use stm32l151::{DMA1, GPIOA, RCC, USART2};
use super::keymap::HidReport;
use super::protocol::{MsgType, KeyboardOperation};


pub enum ReceiveStage {
    Header,
    Body,
}

pub struct Bluetooth {
    usart: USART2,
    receive_stage: ReceiveStage,
}

pub struct Message<'a> {
    msg_type: u8,
    operation: u8,
    data: &'a[u8],
}

static mut SEND_BUFFER: [u8; 0x10] = [0; 0x10];
static mut RECEIVE_BUFFER: [u8; 0x10] = [0; 0x10];

impl Bluetooth {
    pub fn new(usart: USART2, dma: &DMA1, gpioa: &mut GPIOA, rcc: &mut RCC) -> Bluetooth {
        let mut bt = Bluetooth {
            usart: usart,
            receive_stage: ReceiveStage::Header,
        };
        bt.init(dma, gpioa, rcc);
        bt
    }

    pub fn send_report(
        &mut self,
        report: &HidReport,
        dma1: &DMA1,
        stdout: &mut hio::HStdout,
        gpioa: &GPIOA) {
        self.send(MsgType::Keyboard, KeyboardOperation::KeyReport as u8,
                  &report.bytes, &dma1, stdout, &gpioa);
    }

    pub fn receive(&mut self, dma: &mut DMA1, gpioa: &mut GPIOA, stdout: &mut hio::HStdout) {
        if dma.isr.read().tcif6().bit_is_set() {
            dma.ifcr.write(|w| w.cgif6().set_bit());

            match self.receive_stage {
                ReceiveStage::Header => {
                    self.receive_stage = ReceiveStage::Body;

                    // wakeup complete, reset pa1
                    gpioa.bsrr.write(|w| w.br1().set_bit());

                    dma.ccr6.modify(|_, w| { w.en().clear_bit() });
                    dma.cmar6.write(|w| unsafe { w.ma().bits(RECEIVE_BUFFER.as_mut_ptr() as u32 + 2) });
                    dma.cndtr6.modify(|_, w| unsafe { w.ndt().bits(RECEIVE_BUFFER[1] as u16) });
                    dma.ccr6.modify(|_, w| { w.en().set_bit() });
                }
                ReceiveStage::Body => {
                    self.receive_stage = ReceiveStage::Header;
                    
                    let message = Message {
                        msg_type: unsafe { RECEIVE_BUFFER[0] },
                        operation: unsafe { RECEIVE_BUFFER[2] },
                        data: unsafe { &RECEIVE_BUFFER[3..3 + RECEIVE_BUFFER[1] as usize - 1] },
                    };
                    self.handle_message(&message, dma, stdout);
                }
            }
        }
    }

    fn init(&mut self, dma: &DMA1, gpioa: &mut GPIOA, rcc: &mut RCC) {
        gpioa.moder.modify(|_, w| unsafe {
            w.moder1().bits(1)
             .moder2().bits(0b10)
             .moder3().bits(0b10)
        });
        gpioa.pupdr.modify(|_, w| unsafe {
            w.pupdr1().bits(0b01)
             .pupdr2().bits(0b01)
             .pupdr3().bits(0b01)
        });
        gpioa.afrl.modify(|_, w| unsafe { w.afrl2().bits(7).afrl3().bits(7) });
        gpioa.odr.modify(|_, w| w.odr1().clear_bit());

        rcc.apb1enr.modify(|_, w| w.usart2en().set_bit());
        rcc.ahbenr.modify(|_, w| w.dma1en().set_bit());

        self.usart.brr.modify(|_, w| unsafe { w.bits(417) });
        self.usart.cr3.modify(|_, w| w.dmat().set_bit()
                                      .dmar().set_bit());
        self.usart.cr1.modify(|_, w| {
            w.rxneie().set_bit()
             .re().set_bit()
             .te().set_bit()
             .ue().set_bit()
        });

        dma.cpar6.write(|w| unsafe { w.pa().bits(0x4000_4404) });
        dma.cmar6.write(|w| unsafe { w.ma().bits(RECEIVE_BUFFER.as_mut_ptr() as u32) });
        dma.ccr6.modify(|_, w| {
            unsafe {
                w.pl().bits(2);
            }
            w.minc().set_bit()
             .tcie().set_bit()
        });

        dma.cpar7.write(|w| unsafe { w.pa().bits(0x4000_4404) });
        dma.cmar7.write(|w| unsafe { w.ma().bits(SEND_BUFFER.as_mut_ptr() as u32) });
        dma.cndtr7.modify(|_, w| unsafe { w.ndt().bits(0x0) });
        dma.ccr7.modify(|_, w| {
            unsafe {
                w.pl().bits(2);
            }
            w.minc().set_bit()
             .dir().set_bit()
             .tcie().set_bit()
             .en().clear_bit()
        });
    }

    fn send(
        &mut self,
        message_type: MsgType,
        operation: u8, // TODO: make this typed
        data: &[u8],
        dma1: &DMA1,
        stdout: &mut hio::HStdout,
        gpioa: &GPIOA) {
        if dma1.cndtr7.read().ndt().bits() == 0 {
            unsafe {
                SEND_BUFFER[0] = message_type as u8;
                SEND_BUFFER[1] = data.len() as u8;
                SEND_BUFFER[2] = operation;
                SEND_BUFFER[3..3 + data.len()].clone_from_slice(data);
            }

            dma1.ccr6.modify(|_, w| { w.en().clear_bit() });
            dma1.cmar6.write(|w| unsafe { w.ma().bits(RECEIVE_BUFFER.as_mut_ptr() as u32) });
            dma1.cndtr6.modify(|_, w| unsafe { w.ndt().bits(2) });
            dma1.ccr6.modify(|_, w| { w.en().set_bit() });

            self.receive_stage = ReceiveStage::Header;

            gpioa.odr.modify(|_, w| w.odr1().clear_bit());
            gpioa.odr.modify(|_, w| w.odr1().set_bit());
        } else {
            // TODO: return an error instead
            // saying we're busy
            // using https://docs.rs/nb/0.1.1/nb/
            write!(stdout, "tx busy").unwrap();
        }
    }

    fn handle_message(&mut self, message: &Message, dma: &mut DMA1, stdout: &mut hio::HStdout) {
        match (message.msg_type, message.operation) {
            (6, 170) => {
                // Wakeup acknowledged, send data
                unsafe { dma.cndtr7.modify(|_, w| w.ndt().bits(0xb)) };
                dma.ccr7.modify(|_, w| w.en().set_bit());
            }
            _ => {
                write!(stdout, "msg: {} {}", message.msg_type, message.operation).unwrap();
            }
        }
    }
}

pub fn rx(_t: &mut Threshold, mut r: super::DMA1_CHANNEL6::Resources) {
    r.BLUETOOTH.receive(&mut r.DMA1, &mut r.GPIOA, &mut r.STDOUT)
}

pub fn tx(_t: &mut Threshold, r: super::DMA1_CHANNEL7::Resources) {
    r.DMA1.ifcr.write(|w| w.cgif7().set_bit());
    r.DMA1.ccr7.modify(|_, w| w.en().clear_bit());
}
