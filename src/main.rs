#![feature(const_fn)]
#![feature(proc_macro)]
#![feature(non_exhaustive)]
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
mod hidreport;
mod keyboard;
mod keycodes;
mod layout;
mod led;
mod protocol;
mod serial;
//mod usb;

use core::fmt::Write;
use cortex_m_semihosting::hio;
use rtfm::{app, Threshold};
use stm32l151::{DMA1, GPIOA, GPIOC};

use bluetooth::Bluetooth;
use keyboard::Keyboard;
use keyboard::KeyState;
use hidreport::HidReport;
use led::Led;
//use usb::Usb;
use serial::Serial;
use serial::bluetooth_usart::BluetoothUsart;
use serial::led_usart::LedUsart;
use protocol::{MsgType, LedOp};


app! {
    device: stm32l151,

    resources: {
        static KEYBOARD: Keyboard;
        static BLUETOOTH_BUFFERS: [[u8; 0x10]; 2] = [[0; 0x10]; 2];
        static BLUETOOTH: Bluetooth<'static>;
        static LED_BUFFERS: [[u8; 0x10]; 2] = [[0; 0x10]; 2];
        static LED: Led<'static>;
        //static USB: Usb;
        static GPIOA: stm32l151::GPIOA;
        static GPIOB: stm32l151::GPIOB;
        static GPIOC: stm32l151::GPIOC;
        static DMA1: stm32l151::DMA1;
        static SYST: stm32l151::SYST;
        static EXTI: stm32l151::EXTI;
        static NUM_PRESSED_KEYS: usize = 0;
        static STDOUT: Option<hio::HStdout>;
    },

    init: {
        resources: [BLUETOOTH_BUFFERS, LED_BUFFERS],
    },

    tasks: {
        SYS_TICK: {
            path: tick,
            resources: [BLUETOOTH, LED, DMA1, GPIOA, GPIOB, GPIOC, KEYBOARD, NUM_PRESSED_KEYS, STDOUT, SYST],
        },
    /*
        USB_LP: {
            path: usb::usb_lp,
            resources: [STDOUT, USB],
        },
    */
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
        EXTI9_5: {
            path: exti9_5,
            resources: [EXTI],
        },
        USART3: {
            path: led::usart3,
        },
    }
}

fn init(mut p: init::Peripherals, r: init::Resources) -> init::LateResources {
    // vector table relocation because of bootloader
    unsafe { p.core.SCB.vtor.write(0x4000) };

    let mut d = p.device;
    clock::init_clock(&d);
    clock::enable_tick(&mut p.core.SYST, 100_000);

    let keyboard = Keyboard::new(&mut d.GPIOA, &mut d.GPIOB);

    let led_usart = LedUsart::new(d.USART3, &d.DMA1, &mut d.GPIOB, &mut d.RCC);
    let led_serial = Serial::new(led_usart, &mut d.DMA1, &mut d.GPIOA, r.LED_BUFFERS);
    let led = Led::new(led_serial, &mut d.GPIOC);
    // led.on();

    let bluetooth_usart = BluetoothUsart::new(d.USART2, &d.DMA1, &mut d.GPIOA, &mut d.RCC);
    let bluetooth_serial = Serial::new(bluetooth_usart, &mut d.DMA1, &mut d.GPIOA, r.BLUETOOTH_BUFFERS);
    let bluetooth = Bluetooth::new(bluetooth_serial);

    /*
    let usb = Usb::new(d.USB, &mut d.RCC, &mut d.SYSCFG);
    */

    init::LateResources {
        BLUETOOTH: bluetooth,
        KEYBOARD: keyboard,
        LED: led,
        //USB: usb,
        GPIOA: d.GPIOA,
        GPIOB: d.GPIOB,
        GPIOC: d.GPIOC,
        DMA1: d.DMA1,
        SYST: p.core.SYST,
        EXTI: d.EXTI,
        STDOUT: None, //hio::hstdout().ok(), // None
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
        test_led(&mut r.LED, &mut r.DMA1, &mut r.STDOUT, &mut r.GPIOA, &mut r.GPIOC, &r.KEYBOARD.state);
    }
}

fn test_led(led: &mut Led, dma1: &mut DMA1, stdout: &mut Option<hio::HStdout>, gpioa: &mut GPIOA, gpioc: &mut GPIOC, state: &KeyState) {
    if state[0] {
        led.off(gpioc);
    }
    if state[1] {
        led.on(gpioc);
    }
    if state[2] {
        led.serial.send(MsgType::Led, LedOp::ConfigCmd as u8,
                         &[1, 0, 0, 0], dma1, stdout, gpioa);
    }
    if state[3] {
        led.serial.send(MsgType::Led, LedOp::ConfigCmd as u8,
                         &[0, 1, 0, 0], dma1, stdout, gpioa);
    }
    if state[4] {
        led.serial.send(MsgType::Led, LedOp::ConfigCmd as u8,
                         &[0, 0, 1, 0], dma1, stdout, gpioa);
    }
    if state[5] {
        led.serial.send(MsgType::Led, LedOp::ConfigCmd as u8,
                         &[0, 0, 0, 1], dma1, stdout, gpioa);
    }
    if state[15] {
        led.set_theme(0, dma1, stdout, gpioa);
    }
    if state[16] {
        led.set_theme(1, dma1, stdout, gpioa);
    }
    if state[17] {
        led.set_theme(2, dma1, stdout, gpioa);
    }
    if state[18] {
        led.set_theme(3, dma1, stdout, gpioa);
    }
    if state[19] {
        led.set_theme(14, dma1, stdout, gpioa);
    }
    if state[20] {
        led.set_theme(18, dma1, stdout, gpioa);
    }
    if state[21] {
        led.set_theme(17, dma1, stdout, gpioa);
    }
    if state[22] {
        // sends O
        led.send_keys(&[0,0,0,1,0,0,0,0,0], dma1, stdout, gpioa);
    }
    if state[23] {
        led.send_music(&[1,2,3,4,5,6,7,8,9], dma1, stdout, gpioa);
    }
}

fn exti9_5(_t: &mut Threshold, r: EXTI9_5::Resources) {
    // this (plus other exti) are key presses,
    // maybe use them instead of timer based scanning?
    // write!(hio::hstdout().unwrap(), "EXTI9_5").ok();

    // maybe only clear set bits? or ones from 9-5?
    unsafe { r.EXTI.pr.write(|w| w.bits(0xffff)) };
}
