#![feature(const_fn)]
#![feature(proc_macro)]
#![no_std]

extern crate bare_metal;
extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;
extern crate cortex_m_semihosting;
extern crate stm32l151;

#[macro_use]
mod macros;
mod bluetooth;
mod clock;
mod keyboard;
mod keycodes;
mod keymap;
mod layout;
mod led;
mod protocol;
mod serial;
mod usb;

use cortex_m_semihosting::hio;
use rtfm::{app, Threshold};

use bluetooth::Bluetooth;
use keyboard::Keyboard;
use keymap::HidReport;
use led::Led;
use usb::Usb;
use serial::Serial;

app! {
    device: stm32l151,

    resources: {
        static BLUETOOTH_SEND_BUFFER: [u8; 0x10] = [0; 0x10];
        static BLUETOOTH_RECEIVE_BUFFER: [u8; 0x10] = [0; 0x10];
        static BLUETOOTH: Bluetooth<'static>;
        static KEYBOARD: Keyboard;
        static LED_SEND_BUFFER: [u8; 0x10] = [0; 0x10];
        static LED_RECEIVE_BUFFER: [u8; 0x10] = [0; 0x10];
        static LED: Led<'static>;
        static USB: Usb;
        static GPIOA: stm32l151::GPIOA;
        static GPIOB: stm32l151::GPIOB;
        static DMA1: stm32l151::DMA1;
        static SYST: stm32l151::SYST;
        static USB_LOG : usb::log::Log = usb::log::Log::new();
        static NUM_PRESSED_KEYS: usize = 0;
        static STDOUT: Option<hio::HStdout>;
    },

    init: {
        resources: [BLUETOOTH_SEND_BUFFER, BLUETOOTH_RECEIVE_BUFFER,
                    LED_SEND_BUFFER, LED_RECEIVE_BUFFER],
    },

    tasks: {
        SYS_TICK: {
            path: tick,
            resources: [BLUETOOTH, DMA1, GPIOA, GPIOB, KEYBOARD, NUM_PRESSED_KEYS, STDOUT, SYST],
        },
        USB_LP: {
            path: usb::usb_lp,
            resources: [STDOUT, USB, USB_LOG],
        },
        DMA1_CHANNEL2: {
            path: led::tx,
            resources: [LED, DMA1, STDOUT],
        },
        DMA1_CHANNEL3: {
            path: led::rx,
            resources: [LED, DMA1, GPIOA, STDOUT],
        },
        DMA1_CHANNEL6: {
            path: bluetooth::rx,
            resources: [BLUETOOTH, DMA1, GPIOA, KEYBOARD, STDOUT],
        },
        DMA1_CHANNEL7: {
            path: bluetooth::tx,
            resources: [BLUETOOTH, DMA1, STDOUT],
        },
    }
}

fn init(mut p: init::Peripherals, r: init::Resources) -> init::LateResources {
    let mut d = p.device;
    clock::init_clock(&d);
    clock::enable_tick(&mut p.core.SYST, 100_000);

    let keyboard = Keyboard::new(&mut d.GPIOA, &mut d.GPIOB);

    let led_serial = Serial::new(d.USART3, &mut d.DMA1, &mut d.GPIOA, &mut d.GPIOB, &mut d.RCC,
                                 r.LED_SEND_BUFFER, r.LED_RECEIVE_BUFFER);
    let led = Led::new(led_serial);

    let bluetooth_serial = Serial::new(d.USART2, &mut d.DMA1, &mut d.GPIOA, &mut d.GPIOB, &mut d.RCC,
                                       r.BLUETOOTH_SEND_BUFFER, r.BLUETOOTH_RECEIVE_BUFFER);
    let bluetooth = Bluetooth::new(bluetooth_serial);

    let usb = Usb::new(d.USB, &mut d.RCC, &mut d.SYSCFG);

    init::LateResources {
        BLUETOOTH: bluetooth,
        KEYBOARD: keyboard,
        LED: led,
        USB: usb,
        GPIOA: d.GPIOA,
        GPIOB: d.GPIOB,
        DMA1: d.DMA1,
        SYST: p.core.SYST,
        STDOUT: hio::hstdout().ok(),
    }
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

fn tick(_t: &mut Threshold, mut r: SYS_TICK::Resources) {
    r.KEYBOARD.sample(&mut r.GPIOA, &mut r.GPIOB, &r.SYST);
    let pressed = r.KEYBOARD.state.into_iter().filter(|s| **s).count();
    if pressed != *r.NUM_PRESSED_KEYS {
        *r.NUM_PRESSED_KEYS = pressed;
        let report = HidReport::from_key_state(&r.KEYBOARD.state);
        r.BLUETOOTH.send_report(&report, &mut r.DMA1, &mut r.STDOUT, &mut r.GPIOA);
    }
}
