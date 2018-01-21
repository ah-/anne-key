pub mod descriptors;
pub mod log;
pub mod pma;
pub mod hid;

use core::cmp::min;
use core::fmt::Write;
use rtfm::Threshold;

use self::pma::PMA;

const MAX_PACKET_SIZE: u32 = 64;

//(USB_EP_CTR_RX|USB_EP_SETUP|USB_EP_T_FIELD|USB_EP_KIND|USB_EP_CTR_TX|USB_EPADDR_FIELD);
const USB_EPREG_MASK: u32 = (1 << 15 | 1 << 11 | 1 << 10 | 1 << 9 | 1 << 8 | 0xf);

const USB_EPTX_STAT: u32 = 0x30;
const USB_EPTX_DTOGMASK: u32 = (USB_EPTX_STAT | USB_EPREG_MASK);

const USB_EPRX_STAT: u32 = 0x3000;
const USB_EPRX_DTOGMASK: u32 = (USB_EPRX_STAT | USB_EPREG_MASK);

const USB_EP_CTR_RX: u32 = 0x8000;
const USB_EP_CTR_TX: u32 = 0x80000000;

// TODO: more from header
const USB_REQ_GET_STATUS: u8 = 0x00;
//const USB_REQ_CLEAR_FEATURE: u8 = 0x01;
//const USB_REQ_SET_FEATURE: u8 = 0x03;
const USB_REQ_SET_ADDRESS: u8 = 0x05;
const USB_REQ_GET_DESCRIPTOR: u8 = 0x06;
//const USB_REQ_SET_DESCRIPTOR: u8 = 0x07;
//const USB_REQ_GET_CONFIGURATION: u8 = 0x08;
const USB_REQ_SET_CONFIGURATION: u8 = 0x09;
//const USB_REQ_GET_INTERFACE: u8 = 0x0A;
//const USB_REQ_SET_INTERFACE: u8 = 0x0B;
//const USB_REQ_SYNCH_FRAME: u8 = 0x0C;

const USB_DESC_TYPE_DEVICE: u8 = 1;
const USB_DESC_TYPE_CONFIGURATION: u8 = 2;
const USB_DESC_TYPE_STRING: u8 = 3;
//const USB_DESC_TYPE_INTERFACE: u8 = 4;
//const USB_DESC_TYPE_ENDPOINT: u8 = 5;
const USB_DESC_TYPE_DEVICE_QUALIFIER: u8 = 6;
//const USB_DESC_TYPE_OTHER_SPEED_CONFIGURATION: u8 = 7;
//const USB_DESC_TYPE_BOS: u8 = 0x0F;
const USB_DESC_TYPE_HID_REPORT: u8 = 0x22;


pub struct Usb {}

impl Usb {
    pub const fn new() -> Usb {
        Usb {}
    }

    pub fn init(&self, p: &super::init::Peripherals) {
        unsafe { (*(PMA.get())).zero() };

        p.RCC.apb1enr.modify(|_, w| w.usben().set_bit());
        p.RCC.apb1rstr.modify(|_, w| w.usbrst().set_bit());
        p.RCC.apb1rstr.modify(|_, w| w.usbrst().clear_bit());

        p.USB.usb_cntr.modify(|_, w| w.pdwn().clear_bit());

        p.USB.usb_cntr.modify(|_, w| {
            w.ctrm().set_bit()
             .errm().set_bit()
             .pmaovrm().set_bit()
             //.wkupm().set_bit()
             //.suspm().set_bit()
             //.esofm().set_bit()
             //.sofm().set_bit()
             .resetm().set_bit()
        });

        p.USB.btable.reset();

        p.USB.usb_cntr.modify(|_, w| w.fres().clear_bit());

        p.USB.istr.reset();

        p.USB.daddr.modify(|_, w| w.ef().set_bit());

        p.SYSCFG.pmc.modify(|_, w| w.usb_pu().set_bit());
    }
}

pub fn usb_lp(_t: &mut Threshold, mut r: super::USB_LP::Resources) {
    if r.USB.istr.read().ctr().bit_is_set() {
        while r.USB.istr.read().ctr().bit_is_set() {
            let endpoint = r.USB.istr.read().ep_id().bits();
            match endpoint {
                0 => {
                    r.USB_LOG.save(r.USB, 4);
                    usb_ctr(&mut r);
                    r.USB_LOG.save(r.USB, 5);
                }
                1 => {
                    hid::usb_hid_ctr(&mut r);
                }
                _ => loop {},
            }
        }
    }

    if r.USB.istr.read().reset().bit_is_set() {
        usb_reset(&mut r);
    }

    /*
    } else {
        write!(r.STDOUT, "other").unwrap();
        write!(r.STDOUT, "\n{:x}\n", istr.bits()).unwrap();
        loop {}
    }
    */

    // TODO: clear other interrupt bits in ifs?
    //r.USB.istr.modify(|_, w|
    //w.sof().clear_bit()
    //.esof().clear_bit()
    //.susp().clear_bit()
    //);
}

static mut NRESET: usize = 0;

fn usb_reset(r: &mut super::USB_LP::Resources) {
    r.USB.istr.modify(|_, w| w.reset().clear_bit());

    let pma = PMA.get();
    unsafe {
        (*pma).pma_area.set_u16(0, 0x40);
        (*pma).pma_area.set_u16(2, 0x0);
        (*pma).pma_area.set_u16(4, 0x20);
        (*pma).pma_area.set_u16(
            6,
            (0x8000 | ((MAX_PACKET_SIZE / 32) - 1) << 10) as
                u16,
        );
        (*pma).pma_area.set_u16(8, 0x100);
        (*pma).pma_area.set_u16(10, 0x0);

        (*pma).write_buffer_u8(0x100, &hid::HID_REPORT);
        (*pma).pma_area.set_u16(10, 5);
    }

    r.USB.usb_ep0r.modify(|_, w| unsafe {
        w.ep_type().bits(0b01).stat_tx().bits(0b10).stat_rx().bits(
            0b11,
        )
    });

    r.USB.usb_ep1r.modify(|_, w| unsafe {
        w.ep_type()
            .bits(0b11)
            .stat_tx()
            .bits(0b11)
            .stat_rx()
            .bits(0b10)
            .ea()
            .bits(0b1)
    });

    r.USB.daddr.modify(|_, w| w.ef().set_bit());

    unsafe {
        r.USB_LOG.reset();
        if NRESET > 1 {
            write!(r.STDOUT, "r").unwrap();
        }
        NRESET += 1;
    }
}

fn usb_clear_tx_ep_ctr(r: &mut super::USB_LP::Resources) {
    r.USB.usb_ep0r.write(|w| unsafe {
        w.bits((r.USB.usb_ep0r.read().bits() & 0xFF7F) & USB_EPREG_MASK)
    });
}

fn usb_clear_rx_ep_ctr(r: &mut super::USB_LP::Resources) {
    r.USB.usb_ep0r.write(|w| unsafe {
        w.bits((r.USB.usb_ep0r.read().bits() & 0x7FFF) & USB_EPREG_MASK)
    });
}

static mut DADDR: u8 = 0;

fn set_ep_tx_status_valid(r: &mut super::USB_LP::Resources) {
    let mut bb = r.USB.usb_ep0r.read().bits();
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
    r.USB.usb_ep0r.write(|w| unsafe {
        w.bits(bb | USB_EP_CTR_RX | USB_EP_CTR_TX)
    });
}

fn set_ep_tx_status_valid_dtog(r: &mut super::USB_LP::Resources) {
    let mut bb = r.USB.usb_ep0r.read().bits();
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
    r.USB.usb_ep0r.write(|w| unsafe {
        w.bits(bb | USB_EP_CTR_RX | USB_EP_CTR_TX)
    });
}

fn set_ep_rx_status_valid(r: &mut super::USB_LP::Resources) {
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
    bb &= !0x1000;
    //bb |= 0x4000;
    r.USB.usb_ep0r.write(|w| unsafe {
        w.bits(bb | USB_EP_CTR_RX | USB_EP_CTR_TX)
    });
}

fn set_ep_rx_status_valid_dtog(r: &mut super::USB_LP::Resources) {
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
    bb |= 0x1000;
    r.USB.usb_ep0r.write(|w| unsafe {
        w.bits(bb | USB_EP_CTR_RX | USB_EP_CTR_TX)
    });
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

fn usb_ctr(mut r: &mut super::USB_LP::Resources) {
    if !r.USB.istr.read().dir().bit_is_set() {
        usb_clear_tx_ep_ctr(&mut r);
        unsafe {
            if DADDR != 0 {
                r.USB.daddr.modify(|_, w| w.add().bits(DADDR));
                DADDR = 0;
                set_ep_rx_status_valid(&mut r);
            } else {
                let pma = PMA.get();
                (*pma).pma_area.set_u16(6, 0);
                set_ep_rx_status_valid_dtog(&mut r);
            }
        }
    } else {
        usb_clear_rx_ep_ctr(&mut r);
        let pma = PMA.get();
        unsafe {
            let request16 = (*pma).pma_area.get_u16(0x20);
            let value = (*pma).pma_area.get_u16(0x22);
            //let index = (*pma).pma_area.get_u16(0x24);
            let length = (*pma).pma_area.get_u16(0x26);

            (*pma).pma_area.set_u16(
                6,
                (0x8000 | ((MAX_PACKET_SIZE / 32) - 1) << 10) as
                    u16,
            );

            let request = ((request16 & 0xff00) >> 8) as u8;
            let request_type = (request16 & 0xff) as u8;
            match (request_type, request) {
                (0, USB_REQ_SET_ADDRESS) => {
                    DADDR = value as u8;
                    set_ep_tx_status_valid(&mut r);
                }
                (0x80, USB_REQ_GET_DESCRIPTOR) => {
                    let descriptor_type = (value >> 8) as u8;
                    let descriptor_index = (value & 0xff) as u8;
                    match (descriptor_type, descriptor_index) {
                        (USB_DESC_TYPE_DEVICE, _) => {
                            (*pma).write_buffer_u8(0x40, &descriptors::DEV_DESC);
                            (*pma).pma_area.set_u16(
                                2,
                                min(
                                    length,
                                    descriptors::DEV_DESC.len() as u16,
                                ),
                            );
                            set_ep_tx_status_valid(&mut r);
                        }
                        (USB_DESC_TYPE_CONFIGURATION, _) => {
                            (*pma).write_buffer_u8(0x40, &descriptors::CONF_DESC);
                            (*pma).pma_area.set_u16(
                                2,
                                min(
                                    length,
                                    descriptors::CONF_DESC.len() as u16,
                                ),
                            );
                            set_ep_tx_status_valid_dtog(&mut r);
                        }
                        (USB_DESC_TYPE_STRING, _) => {
                            let string = match descriptor_index {
                                0 => &descriptors::LANG_STR[..],
                                1 => &descriptors::MANUFACTURER_STR[..],
                                2 => &descriptors::PRODUCT_STR[..],
                                3 => &descriptors::SERIAL_NUMBER_STR[..],
                                4 => &descriptors::CONF_STR[..],
                                _ => &descriptors::PRODUCT_STR[..],
                            };
                            (*pma).write_buffer_u8(0x40, string);
                            (*pma).pma_area.set_u16(2, min(length, string.len() as u16));
                            set_ep_tx_status_valid_dtog(&mut r);
                        }
                        (USB_DESC_TYPE_DEVICE_QUALIFIER, _) => {
                            (*pma).write_buffer_u8(0x40, &descriptors::DEVICE_QUALIFIER);
                            (*pma).pma_area.set_u16(
                                2,
                                min(
                                    length,
                                    descriptors::DEVICE_QUALIFIER.len() as u16,
                                ),
                            );
                            set_ep_tx_status_valid_dtog(&mut r);
                        }
                        _ => loop {},
                    }
                }
                (0x81, USB_REQ_GET_DESCRIPTOR) => {
                    let descriptor_type = (value >> 8) as u8;
                    let descriptor_index = (value & 0xff) as u8;
                    match (descriptor_type, descriptor_index) {
                        (USB_DESC_TYPE_HID_REPORT, _) => {
                            (*pma).write_buffer_u8(0x40, &descriptors::HID_REPORT_DESC);
                            (*pma).pma_area.set_u16(
                                2,
                                min(
                                    length,
                                    descriptors::HID_REPORT_DESC.len() as u16,
                                ),
                            );
                            set_ep_tx_status_valid_dtog(&mut r);
                        }
                        _ => loop {},
                    }
                }
                (0, USB_REQ_GET_STATUS) => {
                    (*pma).pma_area.set_u16(0x40, 0);
                    (*pma).pma_area.set_u16(2, 2);
                    set_ep_tx_status_valid_dtog(&mut r);
                }
                (0, USB_REQ_SET_CONFIGURATION) => {
                    // TODO: check value?
                    (*pma).pma_area.set_u16(2, 0);
                    set_ep_tx_status_valid_dtog(&mut r);
                }
                (0x21, 0xa) => {
                    // USBHID SET_IDLE
                    (*pma).pma_area.set_u16(2, 0);
                    set_ep_tx_status_valid_dtog(&mut r);
                }
                (33, 11) => {
                    // ???
                    (*pma).pma_area.set_u16(2, 0);
                    set_ep_tx_status_valid_dtog(&mut r);
                }
                _ => loop {},
            }
        }
    }
}
