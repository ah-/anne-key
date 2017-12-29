#![feature(proc_macro)]
#![no_std]

extern crate bare_metal;
extern crate cortex_m;
extern crate cortex_m_semihosting;
extern crate cortex_m_rtfm as rtfm;
extern crate stm32l151;
extern crate vcell;

use core::fmt::Write;
use core::ops::Deref;
use core::cmp::min;

use bare_metal::Peripheral;
use rtfm::{app, Threshold};
use cortex_m::peripheral::SystClkSource;
use cortex_m_semihosting::hio;
use vcell::VolatileCell;

pub const USB_MAX_PACKET_SIZE : u32 = 64;

pub const PMA: Peripheral<PMA> = unsafe { Peripheral::new(0x4000_6000) };
const BTABLE : usize = 0;

pub struct PMA {
    pma_area: PMA_Area,
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
    pub fn get_u16(&self, offset: usize) -> u16 {
        assert!((offset & 0x01) == 0);
        self.words[offset].get()
    }

    pub fn set_u16(&self, offset: usize, val: u16) {
        assert!((offset & 0x01) == 0);
        self.words[offset].set(val);
    }

    pub fn get_rxcount(&self, ep: usize) -> u16 {
        self.get_u16(BTABLE + (ep * 8) + 6) & 0x3ff
    }

    pub fn set_rxcount(&self, ep: usize, val: u16) {
        assert!(val <= 1024);
        let rval: u16 = {
            if val > 62 {
                assert!((val & 0x1f) == 0);
                (((val >> 5) - 1) << 10) | 0x8000
            } else {
                assert!((val & 1) == 0);
                (val >> 1) << 10
            }
        };
        self.set_u16(BTABLE + (ep * 8) + 6, rval)
    }

    pub fn write_buffer_u8(&self, base: usize, buf: &[u8]) {
        let mut last: u16 = 0;
        let mut off: usize = 0;

        for (ofs, v) in buf.iter().enumerate() {
            off = ofs;
            if ofs & 1 == 0 {
                last = *v as u16;
            } else {
                self.set_u16(base + ofs & !1, last | ((*v as u16)<< 8));
            }
        }

        if off & 1 == 0 {
            //self.set_u16(base + (off * 2), last);
            self.set_u16(base + off, last);
        }
    }
}


app! {
    device: stm32l151,

    resources: {
        static STDOUT: hio::HStdout;
        static KEY_STATE: [bool; 5 * 14] = [false; 5 * 14];
    },

    tasks: {
        SYS_TICK: {
            path: tick,
            resources: [STDOUT, GPIOA, GPIOB, SYST, KEY_STATE, USB, SYSCFG, RCC],
        },
        USB_LP: {
            path: usb_lp,
            resources: [STDOUT, USB],
        },
    }
}

fn init(p: init::Peripherals, _r: init::Resources) -> init::LateResourceValues {
    init_clock(&p);
    init_gpio(&p);
    init_usb(&p);

    p.SYST.set_clock_source(SystClkSource::Core);
    p.SYST.set_reload(100_000);
    p.SYST.enable_interrupt();
    p.SYST.enable_counter();

    let pma = PMA.get();
    for i in 0..256 {
        unsafe { 
            (*pma).pma_area.set_u16(i * 2, 0);
        }
    }

    init::LateResourceValues {
        STDOUT: hio::hstdout().unwrap(),
    }
}

fn idle() -> !{
    loop {
        rtfm::wfi();
    }
}

fn tick(_t: &mut Threshold, r: SYS_TICK::Resources) {
    r.GPIOB.odr.modify(|_, w| w.odr5().bit(true));

    let current_tick = r.SYST.cvr.read();
    let wait_until_tick = current_tick - 100;
    while r.SYST.cvr.read() > wait_until_tick {}

    //if r.GPIOA.idr.read().idr0().bit_is_set() {
    //if r.GPIOB.idr.read().idr6().bit_is_set() {
    //if r.GPIOB.idr.read().idr7().bit_is_set() {
    //if r.GPIOB.idr.read().idr8().bit_is_set() {
    if r.GPIOB.idr.read().idr9().bit_is_set() {
        unsafe { HID_REPORT[2] = 0x4 };
        //write!(r.STDOUT, "x").unwrap()
    } else {
        unsafe { HID_REPORT[2] = 0x0 };
        //write!(r.STDOUT, "o").unwrap()
    }

    r.GPIOB.odr.modify(|_, w| w.odr5().bit(false));
}

static mut nreset: usize = 0;

fn usb_reset(r: &mut USB_LP::Resources) {
    r.USB.istr.modify(|_, w| w.reset().clear_bit());

    let pma = PMA.get();
    unsafe {
        (*pma).pma_area.set_u16(0, 0x40);
        (*pma).pma_area.set_u16(2, 0x0);
        (*pma).pma_area.set_u16(4, 0x20);
        (*pma).pma_area.set_u16(6, (0x8000 | ((USB_MAX_PACKET_SIZE / 32) - 1) << 10) as u16);
        (*pma).pma_area.set_u16(8, 0x100);
        (*pma).pma_area.set_u16(10, 0x0);


        (*pma).write_buffer_u8(0x100, &HID_REPORT);
        (*pma).pma_area.set_u16(10, 5);
    }

    r.USB.usb_ep0r.modify(|_, w|
        unsafe {
            w.ep_type().bits(0b01)
             .stat_tx().bits(0b10)
             .stat_rx().bits(0b11)
        }
    );

    r.USB.usb_ep1r.modify(|_, w|
        unsafe {
            w.ep_type().bits(0b11)
             .stat_tx().bits(0b11)
             .stat_rx().bits(0b10)
             .ea().bits(0b1)
        }
    );

    r.USB.daddr.modify(|_, w| w.ef().set_bit());

    unsafe {
        logp = 0;
        if nreset > 1 {
            write!(r.STDOUT, "r").unwrap();
        }
        nreset += 1;
    }
}


//(USB_EP_CTR_RX|USB_EP_SETUP|USB_EP_T_FIELD|USB_EP_KIND|USB_EP_CTR_TX|USB_EPADDR_FIELD);
const USB_EPREG_MASK: u32 = (1 << 15 | 1 << 11 | 1 << 10 | 1 << 9 | 1 << 8 | 0xf);

const USB_EPTX_STAT: u32 = 0x30;
const USB_EPTX_DTOGMASK: u32 = (USB_EPTX_STAT|USB_EPREG_MASK);

const USB_EPRX_STAT: u32 = 0x3000;
const USB_EPRX_DTOGMASK: u32 = (USB_EPRX_STAT|USB_EPREG_MASK);

const USB_EP_CTR_RX: u32 = 0x8000;
const USB_EP_CTR_TX: u32 = 0x80000000;

// TODO: more from header
const USB_REQ_GET_STATUS: u8 = 0x00;
const USB_REQ_CLEAR_FEATURE: u8 = 0x01;
const USB_REQ_SET_FEATURE: u8 = 0x03;
const USB_REQ_SET_ADDRESS: u8 = 0x05;
const USB_REQ_GET_DESCRIPTOR: u8 = 0x06;
const USB_REQ_SET_DESCRIPTOR: u8 = 0x07;
const USB_REQ_GET_CONFIGURATION: u8 = 0x08;
const USB_REQ_SET_CONFIGURATION: u8 = 0x09;
const USB_REQ_GET_INTERFACE: u8 = 0x0A;
const USB_REQ_SET_INTERFACE: u8 = 0x0B;
const USB_REQ_SYNCH_FRAME: u8 = 0x0C;

const USB_DESC_TYPE_DEVICE: u8 = 1;
const USB_DESC_TYPE_CONFIGURATION: u8 = 2;
const USB_DESC_TYPE_STRING: u8 = 3;
const USB_DESC_TYPE_INTERFACE: u8 = 4;
const USB_DESC_TYPE_ENDPOINT: u8 = 5;
const USB_DESC_TYPE_DEVICE_QUALIFIER: u8 = 6;
const USB_DESC_TYPE_OTHER_SPEED_CONFIGURATION: u8 = 7;
const USB_DESC_TYPE_BOS: u8 = 0x0F;
const USB_DESC_TYPE_HID_REPORT: u8 = 0x22;

const DEV_DESC: [u8; 18] = [
    0x12, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, 0x40,
    0xff, 0xff, 0xff, 0xff, 0x01, 0x00, 0x01, 0x02,
    0x03, 0x01
];

const CONF_DESC: [u8; 34] = [
    0x09, 0x02, 0x22, 0x00, 0x01, 0x01, 0x04, 0x80, 0xfa,
    0x09, 0x04, 0x00, 0x00, 0x01, 0x03, 0x01, 0x01, 0x05,
    0x09, 0x21, 0x11, 0x01, 0x00, 0x01, 0x22, 0x29, 0x00,
    0x07, 0x05, 0x81, 0x03, 0x40, 0x00, 0x01
];

const HID_REPORT_DESC: [u8; 41] = [
    0x05, 0x01, // Usage Page: Generic Desktop Controls
    0x09, 0x06, // Usage: Keyboard
    0xa1, 0x01, // Collection: Application
    0x85, 0x01, //   Report ID: 1
    0x05, 0x07, //   Usage Page: Keybaord
    0x75, 0x01, //   Report Size: 1
    0x95, 0x08, //   Report Count: 8
    0x19, 0xe0, //   Usage Minimum: Keyboard LeftControl
    0x29, 0xe7, //   Usage Maximum: Keyboard Right GUI
    0x15, 0x00, //   Logical Minimum: 0
    0x25, 0x01, //   Logical Maximum: 1
    0x81, 0x02, //   Input: Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position
    0x95, 0x03, //   Report Count (3)
    0x75, 0x08, //   Report Size (8)
    0x15, 0x00, //   Logical Minimum (0)
    0x25, 0x64, //   Logical Maximum (100)
    0x05, 0x07, //   Usage Page (Kbrd/Keypad)
    0x19, 0x00, //   Usage Minimum (0x00)
    0x29, 0x65, //   Usage Maximum (0x65)
    0x81, 0x00, //   Input (Data,Array,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0xC0,       // End Collection
];

const DEVICE_QUALIFIER: [u8; 10] = [
    0x0a, 0x06, 0x00, 0x02, 0x00, 0x00, 0x40, 0x01, 0x00, 0x00
];

const LANG_STR: [u8; 4] = [0x04, 0x03, 0x09, 0x04];

// String Descriptor 1, "Rusty Manufacturer"
const MANUFACTURER_STR: [u8; 38] = [
    0x26, 0x03, 0x52, 0x00, 0x75, 0x00, 0x73, 0x00,
    0x74, 0x00, 0x79, 0x00, 0x20, 0x00, 0x4d, 0x00,
    0x61, 0x00, 0x6e, 0x00, 0x75, 0x00, 0x66, 0x00,
    0x61, 0x00, 0x63, 0x00, 0x74, 0x00, 0x75, 0x00,
    0x72, 0x00, 0x65, 0x00, 0x72, 0x00
];

// String Descriptor 2, "Rusty Product"
const PRODUCT_STR: [u8; 28] = [
    0x1c, 0x03, 0x52, 0x00, 0x75, 0x00, 0x73, 0x00,
    0x74, 0x00, 0x79, 0x00, 0x20, 0x00, 0x50, 0x00,
    0x72, 0x00, 0x6f, 0x00, 0x64, 0x00, 0x75, 0x00,
    0x63, 0x00, 0x74, 0x00
];

// String Descriptor 3, "123ABC"
const SERIAL_NUMBER_STR: [u8; 14] = [
    0x0e, 0x03, 0x31, 0x00, 0x32, 0x00, 0x33, 0x00,
    0x41, 0x00, 0x42, 0x00, 0x43, 0x00
];

const CONF_STR: [u8; 40] = [
    0x28, 0x03, 0x52, 0x00, 0x75, 0x00, 0x73, 0x00,
    0x74, 0x00, 0x79, 0x00, 0x20, 0x00, 0x43, 0x00,
    0x6f, 0x00, 0x6e, 0x00, 0x66, 0x00, 0x69, 0x00,
    0x67, 0x00, 0x75, 0x00, 0x72, 0x00, 0x61, 0x00,
    0x74, 0x00, 0x69, 0x00, 0x6f, 0x00, 0x6e, 0x00
];

fn usb_clear_tx_ep_ctr(r: &mut USB_LP::Resources) {
    r.USB.usb_ep0r.write(|w|
        unsafe {
            w.bits((r.USB.usb_ep0r.read().bits() & 0xFF7F) & USB_EPREG_MASK)
        }
    );
}

fn usb_clear_rx_ep_ctr(r: &mut USB_LP::Resources) {
    r.USB.usb_ep0r.write(|w|
        unsafe {
            w.bits((r.USB.usb_ep0r.read().bits() & 0x7FFF) & USB_EPREG_MASK)
        }
    );
}


static mut daddr: u8 = 0;
static mut logp: usize = 0;
static mut logd: [u32; 100] = [0; 100];
static mut logi: [u32; 100] = [0; 100];
static mut logr: [u32; 100] = [0; 100];
static mut logv: [u32; 100] = [0; 100];
static mut logrxc: [u16; 100] = [0; 100];
static mut logrxv: [u16; 100] = [0; 100];
static mut logrxv2: [u16; 100] = [0; 100];
static mut logrxv3: [u16; 100] = [0; 100];
static mut logrxv4: [u16; 100] = [0; 100];
static mut logtxc: [u16; 100] = [0; 100];
static mut logtxv: [u16; 100] = [0; 100];
static mut logaddr: [u16; 100] = [0; 100];

fn log(r: &mut USB_LP::Resources, d: u32) {
    unsafe {
        if logp < 100 {
            logd[logp] = d;
            logv[logp] = r.USB.usb_ep0r.read().bits();
            logr[logp] = r.USB.usb_cntr.read().bits();
            logi[logp] = r.USB.istr.read().bits();
            logaddr[logp] = r.USB.daddr.read().bits() as u16;
            let pma = PMA.get();
            logrxc[logp] = (*pma).pma_area.get_u16(6);
            logtxc[logp] = (*pma).pma_area.get_u16(2);
            logrxv[logp] = (*pma).pma_area.get_u16(0x20);
            logrxv2[logp] = (*pma).pma_area.get_u16(0x22);
            logrxv3[logp] = (*pma).pma_area.get_u16(0x24);
            logrxv4[logp] = (*pma).pma_area.get_u16(0x26);
            logtxv[logp] = (*pma).pma_area.get_u16(0x40);
            /*
            if logp == 20 {
                write!(r.STDOUT, "iiiii").unwrap();
            }
            */
            logp += 1;
        }
    }
}

fn set_ep_tx_status_valid(r: &mut USB_LP::Resources) {
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
    r.USB.usb_ep0r.write(|w|
    unsafe {
         w.bits(bb | USB_EP_CTR_RX | USB_EP_CTR_TX)
    });
}

fn set_ep_tx_status_valid_dtog(r: &mut USB_LP::Resources) {
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
    r.USB.usb_ep0r.write(|w|
    unsafe {
         w.bits(bb | USB_EP_CTR_RX | USB_EP_CTR_TX)
    });
}

fn set_ep_rx_status_valid(r: &mut USB_LP::Resources) {
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
    r.USB.usb_ep0r.write(|w|
        unsafe {
            w.bits(bb | USB_EP_CTR_RX | USB_EP_CTR_TX)
        }
    );
}

fn set_ep_rx_status_valid_dtog(r: &mut USB_LP::Resources) {
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
    r.USB.usb_ep0r.write(|w|
        unsafe {
            w.bits(bb | USB_EP_CTR_RX | USB_EP_CTR_TX)
        }
    );
}

fn ep_rx_toggle_dtog(r: &mut USB_LP::Resources) {
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
    r.USB.usb_ep0r.write(|w|
        unsafe {
            w.bits(bb | USB_EP_CTR_RX | USB_EP_CTR_TX)
        }
    );
}

fn usb_ctr(mut r: &mut USB_LP::Resources) {
    //if unsafe { logp == 30 } {
        //log(&mut r, 1);
    //}

    if !r.USB.istr.read().dir().bit_is_set() {
        usb_clear_tx_ep_ctr(&mut r);
        unsafe {
            if daddr != 0 {
                r.USB.daddr.modify(|_, w| unsafe { w.add().bits(daddr) });
                daddr = 0;
                set_ep_rx_status_valid(&mut r);
            } else {
                let pma = PMA.get();
                unsafe {
                    (*pma).pma_area.set_u16(6, 0);
                }
                set_ep_rx_status_valid_dtog(&mut r);
            }
        }
    } else {

        usb_clear_rx_ep_ctr(&mut r);
        //log(&mut r, 2);

        //if unsafe { logp != 15 } {
        let pma = PMA.get();
        unsafe {
            let request16 = (*pma).pma_area.get_u16(0x20);
            let value = (*pma).pma_area.get_u16(0x22);
            let index = (*pma).pma_area.get_u16(0x24);
            let length = (*pma).pma_area.get_u16(0x26);

            (*pma).pma_area.set_u16(6, (0x8000 | ((USB_MAX_PACKET_SIZE / 32) - 1) << 10) as u16);

            let request = ((request16 & 0xff00) >> 8) as u8;
            let request_type = (request16 & 0xff) as u8;
            match (request_type, request)  {
                (0, USB_REQ_SET_ADDRESS) => {
                    unsafe {
                        daddr = value as u8;
                    }
                    set_ep_tx_status_valid(&mut r);
                }
                (0x80, USB_REQ_GET_DESCRIPTOR) => {
                    let descriptor_type = (value >> 8) as u8;
                    let descriptor_index = (value & 0xff) as u8;
                    match (descriptor_type, descriptor_index) {
                        (USB_DESC_TYPE_DEVICE, _) => {
                            (*pma).write_buffer_u8(0x40, &DEV_DESC);
                            (*pma).pma_area.set_u16(2, min(length, DEV_DESC.len() as u16));
                            set_ep_tx_status_valid(&mut r);
                        }
                        (USB_DESC_TYPE_CONFIGURATION, _) => {
                            (*pma).write_buffer_u8(0x40, &CONF_DESC);
                            (*pma).pma_area.set_u16(2, min(length, CONF_DESC.len() as u16));
                            set_ep_tx_status_valid_dtog(&mut r);
                        }
                        (USB_DESC_TYPE_STRING, _) => {
                            let string = match descriptor_index {
                                0 => &LANG_STR[..],
                                1 => &MANUFACTURER_STR[..],
                                2 => &PRODUCT_STR[..],
                                3 => &SERIAL_NUMBER_STR[..],
                                4 => &CONF_STR[..],
                                _ => &PRODUCT_STR[..],
                            };
                            (*pma).write_buffer_u8(0x40, string);
                            (*pma).pma_area.set_u16(2, min(length, string.len() as u16));
                            set_ep_tx_status_valid_dtog(&mut r);
                        }
                        (USB_DESC_TYPE_DEVICE_QUALIFIER, _) => {
                            (*pma).write_buffer_u8(0x40, &DEVICE_QUALIFIER);
                            (*pma).pma_area.set_u16(2, min(length, DEVICE_QUALIFIER.len() as u16));
                            set_ep_tx_status_valid_dtog(&mut r);
                        }
                        _ => loop {}
                    }
                }
                (0x81, USB_REQ_GET_DESCRIPTOR) => {
                    let descriptor_type = (value >> 8) as u8;
                    let descriptor_index = (value & 0xff) as u8;
                    match (descriptor_type, descriptor_index) {
                        (USB_DESC_TYPE_HID_REPORT, _) => {
                            (*pma).write_buffer_u8(0x40, &HID_REPORT_DESC);
                            (*pma).pma_area.set_u16(2, min(length, HID_REPORT_DESC.len() as u16));
                            set_ep_tx_status_valid_dtog(&mut r);
                        }
                        _ => loop {}
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
                (0x21, 0xa) => { // USBHID SET_IDLE
                    (*pma).pma_area.set_u16(2, 0);
                    set_ep_tx_status_valid_dtog(&mut r);
                }
                (33, 11) => { // ???
                    (*pma).pma_area.set_u16(2, 0);
                    set_ep_tx_status_valid_dtog(&mut r);
                }
                _ => loop{}
            }
        }
    }
}

fn usb_clear_tx_ep1_ctr(r: &mut USB_LP::Resources) {
    r.USB.usb_ep1r.write(|w|
        unsafe {
            w.bits((r.USB.usb_ep1r.read().bits() & 0xFF7F) & USB_EPREG_MASK)
        }
    );
}

fn usb_clear_rx_ep1_ctr(r: &mut USB_LP::Resources) {
    r.USB.usb_ep1r.write(|w|
        unsafe {
            w.bits((r.USB.usb_ep1r.read().bits() & 0x7FFF) & USB_EPREG_MASK)
        }
    );
}

fn set_ep1_rx_status_valid_dtog(r: &mut USB_LP::Resources) {
    let mut bb = r.USB.usb_ep1r.read().bits();
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
    r.USB.usb_ep1r.write(|w|
        unsafe {
            w.bits(bb | USB_EP_CTR_RX | USB_EP_CTR_TX)
        }
    );
}

fn set_ep1_tx_status_valid_dtog(r: &mut USB_LP::Resources) {
    let mut bb = r.USB.usb_ep1r.read().bits();
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
    r.USB.usb_ep1r.write(|w|
    unsafe {
         w.bits(bb | USB_EP_CTR_RX | USB_EP_CTR_TX)
    });
}

static mut HID_REPORT: [u8; 5] = [
    0x01, 0x00, 0x04, 0x00, 0x00
];

fn usb_hid_ctr(mut r: &mut USB_LP::Resources) {
    if !r.USB.istr.read().dir().bit_is_set() {
        usb_clear_tx_ep1_ctr(&mut r);
        let pma = PMA.get();
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

fn usb_lp(_t: &mut Threshold, mut r: USB_LP::Resources) {

    if r.USB.istr.read().ctr().bit_is_set() {
        while r.USB.istr.read().ctr().bit_is_set() {
            let endpoint = r.USB.istr.read().ep_id().bits();
            match endpoint {
                0 => {
                    log(&mut r, 4);
                    usb_ctr(&mut r);
                    log(&mut r, 5);
                }
                1 => {
                    usb_hid_ctr(&mut r);
                }
                _ => loop {}
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

fn init_clock(p: &init::Peripherals) {
    p.USB.usb_cntr.modify(|_, w| w.pdwn().clear_bit());

    p.FLASH.acr.modify(|_, w| { w.acc64().set_bit() });
    p.FLASH.acr.modify(|_, w| { w.prften().set_bit() });
    p.FLASH.acr.modify(|_, w| { w.latency().set_bit() });

    p.RCC.cr.modify(|_, w| w.hseon().set_bit());
    while p.RCC.cr.read().hserdy().bit_is_clear() {}

    p.RCC.apb2enr.modify(|_, w| w.syscfgen().set_bit());
    p.RCC.apb1enr.modify(|_, w| w.pwren().set_bit());

    p.PWR.cr.modify(|_, w| {
        w.lprun().clear_bit();
        unsafe { w.vos().bits(0b01) }
    });
    while p.PWR.csr.read().vosf().bit_is_set() {}

    p.RCC.cfgr.modify(|_, w| unsafe {
        w.ppre1().bits(0b100) 
         .ppre2().bits(0b100)
         .pllmul().bits(0b0010)
         .plldiv().bits(0b10)
         .pllsrc().set_bit()
    });

    p.RCC.cr.modify(|_, w| w.pllon().set_bit());
    while p.RCC.cr.read().pllrdy().bit_is_clear() {}

    p.RCC.cfgr.modify(|_, w| unsafe { w.sw().bits(0b11) });
    while p.RCC.cfgr.read().sws().bits() != 0b11 {};

    p.RCC.cr.modify(|_, w| w.msion().clear_bit());
}

fn init_gpio(p: &init::Peripherals) {
    p.RCC.ahbenr.modify(|_, w| {
        w.gpiopaen().set_bit()
         .gpiopben().set_bit()
    });

    p.GPIOB.moder.modify(|_, w| unsafe {
        w.moder3().bits(1)
         .moder4().bits(1)
         .moder5().bits(1)
    });

    p.GPIOA.pupdr.modify(|_, w| unsafe {
        w.pupdr0().bits(0b10)
    });

    p.GPIOB.pupdr.modify(|_, w| unsafe {
        w.pupdr3().bits(1)
         .pupdr4().bits(1)
         .pupdr5().bits(1)
         .pupdr6().bits(0b10)
         .pupdr7().bits(0b10)
         .pupdr8().bits(0b10)
         .pupdr9().bits(0b10)
    });
}

fn init_usb(p: &init::Peripherals) {
    p.RCC.apb1enr.modify(|_, w| w.usben().set_bit());
    p.RCC.apb1rstr.modify(|_, w| w.usbrst().set_bit());
    p.RCC.apb1rstr.modify(|_, w| w.usbrst().clear_bit());

    p.USB.usb_cntr.modify(|_, w| w.pdwn().clear_bit());

    p.USB.usb_cntr.modify(|_, w|
        w.ctrm().set_bit()
         .errm().set_bit()
         .pmaovrm().set_bit()
         //.wkupm().set_bit()
         //.suspm().set_bit()
         //.esofm().set_bit()
         //.sofm().set_bit()
         .resetm().set_bit()
    );

    p.USB.btable.reset();

    p.USB.usb_cntr.modify(|_, w| w.fres().clear_bit());

    p.USB.istr.reset();

    p.USB.daddr.modify(|_, w| w.ef().set_bit());

    p.SYSCFG.pmc.modify(|_, w| w.usb_pu().set_bit());
}
