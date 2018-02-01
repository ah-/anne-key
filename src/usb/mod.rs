pub mod descriptors;
pub mod log;
pub mod pma;
pub mod hid;
pub mod usb_ext;

use core::cmp::min;
use core::fmt::Write;
use cortex_m_semihosting::hio;
use rtfm::Threshold;

use stm32l151;

use self::usb_ext::UsbExt;
use self::pma::PMA;

const MAX_PACKET_SIZE: u32 = 64;

// TODO: more from header, make this an enum
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

// TODO: make this an enum
const USB_DESC_TYPE_DEVICE: u8 = 1;
const USB_DESC_TYPE_CONFIGURATION: u8 = 2;
const USB_DESC_TYPE_STRING: u8 = 3;
//const USB_DESC_TYPE_INTERFACE: u8 = 4;
//const USB_DESC_TYPE_ENDPOINT: u8 = 5;
const USB_DESC_TYPE_DEVICE_QUALIFIER: u8 = 6;
//const USB_DESC_TYPE_OTHER_SPEED_CONFIGURATION: u8 = 7;
//const USB_DESC_TYPE_BOS: u8 = 0x0F;
const USB_DESC_TYPE_HID_REPORT: u8 = 0x22;


pub struct Usb {
    usb: stm32l151::USB,
    log: self::log::Log,
    nreset: usize,
    pending_daddr: u8,
}

impl Usb {
    pub fn new(usb: stm32l151::USB, rcc: &mut stm32l151::RCC, syscfg: &mut stm32l151::SYSCFG) -> Usb {
        unsafe { (*(PMA.get())).zero() };

        rcc.apb1enr.modify(|_, w| w.usben().set_bit());
        rcc.apb1rstr.modify(|_, w| w.usbrst().set_bit());
        rcc.apb1rstr.modify(|_, w| w.usbrst().clear_bit());

        usb.usb_cntr.modify(|_, w| w.pdwn().clear_bit());
        usb.usb_cntr.modify(|_, w| {
            w.ctrm().set_bit()
             .errm().set_bit()
             .pmaovrm().set_bit()
             //.wkupm().set_bit()
             //.suspm().set_bit()
             //.esofm().set_bit()
             //.sofm().set_bit()
             .resetm().set_bit()
        });
        usb.btable.reset();
        usb.usb_cntr.modify(|_, w| w.fres().clear_bit());
        usb.istr.reset();
        usb.daddr.modify(|_, w| w.ef().set_bit());

        syscfg.pmc.modify(|_, w| w.usb_pu().set_bit());

        Usb {
            usb: usb,
            log: self::log::Log::new(),
            nreset: 0,
            pending_daddr: 0,
        }
    }

    pub fn interrupt(&mut self, stdout: &mut Option<hio::HStdout>) {
        if self.usb.istr.read().ctr().bit_is_set() {
            while self.usb.istr.read().ctr().bit_is_set() {
                let endpoint = self.usb.istr.read().ep_id().bits();
                match endpoint {
                    0 => {
                        self.log.save(&mut self.usb, 4);
                        self.ctr();
                        self.log.save(&mut self.usb, 5);
                    }
                    1 => {
                        // TODO
                        hid::usb_hid_ctr(&mut self.usb);
                    }
                    _ => panic!(),
                }
            }
        }

        if self.usb.istr.read().reset().bit_is_set() {
            self.reset(stdout);
        }

        /*
        } else {
            write!(r.STDOUT, "other").unwrap();
            write!(r.STDOUT, "\n{:x}\n", istr.bits()).unwrap();
            panic!()
        }
        */

        // TODO: clear other interrupt bits in ifs?
        //r.USB.istr.modify(|_, w|
        //w.sof().clear_bit()
        //.esof().clear_bit()
        //.susp().clear_bit()
        //);
    }

    fn reset(&mut self, stdout: &mut Option<hio::HStdout>) {
        self.usb.istr.modify(|_, w| w.reset().clear_bit());

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

        self.usb.usb_ep0r.modify(|_, w| unsafe {
            w.ep_type().bits(0b01).stat_tx().bits(0b10).stat_rx().bits(
                0b11,
            )
        });

        self.usb.usb_ep1r.modify(|_, w| unsafe {
            w.ep_type()
                .bits(0b11)
                .stat_tx()
                .bits(0b11)
                .stat_rx()
                .bits(0b10)
                .ea()
                .bits(0b1)
        });

        self.usb.daddr.modify(|_, w| w.ef().set_bit());

        self.log.reset();
        if self.nreset > 1 {
            debug!(stdout, "r").unwrap();
        }
        self.nreset += 1;
    }

    fn ctr(&mut self) {
        if !self.usb.istr.read().dir().bit_is_set() {
            self.usb.clear_tx_ep_ctr();
            unsafe {
                if self.pending_daddr != 0 {
                    self.usb.daddr.modify(|_, w| w.add().bits(self.pending_daddr));
                    self.pending_daddr = 0;
                    self.usb.set_ep_rx_status_valid();
                } else {
                    let pma = PMA.get();
                    (*pma).pma_area.set_u16(6, 0);
                    self.usb.set_ep_rx_status_valid_dtog();
                }
            }
        } else {
            self.usb.clear_rx_ep_ctr();
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
                        self.pending_daddr = value as u8;
                        self.usb.set_ep_tx_status_valid();
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
                                self.usb.set_ep_tx_status_valid();
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
                                self.usb.set_ep_tx_status_valid_dtog();
                            }
                            (USB_DESC_TYPE_STRING, _) => {
                                let string = match descriptor_index {
                                    0 => &descriptors::LANG_STR[..],
                                    1 => &descriptors::MANUFACTURER_STR[..],
                                    2 => &descriptors::PRODUCT_STR[..],
                                    3 => &descriptors::SERIAL_NUMBER_STR[..],
                                    4 => &descriptors::CONF_STR[..],
                                    _ => &descriptors::PRODUCT_STR[..],
                                    // last one should stall?
                                };
                                (*pma).write_buffer_u8(0x40, string);
                                (*pma).pma_area.set_u16(2, min(length, string.len() as u16));
                                self.usb.set_ep_tx_status_valid_dtog();
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
                                self.usb.set_ep_tx_status_valid_dtog();
                            }
                            _ => panic!(),
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
                                self.usb.set_ep_tx_status_valid_dtog();
                            }
                            _ => panic!(),
                        }
                    }
                    (0, USB_REQ_GET_STATUS) => {
                        (*pma).pma_area.set_u16(0x40, 0);
                        (*pma).pma_area.set_u16(2, 2);
                        self.usb.set_ep_tx_status_valid_dtog();
                    }
                    (0, USB_REQ_SET_CONFIGURATION) => {
                        // TODO: check value?
                        (*pma).pma_area.set_u16(2, 0);
                        self.usb.set_ep_tx_status_valid_dtog();
                    }
                    (0x21, 0xa) => {
                        // USBHID SET_IDLE
                        (*pma).pma_area.set_u16(2, 0);
                        self.usb.set_ep_tx_status_valid_dtog();
                    }
                    (33, 11) => {
                        // ???
                        (*pma).pma_area.set_u16(2, 0);
                        self.usb.set_ep_tx_status_valid_dtog();
                    }
                    _ => panic!(),
                }
            }
        }
    }
}

pub fn usb_lp(_t: &mut Threshold, mut r: super::USB_LP::Resources) {
    let stdout = &mut r.STDOUT;
    //r.USB.interrupt(stdout)
}
