use stm32l151::USB;
use super::pma::PMA;
use usb::usb_ext::UsbEpExt;

pub static mut HID_REPORT: [u8; 5] = [0x01, 0x00, 0x00, 0x00, 0x00];

pub fn usb_hid_ctr(usb: &mut USB, pma: &mut PMA) {
    if !usb.istr.read().dir().bit_is_set() {
        usb.istr.modify(|_, w| w.ctr().clear_bit());

        unsafe {
        pma.write_buffer_u8(0x100, &HID_REPORT);
        pma.pma_area.set_u16(10, HID_REPORT.len() as u16);
        }
        usb.usb_ep1r.toggle_0();
    } else {
        panic!()
    }
}
