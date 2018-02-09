#![feature(const_fn)]

extern crate stm32l151;

use super::pma::PMA;

const SIZE: usize = 80;

pub struct Log {
    p: usize,
    d: [u32; SIZE],
    i: [u32; SIZE],
    r: [u32; SIZE],
    v: [u32; SIZE],
    rxc: [u16; SIZE],
    rxv: [u16; SIZE],
    rxv2: [u16; SIZE],
    rxv3: [u16; SIZE],
    rxv4: [u16; SIZE],
    txc: [u16; SIZE],
    txv: [u16; SIZE],
    addr: [u16; SIZE],
}

impl Log {
    pub const fn new() -> Log {
        Log {
            p: 0,
            d: [0; SIZE],
            i: [0; SIZE],
            r: [0; SIZE],
            v: [0; SIZE],
            rxc: [0; SIZE],
            rxv: [0; SIZE],
            rxv2: [0; SIZE],
            rxv3: [0; SIZE],
            rxv4: [0; SIZE],
            txc: [0; SIZE],
            txv: [0; SIZE],
            addr: [0; SIZE],
        }
    }

    pub fn reset(&mut self) {
        self.p = 0;
    }

    pub fn save(&mut self, usb: &mut stm32l151::USB, d: u32) {
        unsafe {
            if self.p < SIZE {
                self.d[self.p] = d;
                self.v[self.p] = usb.usb_ep0r.read().bits();
                self.r[self.p] = usb.usb_cntr.read().bits();
                self.i[self.p] = usb.istr.read().bits();
                self.addr[self.p] = usb.daddr.read().bits() as u16;
                let pma = PMA.get();
                self.rxc[self.p] = (*pma).pma_area.get_u16(6);
                self.txc[self.p] = (*pma).pma_area.get_u16(2);
                self.rxv[self.p] = (*pma).pma_area.get_u16(0x20);
                self.rxv2[self.p] = (*pma).pma_area.get_u16(0x22);
                self.rxv3[self.p] = (*pma).pma_area.get_u16(0x24);
                self.rxv4[self.p] = (*pma).pma_area.get_u16(0x26);
                self.txv[self.p] = (*pma).pma_area.get_u16(0x40);
                self.p += 1;
            }
        }
    }
}
