#![feature(const_fn)]

use core::fmt::Write;
use rtfm::Threshold;
use stm32l151::DMA1;
use super::Keyboard;


pub struct Bluetooth {
    send_buffer: [u8; 0x10],
}


impl Bluetooth {
    pub const fn new() -> Bluetooth {
        Bluetooth {
            send_buffer: [0; 0x10]
        }
    }

    pub fn init(&mut self, p: &super::init::Peripherals) {
        p.GPIOA.moder.modify(|_, w| unsafe {
            w.moder2().bits(0b10)
             .moder3().bits(0b10)
        });
        p.GPIOA.pupdr.modify(|_, w| unsafe {
            w.pupdr2().bits(0b01)
             .pupdr3().bits(0b01)
        });
        p.GPIOA.afrl.modify(|_, w| unsafe {
            w.afrl2().bits(7)
             .afrl3().bits(7)
        });

        p.RCC.apb1enr.modify(|_, w| w.usart2en().set_bit());
        p.RCC.ahbenr.modify(|_, w| w.dma1en().set_bit());

        p.USART2.brr.modify(|_, w| unsafe { w.bits(417) });
        p.USART2.cr3.modify(|_, w| w.dmat().set_bit());
        p.USART2.cr3.modify(|_, w| w.dmar().set_bit());
        p.USART2.cr1.modify(|_, w| {
            w.rxneie().set_bit()
             .re().set_bit()
             .te().set_bit()
             .ue().set_bit()
        });

        p.DMA1.cpar6.write(|w| unsafe { w.pa().bits(0x4000_4404) });
        p.DMA1.cpar7.write(|w| unsafe { w.pa().bits(0x4000_4404) });
        p.DMA1.cmar7.write(|w| unsafe {
            w.ma().bits(self.send_buffer.as_mut_ptr() as u32)
        });
        p.DMA1.cndtr7.modify(|_, w| unsafe { w.ndt().bits(0x8) });
        p.DMA1.ccr7.modify(|_, w| {
            unsafe {
                w.pl().bits(2);
            }
            w.minc().set_bit()
             .dir().set_bit()
             .tcie().set_bit()
             .en().clear_bit()
        });

    }

    pub fn send_report(&mut self, keyboard: &Keyboard, dma1: &DMA1) {
        self.send_buffer[0] = 0x7;
        self.send_buffer[1] = 0x9;
        self.send_buffer[2] = 0x1;
        self.send_buffer[3] = 0x0;
        self.send_buffer[4] = 0x0;
        self.send_buffer[5] = 0x0;
        self.send_buffer[6] = 0x0;
        self.send_buffer[7] = 0x0;
        self.send_buffer[8] = 0x0;

        if keyboard.state[0] {
            //unsafe { usb::hid::HID_REPORT[2] = 0x5 };
            self.send_buffer[5] = 0x4;
        } else if keyboard.state[1] {
            self.send_buffer[5] = 0x5;
        } else if keyboard.state[2] {
            self.send_buffer[5] = 0x6;
        } else if keyboard.state[3] {
            self.send_buffer[5] = 0x7;
        } else if keyboard.state[4] {
            self.send_buffer[5] = 0x8;
        }

        dma1.cndtr7.modify(|_, w| unsafe { w.ndt().bits(0xb) });
        dma1.ccr7.modify(|_, w| w.en().set_bit());
    }
}

/*
static mut N: usize = 0;
static mut D: [u8; 100] = [0; 100];

pub fn log_receive(bits: u8, r: super::USART2::Resources) {
    // f6 1 170 6 1 170 6 1 170 6 1 170 6 4 134 15 12 0 6 1 170 6 4 134 15 12 0 6 4 134 15 12 0 6 4 134 15 12 0 6 4 134 15 12 0 6 4 134 15 12 0 6 4 134 15 12 0 
    //  6 4 134 15 12 0 6 4 134 15 12 0 6 4 134 15 12 0 6 4 134 15 12 0 6 4 134 15 12 0 6 4 134 15 12 0 6 4 134 15 12 0 6
    unsafe {
        D[N] = bits;
        N += 1;
        if N == 20 {
            write!(r.STDOUT, "f").unwrap();
            for i in 0..N {
                write!(r.STDOUT, "{} ", D[i]).unwrap();
            }
            N = 0;
        }
    }
}
*/

static mut STATE: u8 = 0;
static mut RECEIVE_COUNT: usize = 0;
static mut RECEIVE_COUNTER: usize = 0;
static mut RECEIVE_BUFFER: [u8; 0x10] = [0; 0x10];

pub fn receive(_t: &mut Threshold, r: super::USART2::Resources) {
    if r.USART2.sr.read().rxne().bit_is_set() {
        let bits = r.USART2.dr.read().bits() as u8;

        //write!(r.STDOUT, "r{}", bits).unwrap();
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
                    write!(r.STDOUT, "recv").unwrap();
                    for i in 0..RECEIVE_COUNT {
                        write!(r.STDOUT, "{} ", RECEIVE_BUFFER[i]).unwrap();
                    }
                    STATE = 0;
                }
            }
        }

        //log_receive(bits, r);
    }
}

pub fn tx_complete(_t: &mut Threshold, r: super::DMA1_CHANNEL7::Resources) {
    r.DMA1.ifcr.write(|w| w.cgif7().set_bit());
    r.DMA1.ccr7.modify(|_, w| w.en().clear_bit());
}
