use bare_metal::Peripheral;
use core::ops::Deref;
use vcell::VolatileCell;

// TODO: make this take-able? or at least move into the main usb part
pub const PMA: Peripheral<PMA> = unsafe { Peripheral::new(0x4000_6000) };
//const BTABLE: usize = 0;

pub struct PMA {
    pub pma_area: PMA_Area,
}

impl PMA {
    pub fn zero(&mut self) {
        for i in 0..256 {
            self.pma_area.set_u16(i * 2, 0);
        }
    }
}

impl Deref for PMA {
    type Target = PMA_Area;
    fn deref(&self) -> &PMA_Area {
        &self.pma_area
    }
}

#[repr(C)]
pub struct PMA_Area {
    // The PMA consists of 256 u16 words separated by u16 gaps, so lets
    // represent that as 512 u16 words which we'll only use every other of.
    words: [VolatileCell<u16>; 512],
}

impl PMA_Area {
    // TODO: use operator overloading and just impl [] access, without double-counting
    pub fn get_u16(&self, offset: usize) -> u16 {
        assert!((offset & 0x01) == 0);
        self.words[offset].get()
    }

    pub fn set_u16(&self, offset: usize, val: u16) {
        assert!((offset & 0x01) == 0);
        self.words[offset].set(val);
    }

    pub fn write_buffer_u8(&self, base: usize, buf: &[u8]) {
        let mut last: u16 = 0;
        let mut off: usize = 0;

        for (ofs, v) in buf.iter().enumerate() {
            off = ofs;
            if ofs & 1 == 0 {
                last = u16::from(*v);
            } else {
                self.set_u16((base + ofs) & !1, last | (u16::from(*v) << 8));
            }
        }

        if off & 1 == 0 {
            //self.set_u16(base + (off * 2), last);
            self.set_u16(base + off, last);
        }
    }
}
