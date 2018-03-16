use stm32l151::USB;
use usb::usb_ext::UsbExt;

pub static mut HID_REPORT: [u8; 5] = [0x01, 0x00, 0x04, 0x00, 0x00];

pub fn usb_hid_ctr(usb: &mut USB) {
    if !usb.istr.read().dir().bit_is_set() {
        let pma = super::pma::PMA.get();
        unsafe {
            (*pma).write_buffer_u8(0x100, &HID_REPORT);
            (*pma).pma_area.set_u16(10, 5);
        }
        usb.set_ep1_tx_status_valid_dtog();
    } else {
        panic!()
    }
}
