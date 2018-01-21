#![feature(const_fn)]

extern crate stm32l151;

use super::pma::PMA;

pub struct Log {
    p: usize,
    d: [u32; 100],
    i: [u32; 100],
    r: [u32; 100],
    v: [u32; 100],
    rxc: [u16; 100],
    rxv: [u16; 100],
    rxv2: [u16; 100],
    rxv3: [u16; 100],
    rxv4: [u16; 100],
    txc: [u16; 100],
    txv: [u16; 100],
    addr: [u16; 100],
}

impl Log {
    pub const fn new() -> Log {
        Log {
            p: 0,
            d: [0; 100],
            i: [0; 100],
            r: [0; 100],
            v: [0; 100],
            rxc: [0; 100],
            rxv: [0; 100],
            rxv2: [0; 100],
            rxv3: [0; 100],
            rxv4: [0; 100],
            txc: [0; 100],
            txv: [0; 100],
            addr: [0; 100],
        }
    }

    pub fn reset(&mut self) {
        self.p = 0;
    }

    pub fn save(&mut self, usb: &mut stm32l151::USB, d: u32) {
        unsafe {
            if self.p < 100 {
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
