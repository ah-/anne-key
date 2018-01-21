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

use core::fmt::Write;
use cortex_m::peripheral::SystClkSource;
use cortex_m_semihosting::hio;
use rtfm::{app, Threshold};

use bluetooth::Bluetooth;
use keyboard::Keyboard;
use led::Led;
use usb::Usb;

app! {
    device: stm32l151,

    resources: {
        static BLUETOOTH: Bluetooth = Bluetooth::new();
        static KEYBOARD: Keyboard = Keyboard::new();
        static LED: Led = Led::new();
        static USBC: Usb = Usb::new();
        static USB_LOG : usb::log::Log = usb::log::Log::new();
        static STDOUT: hio::HStdout;
        static X: usize = 0;
    },

    tasks: {
        SYS_TICK: {
            path: tick,
            resources: [STDOUT, GPIOA, GPIOB, SYST, KEYBOARD, BLUETOOTH, DMA1, X],
        },
        USART3: {
            path: led::receive,
            resources: [DMA1, USART3, STDOUT, LED],
        },
        USB_LP: {
            path: usb::usb_lp,
            resources: [STDOUT, USB, USB_LOG, USBC],
        },
        USART2: {
            path: bluetooth::receive,
            resources: [DMA1, USART2, STDOUT, BLUETOOTH, KEYBOARD],
        },
        DMA1_CHANNEL7: {
            path: bluetooth::tx_complete,
            resources: [DMA1],
        }
    }
}

fn init(p: init::Peripherals, r: init::Resources) -> init::LateResourceValues {
    clock::init_clock(&p);

    r.KEYBOARD.init(&p);
    r.LED.init(&p);
    r.BLUETOOTH.init(&p);
    //r.USBC.init(&p);

    p.SYST.set_clock_source(SystClkSource::Core);
    p.SYST.set_reload(100_000);
    p.SYST.enable_interrupt();
    p.SYST.enable_counter();

    /*
    p.GPIOA.moder.modify(|_, w| unsafe {
        w.moder0().bits(1)
    });

    p.GPIOA.pupdr.modify(|_, w| unsafe {
        w.pupdr0().bits(0b10)
    });
    */

    init::LateResourceValues {
        STDOUT: hio::hstdout().unwrap(),
    }
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

fn tick(_t: &mut Threshold, r: SYS_TICK::Resources) {
    r.KEYBOARD.sample(r.GPIOA, r.GPIOB, r.SYST);
    let pressed = r.KEYBOARD.state.into_iter().filter(|s| **s).count();
    if pressed != **r.X {
        **r.X = pressed;
        r.BLUETOOTH.send_report(&r.KEYBOARD, &r.DMA1);
        //write!(r.STDOUT, "{}", pressed).unwrap();
    }
}
