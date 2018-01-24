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
        //static USB_LOG : usb::log::Log = usb::log::Log::new();
        static X: usize = 0;
        static STDOUT: hio::HStdout;
    },

    tasks: {
        SYS_TICK: {
            path: tick,
            resources: [STDOUT, X, LED, KEYBOARD, GPIOA, GPIOB, DMA1, BLUETOOTH, USB, SYST],
        },
        USART3: {
            path: led::receive,
            resources: [STDOUT, LED],
        },
        /*
        USB_LP: {
            path: usb::usb_lp,
            resources: [STDOUT, USB, USB_LOG],
        },
        */
        USART2: {
            path: bluetooth::receive,
            resources: [STDOUT, BLUETOOTH, KEYBOARD, GPIOA, DMA1],
        },
        DMA1_CHANNEL7: {
            path: bluetooth::tx_complete,
            resources: [DMA1],
        }
    }
}

fn init(mut p: init::Peripherals, r: init::Resources) -> init::LateResources {
    clock::init_clock(&p.device);

    // TODO: merge new with init()?
    let mut led = Led::new(p.device.USART3);
    led.init(&p.device.DMA1, &mut p.device.GPIOB, &mut p.device.RCC);
    let keyboard = Keyboard::new();
    keyboard.init(&mut p.device.GPIOA, &mut p.device.GPIOB);
    let mut bluetooth = Bluetooth::new(p.device.USART2);
    bluetooth.init(&p.device.DMA1, &mut p.device.GPIOA, &mut p.device.RCC);
    let mut usb = Usb::new(p.device.USB);
    usb.init(&mut p.device.RCC, &mut p.device.SYSCFG);

    let mut syst = p.core.SYST;
    syst.set_reload(100_000);
    syst.enable_interrupt();
    syst.enable_counter();

    let gpioa = p.device.GPIOA;
    gpioa.moder.modify(|_, w| unsafe { w.moder1().bits(1) });
    gpioa.pupdr.modify(|_, w| unsafe { w.pupdr1().bits(0b01) });
    gpioa.odr.modify(|_, w| w.odr1().clear_bit()); 

    init::LateResources {
        BLUETOOTH: bluetooth,
        KEYBOARD: keyboard,
        LED: led,
        USB: usb,
        GPIOA: gpioa,
        GPIOB: p.device.GPIOB,
        DMA1: p.device.DMA1,
        SYST: syst,
        STDOUT: hio::hstdout().unwrap(),
    }
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

fn tick(_t: &mut Threshold, mut r: SYS_TICK::Resources) {
    write!(r.STDOUT, "tick {}", *r.X).unwrap();
    r.KEYBOARD.sample(&mut r.GPIOA, &mut r.GPIOB, &r.SYST);
    let pressed = r.KEYBOARD.state.into_iter().filter(|s| **s).count();
    if pressed != *r.X {
        *r.X = pressed;
        r.BLUETOOTH.send_report(&r.KEYBOARD, &r.DMA1, &mut r.STDOUT, &r.GPIOA);
        //write!(r.STDOUT, "{}", pressed).unwrap();
    }
}
