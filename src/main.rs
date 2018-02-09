#![feature(const_fn)]
#![feature(proc_macro)]
#![feature(non_exhaustive)]
#![no_std]

extern crate bare_metal;
extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;
extern crate cortex_m_semihosting;
extern crate embedded_hal;
extern crate stm32l151;
extern crate stm32l151_hal as hal;

#[macro_use]
mod debug;

mod bluetooth;
mod clock;
mod hidreport;
mod keyboard;
mod keycodes;
mod layout;
mod led;
mod protocol;
mod serial;
//mod usb;

use cortex_m_semihosting::hio;
use rtfm::{app, Threshold};
use hal::gpio::GpioExt;
use stm32l151::DMA1;

use bluetooth::Bluetooth;
use keyboard::{Keyboard, KeyState};
use hidreport::HidReport;
use led::Led;
//use usb::Usb;
use serial::Serial;
use serial::bluetooth_usart::BluetoothUsart;
use serial::led_usart::LedUsart;


app! {
    device: stm32l151,

    resources: {
        static KEYBOARD: Keyboard;
        static BLUETOOTH_BUFFERS: [[u8; 0x10]; 2] = [[0; 0x10]; 2];
        static BLUETOOTH: Bluetooth<'static>;
        static LED_BUFFERS: [[u8; 0x10]; 2] = [[0; 0x10]; 2];
        static LED: Led<'static>;
        //static USB: Usb;
        static DMA1: stm32l151::DMA1;
        static SYST: stm32l151::SYST;
        static EXTI: stm32l151::EXTI;
        static NUM_PRESSED_KEYS: usize = 0;
    },

    init: {
        resources: [BLUETOOTH_BUFFERS, LED_BUFFERS],
    },

    tasks: {
        SYS_TICK: {
            path: tick,
            resources: [BLUETOOTH, LED, DMA1, KEYBOARD, NUM_PRESSED_KEYS, SYST],
        },
    /*
        USB_LP: {
            path: usb::usb_lp,
            resources: [USB],
        },
    */
        DMA1_CHANNEL2: {
            path: led::tx,
            resources: [LED, DMA1],
        },
        DMA1_CHANNEL3: {
            path: led::rx,
            resources: [LED, DMA1],
        },
        DMA1_CHANNEL6: {
            path: bluetooth::rx,
            resources: [BLUETOOTH, DMA1, KEYBOARD],
        },
        DMA1_CHANNEL7: {
            path: bluetooth::tx,
            resources: [BLUETOOTH, DMA1],
        },
        EXTI9_5: {
            path: exti9_5,
            resources: [EXTI],
        },
    }
}

fn init(mut p: init::Peripherals, r: init::Resources) -> init::LateResources {
    // vector table relocation because of bootloader
    unsafe { p.core.SCB.vtor.write(0x4000) };

    let mut d = p.device;
    clock::init_clock(&d);
    clock::enable_tick(&mut p.core.SYST, 100_000);

    let gpioa = d.GPIOA.split();
    let gpiob = d.GPIOB.split();
    let gpioc = d.GPIOC.split();

    let row_pins = (gpiob.pb9.pull_down(),
                    gpiob.pb8.pull_down(),
                    gpiob.pb7.pull_down(),
                    gpiob.pb6.pull_down(),
                    gpioa.pa0.pull_down());

    let column_pins = (gpioa.pa5.into_output().pull_up(),
                       gpioa.pa6.into_output().pull_up(),
                       gpioa.pa7.into_output().pull_up(),
                       gpiob.pb0.into_output().pull_up(),
                       gpiob.pb1.into_output().pull_up(),
                       gpiob.pb12.into_output().pull_up(),
                       gpiob.pb13.into_output().pull_up(),
                       gpiob.pb14.into_output().pull_up(),
                       gpioa.pa8.into_output().pull_up(),
                       gpioa.pa9.into_output().pull_up(),
                       gpioa.pa15.into_output().pull_up(),
                       gpiob.pb3.into_output().pull_up(),
                       gpiob.pb4.into_output().pull_up(),
                       gpiob.pb5.into_output().pull_up());

    let keyboard = Keyboard::new(row_pins, column_pins);

    let led_usart = LedUsart::new(d.USART3, gpiob.pb10, gpiob.pb11, &d.DMA1, &mut d.RCC);
    let led_serial = Serial::new(led_usart, &mut d.DMA1, r.LED_BUFFERS);
    let mut led = Led::new(led_serial, gpioc.pc15);

    let bluetooth_usart = BluetoothUsart::new(d.USART2, gpioa.pa1, gpioa.pa2, gpioa.pa3, &mut d.DMA1, &mut d.RCC);
    let bluetooth_serial = Serial::new(bluetooth_usart, &mut d.DMA1, r.BLUETOOTH_BUFFERS);
    let bluetooth = Bluetooth::new(bluetooth_serial);

    led.on();

    //let usb = Usb::new(d.USB, &mut d.RCC, &mut d.SYSCFG);

    init::LateResources {
        BLUETOOTH: bluetooth,
        KEYBOARD: keyboard,
        LED: led,
        //USB: usb,
        DMA1: d.DMA1,
        SYST: p.core.SYST,
        EXTI: d.EXTI,
    }
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

fn tick(_t: &mut Threshold, mut r: SYS_TICK::Resources) {
    r.KEYBOARD.sample(&r.SYST);
    let pressed = r.KEYBOARD.state.into_iter().filter(|s| **s).count();
    if pressed != *r.NUM_PRESSED_KEYS {
        *r.NUM_PRESSED_KEYS = pressed;
        let report = HidReport::from_key_state(&r.KEYBOARD.state);
        r.BLUETOOTH.send_report(&report, &mut r.DMA1);
        test_led(&mut r.LED, &mut r.DMA1, &r.KEYBOARD.state);
    }
}

fn test_led(led: &mut Led, dma1: &mut DMA1, state: &KeyState) {
    if state[0] {
        led.off();
    }
    if state[1] {
        led.on();
    }
    if state[2] {
        led.next_theme(dma1);
    }
    if state[3] {
        led.next_brightness(dma1);
    }
    if state[4] {
        led.next_animation_speed(dma1);
    }
    if state[15] {
        led.set_theme(0, dma1);
    }
    if state[16] {
        led.set_theme(2, dma1);
    }
    if state[17] {
        led.set_theme(2, dma1);
    }
    if state[18] {
        led.set_theme(3, dma1);
    }
    if state[19] {
        led.set_theme(14, dma1);
    }
    if state[20] {
        led.set_theme(18, dma1);
    }
    if state[21] {
        led.set_theme(17, dma1);
    }
    if state[22] {
        // sends O
        led.send_keys(&[0,0,0,1,0,0,0,0,0], dma1);
    }
    if state[23] {
        led.send_music(&[1,2,3,4,5,6,7,8,9], dma1);
    }
}

fn exti9_5(_t: &mut Threshold, r: EXTI9_5::Resources) {
    // this (plus other exti) are key presses,
    // maybe use them instead of timer based scanning?

    // maybe only clear set bits? or ones from 9-5?
    unsafe { r.EXTI.pr.write(|w| w.bits(0xffff)) };
}
