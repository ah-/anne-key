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
mod usb;

use cortex_m_semihosting::hio;
use rtfm::{app, Threshold};

use bluetooth::Bluetooth;
use keyboard::Keyboard;
use keymap::HidReport;
use led::Led;
use usb::Usb;

app! {
    device: stm32l151,

    resources: {
        static BLUETOOTH: Bluetooth;
        static KEYBOARD: Keyboard;
        static LED: Led;
        static USB: Usb;
        static GPIOA: stm32l151::GPIOA;
        static GPIOB: stm32l151::GPIOB;
        static DMA1: stm32l151::DMA1;
        static SYST: stm32l151::SYST;
        static USB_LOG : usb::log::Log = usb::log::Log::new();
        static NUM_PRESSED_KEYS: usize = 0;
        static STDOUT: Option<hio::HStdout>;
    },

    tasks: {
        SYS_TICK: {
            path: tick,
            resources: [BLUETOOTH, DMA1, GPIOA, GPIOB, KEYBOARD, NUM_PRESSED_KEYS, STDOUT, SYST],
        },
        USART3: {
            path: led::receive,
            resources: [LED, STDOUT],
        },
        USB_LP: {
            path: usb::usb_lp,
            resources: [STDOUT, USB, USB_LOG],
        },
        DMA1_CHANNEL6: {
            path: bluetooth::rx,
            resources: [BLUETOOTH, DMA1, GPIOA, KEYBOARD, STDOUT],
        },
        DMA1_CHANNEL7: {
            path: bluetooth::tx,
            resources: [DMA1, STDOUT],
        }
    }
}

fn init(mut p: init::Peripherals, _r: init::Resources) -> init::LateResources {
    let mut d = p.device;
    clock::init_clock(&d);
    clock::enable_tick(&mut p.core.SYST, 100_000);

    let keyboard = Keyboard::new(&mut d.GPIOA, &mut d.GPIOB);
    let led = Led::new(d.USART3, &d.DMA1, &mut d.GPIOB, &mut d.RCC);
    let bluetooth = Bluetooth::new(d.USART2, &d.DMA1, &mut d.GPIOA, &mut d.RCC);
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
        r.BLUETOOTH.send_report(&report, &r.DMA1, &mut r.STDOUT, &r.GPIOA);
    }
}
