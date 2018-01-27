#![feature(const_fn)]
#![feature(proc_macro)]
#![no_std]

extern crate bare_metal;
extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;
extern crate cortex_m_semihosting;
extern crate stm32l151;

mod bluetooth;
mod clock;
mod keyboard;
mod led;
mod usb;

use cortex_m_semihosting::hio;
use rtfm::{app, Threshold};

use bluetooth::Bluetooth;
use keyboard::Keyboard;
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
        static STDOUT: hio::HStdout;
    },

    tasks: {
        SYS_TICK: {
            path: tick,
            resources: [STDOUT, NUM_PRESSED_KEYS, KEYBOARD, BLUETOOTH, SYST, GPIOA, GPIOB, DMA1],
        },
        USART3: {
            path: led::receive,
            resources: [STDOUT, LED],
        },
        USB_LP: {
            path: usb::usb_lp,
            resources: [STDOUT, USB, USB_LOG],
        },
        USART2: {
            path: bluetooth::receive,
            resources: [STDOUT, BLUETOOTH, KEYBOARD, GPIOA, DMA1],
        },
        DMA1_CHANNEL7: {
            path: bluetooth::tx_complete,
            resources: [STDOUT, DMA1],
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
        STDOUT: hio::hstdout().unwrap(),
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
        r.BLUETOOTH.send_report(&r.KEYBOARD, &r.DMA1, &mut r.STDOUT, &r.GPIOA);
    }
}
