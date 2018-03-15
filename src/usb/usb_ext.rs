use stm32l151::USB;

pub trait UsbExt {
    fn toggle_ep0_out(&self);
    fn toggle_ep0_0(&self);
    fn toggle_ep0(&self, mask: u32, val: u32, flags: u32);

    fn clear_tx_ep_ctr(&self);
    fn clear_rx_ep_ctr(&self);

    fn clear_tx_ep1_ctr(&self);
    fn clear_rx_ep1_ctr(&self);

    fn set_ep_rx_status_valid(&self);
    fn set_ep_rx_status_valid_dtog(&self);

    fn set_ep1_tx_status_valid_dtog(&self);
}

//(USB_EP_CTR_RX|USB_EP_SETUP|USB_EP_T_FIELD|USB_EP_KIND|USB_EP_CTR_TX|USB_EPADDR_FIELD);
const USB_EPREG_MASK: u32 = (1 << 15 | 1 << 11 | 1 << 10 | 1 << 9 | 1 << 8 | 0xf);

const USB_EPTX_STAT: u32 = 0x30;
const USB_EPTX_DTOGMASK: u32 = (USB_EPTX_STAT | USB_EPREG_MASK);

const USB_EPRX_STAT: u32 = 0x3000;
const USB_EPRX_DTOGMASK: u32 = (USB_EPRX_STAT | USB_EPREG_MASK);

const USB_EP_CTR_RX: u32 = 0x8000;
const USB_EP_CTR_TX: u32 = 0x8000_0000;

const EP_MASK: u32 = 0x0F0F;
const EP_TX_MASK: u32 = 0x0030;
const EP_RX_MASK: u32 = 0x3000;
const EP_TX_RX_MASK: u32 = (EP_TX_MASK | EP_RX_MASK);
// --> EP_TX_RX_MASK = 0x3F3F

const EP_TX_VALID: u32 = 0x0030;
const EP_RX_VALID: u32 = 0x3000;
const EP_TX_RX_VALID: u32 = (EP_TX_VALID | EP_RX_VALID);
// --> EP_TX_RX_VALID = 0x3030

const EP_STATUS_OUT: u32 = 0x0100;

impl UsbExt for USB {
    // TODO: pass explicit endpoint nr?
    fn clear_tx_ep_ctr(&self) {
        // TODO: modify? with |r, w| instead of read()?
        // everywhere here really
        self.usb_ep0r
            .write(|w| unsafe { w.bits((self.usb_ep0r.read().bits() & 0xFF7F) & USB_EPREG_MASK) });
    }

    fn clear_rx_ep_ctr(&self) {
        self.usb_ep0r
            .write(|w| unsafe { w.bits((self.usb_ep0r.read().bits() & 0x7FFF) & USB_EPREG_MASK) });
    }

    fn clear_tx_ep1_ctr(&self) {
        self.usb_ep1r
            .write(|w| unsafe { w.bits((self.usb_ep1r.read().bits() & 0xFF7F) & USB_EPREG_MASK) });
    }

    fn clear_rx_ep1_ctr(&self) {
        self.usb_ep1r
            .write(|w| unsafe { w.bits((self.usb_ep1r.read().bits() & 0x7FFF) & USB_EPREG_MASK) });
    }

    fn toggle_ep0_out(&self) {
        self.toggle_ep0(EP_TX_RX_MASK, EP_TX_RX_VALID, EP_STATUS_OUT)
    }

    fn toggle_ep0_0(&self) {
        self.toggle_ep0(EP_TX_RX_MASK, EP_TX_RX_VALID, 0)
    }

    fn toggle_ep0(&self, mask: u32, val: u32, flags: u32) {
        self.usb_ep0r
            .modify(|r, w| unsafe { w.bits(((r.bits() & (EP_MASK | mask)) ^ val) | flags) })
    }

    fn set_ep_rx_status_valid(&self) {
        let mut bb = self.usb_ep0r.read().bits();
        bb &= USB_EPRX_DTOGMASK;
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
        bb &= !0x1000;
        //bb |= 0x4000;
        self.usb_ep0r
            .write(|w| unsafe { w.bits(bb | USB_EP_CTR_RX | USB_EP_CTR_TX) });
    }

    fn set_ep_rx_status_valid_dtog(&self) {
        let mut bb = self.usb_ep0r.read().bits();
        bb &= USB_EPRX_DTOGMASK;
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
        self.usb_ep0r
            .write(|w| unsafe { w.bits(bb | USB_EP_CTR_RX | USB_EP_CTR_TX) });
    }

    fn set_ep1_tx_status_valid_dtog(&self) {
        let mut bb = self.usb_ep1r.read().bits();
        bb &= USB_EPTX_DTOGMASK;
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
        self.usb_ep1r
            .write(|w| unsafe { w.bits(bb | USB_EP_CTR_RX | USB_EP_CTR_TX) });
    }
}

/*
fn ep_rx_toggle_dtog(r: &mut super::USB_LP::Resources) {
    let mut bb = r.USB.usb_ep0r.read().bits();
    bb &= USB_EPRX_DTOGMASK;
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
    r.USB
        .usb_ep0r
        .write(|w| unsafe { w.bits(bb | USB_EP_CTR_RX | USB_EP_CTR_TX) });
}
*/

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
