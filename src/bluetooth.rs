#![feature(const_fn)]

use core::fmt::Write;
use cortex_m_semihosting::hio;
use rtfm::Threshold;
use stm32l151::{DMA1, GPIOA, RCC};
use stm32l151;
use super::Keyboard;

pub struct Bluetooth {
    usart: stm32l151::USART2,
}

static mut SEND_BUFFER: [u8; 0x10] = [0; 0x10];

impl Bluetooth {
    pub fn new(usart: stm32l151::USART2, dma: &DMA1, gpioa: &mut GPIOA, rcc: &mut RCC) -> Bluetooth {
        let mut bt = Bluetooth {
            usart: usart,
        };
        bt.init(dma, gpioa, rcc);
        bt
    }

    fn init(&mut self, dma: &DMA1, gpioa: &mut GPIOA, rcc: &mut RCC) {
        gpioa.moder.modify(|_, w| unsafe { w.moder1().bits(1) });
        gpioa.pupdr.modify(|_, w| unsafe { w.pupdr1().bits(0b01) });
        gpioa.odr.modify(|_, w| w.odr1().clear_bit());

        gpioa.moder.modify(|_, w| unsafe { w.moder2().bits(0b10).moder3().bits(0b10) });
        gpioa.pupdr.modify(|_, w| unsafe { w.pupdr2().bits(0b01).pupdr3().bits(0b01) });
        gpioa.afrl.modify(|_, w| unsafe { w.afrl2().bits(7).afrl3().bits(7) });

        rcc.apb1enr.modify(|_, w| w.usart2en().set_bit());
        rcc.ahbenr.modify(|_, w| w.dma1en().set_bit());

        self.usart.brr.modify(|_, w| unsafe { w.bits(417) });
        self.usart.cr3.modify(|_, w| w.dmat().set_bit());
        self.usart.cr3.modify(|_, w| w.dmar().set_bit());
        self.usart.cr1.modify(|_, w| {
            w.rxneie().set_bit()
             .re().set_bit()
             .te().set_bit()
             .ue().set_bit()
        });

        dma.cpar6.write(|w| unsafe { w.pa().bits(0x4000_4404) });
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

    pub fn send_report(
        &mut self,
        keyboard: &Keyboard,
        dma1: &DMA1,
        stdout: &mut hio::HStdout,
        gpioa: &GPIOA,
    ) {
        let bits = dma1.cndtr7.read().ndt().bits();
        if bits == 0 {
            unsafe {
                SEND_BUFFER[0] = 0x7;
                SEND_BUFFER[1] = 0x9;
                SEND_BUFFER[2] = 0x1;
                SEND_BUFFER[3] = 0x0;
                SEND_BUFFER[4] = 0x0;
                SEND_BUFFER[5] = 0x0;
                SEND_BUFFER[6] = 0x0;
                SEND_BUFFER[7] = 0x0;
                SEND_BUFFER[8] = 0x0;

                if keyboard.state[0] {
                    //unsafe { usb::hid::HID_REPORT[2] = 0x5 };
                    SEND_BUFFER[5] = 0x4;
                } else if keyboard.state[1] {
                    SEND_BUFFER[5] = 0x5;
                } else if keyboard.state[2] {
                    SEND_BUFFER[5] = 0x6;
                } else if keyboard.state[3] {
                    SEND_BUFFER[5] = 0x7;
                } else if keyboard.state[4] {
                    SEND_BUFFER[5] = 0x8;
                } else {
                    let pressed = keyboard.state.into_iter().filter(|s| **s).count();
                    if pressed > 0 {
                        SEND_BUFFER[5] = 0x9;
                    }
                }
            }
            gpioa.odr.modify(|_, w| w.odr1().clear_bit());
            gpioa.odr.modify(|_, w| w.odr1().set_bit());
        } else {
            write!(stdout, "incomplete tx {}", bits).unwrap();
        }
    }

    pub fn receive(&mut self, dma: &mut stm32l151::DMA1, gpioa: &mut stm32l151::GPIOA) {
        // TODO: always just receive two via DMA?
        // and then from there on via length field
        if self.usart.sr.read().rxne().bit_is_set() {
            let bits = self.usart.dr.read().bits() as u8;

            if unsafe { STATE == 0 } && bits == 6 {
                unsafe { STATE = 1 }
            } else if unsafe { STATE == 1 } {
                unsafe {
                    RECEIVE_COUNT = bits as usize;
                    RECEIVE_COUNTER = 0;
                    STATE = 2;
                }
            } else if unsafe { STATE == 2 } {
                unsafe {
                    RECEIVE_BUFFER[RECEIVE_COUNTER] = bits;
                    RECEIVE_COUNTER += 1;
                }

                if unsafe { RECEIVE_COUNTER == RECEIVE_COUNT } {
                    unsafe {
                        gpioa.odr.modify(|_, w| w.odr1().clear_bit());
                        dma.cndtr7.modify(|_, w| w.ndt().bits(0xb));
                        dma.ccr7.modify(|_, w| w.en().set_bit());
                        STATE = 0;
                    }
                }
            }
        }
    }
}

static mut STATE: u8 = 0;
static mut RECEIVE_COUNT: usize = 0;
static mut RECEIVE_COUNTER: usize = 0;
static mut RECEIVE_BUFFER: [u8; 0x10] = [0; 0x10];

pub fn receive(_t: &mut Threshold, mut r: super::USART2::Resources) {
    r.BLUETOOTH.receive(&mut r.DMA1, &mut r.GPIOA)
}

pub fn tx_complete(_t: &mut Threshold, r: super::DMA1_CHANNEL7::Resources) {
    r.DMA1.ifcr.write(|w| w.cgif7().set_bit());
    r.DMA1.ccr7.modify(|_, w| w.en().clear_bit());
}
