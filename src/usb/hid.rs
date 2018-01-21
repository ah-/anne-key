pub static mut HID_REPORT: [u8; 5] = [0x01, 0x00, 0x04, 0x00, 0x00];

pub fn usb_hid_ctr(mut r: &mut super::super::USB_LP::Resources) {
    if !r.USB.istr.read().dir().bit_is_set() {
        usb_clear_tx_ep1_ctr(&mut r);
        let pma = super::pma::PMA.get();
        unsafe {
            (*pma).write_buffer_u8(0x100, &HID_REPORT);
            (*pma).pma_area.set_u16(10, 5);
        }
        set_ep1_tx_status_valid_dtog(&mut r);
    //loop {}
    } else {
        usb_clear_rx_ep1_ctr(&mut r);
        loop {}
    }
}

fn usb_clear_tx_ep1_ctr(r: &mut super::super::USB_LP::Resources) {
    r.USB.usb_ep1r.write(|w| unsafe {
        w.bits(
            (r.USB.usb_ep1r.read().bits() & 0xFF7F) & super::USB_EPREG_MASK,
        )
    });
}

fn usb_clear_rx_ep1_ctr(r: &mut super::super::USB_LP::Resources) {
    r.USB.usb_ep1r.write(|w| unsafe {
        w.bits(
            (r.USB.usb_ep1r.read().bits() & 0x7FFF) & super::USB_EPREG_MASK,
        )
    });
}

/*
fn set_ep1_rx_status_valid_dtog(r: &mut super::super::USB_LP::Resources) {
    let mut bb = r.USB.usb_ep1r.read().bits();
    bb &= super::USB_EPRX_DTOGMASK;
    if (bb & 0x1000) == 0 {
        bb |= 0x1000
    } else {
        bb &= !0x1000
    }
    if (bb & 0x2000) == 0 {
        bb |= 0x2000
    } else {
        bb &= !0x2000
    }
    bb |= 0x1000;
    r.USB
        .usb_ep1r
        .write(|w| unsafe { w.bits(bb | super::USB_EP_CTR_RX | super::USB_EP_CTR_TX) });
}
*/

fn set_ep1_tx_status_valid_dtog(r: &mut super::super::USB_LP::Resources) {
    let mut bb = r.USB.usb_ep1r.read().bits();
    bb &= super::USB_EPTX_DTOGMASK;
    if (bb & 0x10) == 0 {
        bb |= 0x10
    } else {
        bb &= !0x10
    }
    if (bb & 0x20) == 0 {
        bb |= 0x20
    } else {
        bb &= !0x20
    }
    bb |= 0x1000;
    r.USB.usb_ep1r.write(|w| unsafe {
        w.bits(bb | super::USB_EP_CTR_RX | super::USB_EP_CTR_TX)
    });
}
