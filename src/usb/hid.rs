use crate::usb::pma::PMA;
use crate::usb::usb_ext::UsbEpExt;
use stm32l1::stm32l151::USB;

pub struct UsbHid {
    pub report: [u8; 8],
    pub protocol: u8,
}

impl UsbHid {
    pub fn new() -> UsbHid {
        UsbHid {
            report: [0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            protocol: 0,
        }
    }

    pub fn ctr(&mut self, usb: &mut USB, pma: &mut PMA) {
        if !usb.istr.read().dir().bit_is_set() {
            pma.write_buffer_u8(0x100, &self.report);
            pma.pma_area.set_u16(10, self.report.len() as u16);
            usb.ep1r.toggle_tx_out();
        //TODO: stall?
        } else {
            panic!()
        }
    }
}
