#![feature(const_fn)]
#![feature(unsize)]
#![no_main]
#![no_std]

#[cfg(not(feature = "use_semihosting"))]
extern crate panic_abort;
#[cfg(feature = "use_semihosting")]
extern crate panic_semihosting;
#[cfg(feature = "use_semihosting")]
use cortex_m_semihosting::heprintln;

mod debug;

#[macro_use]
mod action;
mod bluetooth;
mod clock;
mod hidreport;
mod keyboard;
mod keycodes;
mod keymatrix;
mod layout;
mod led;
mod protocol;
mod serial;
mod usb;

use hal::dma::DmaExt;
use hal::gpio::GpioExt;
use rtfm::app;

use crate::bluetooth::Bluetooth;
use crate::keyboard::Keyboard;
use crate::keymatrix::KeyMatrix;
use crate::led::Led;
use crate::serial::bluetooth_usart::BluetoothUsart;
use crate::serial::led_usart::LedUsart;
use crate::serial::Serial;
use crate::usb::Usb;

#[app(device = stm32l1::stm32l151)]
const APP: () = {
    static mut KEYBOARD: Keyboard = Keyboard::new();
    static mut BLUETOOTH_BUFFERS: [[u8; 0x80]; 2] = [[0; 0x80]; 2];
    static mut LED_BUFFERS: [[u8; 0x80]; 2] = [[0; 0x80]; 2];

    // Late resources
    static mut BLUETOOTH: Bluetooth<[u8; 0x80]> = ();
    static mut LED: Led<[u8; 0x80]> = ();
    static mut KEY_MATRIX: KeyMatrix = ();
    static mut SYST: stm32l1::stm32l151::SYST = ();
    static mut EXTI: stm32l1::stm32l151::EXTI = ();
    static mut USB: Usb = ();

    #[init(resources = [BLUETOOTH_BUFFERS, LED_BUFFERS])]
    fn init() -> init::LateResources {
        // re-locate vector table to 0x80004000 because bootloader uses 0x80000000
        unsafe { core.SCB.vtor.write(0x4000) };

        clock::init_clock(&device);
        clock::enable_tick(&mut core.SYST, 100_000);

        let dma = device.DMA1.split();
        let gpioa = device.GPIOA.split();
        let gpiob = device.GPIOB.split();
        let gpioc = device.GPIOC.split();

        let row_pins = (
            gpiob.pb9.pull_down(),
            gpiob.pb8.pull_down(),
            gpiob.pb7.pull_down(),
            gpiob.pb6.pull_down(),
            gpioa.pa0.pull_down(),
        );

        // TODO: make pin a generic trait, then iterate over list and call .into_output().pull_up()?
        let column_pins = (
            gpioa.pa5.into_output().pull_up(),
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
            gpiob.pb5.into_output().pull_up(),
        );

        let key_matrix = KeyMatrix::new(row_pins, column_pins);

        let led_usart = LedUsart::new(
            device.USART3,
            gpiob.pb10,
            gpiob.pb11,
            dma.3,
            dma.2,
            &mut device.RCC,
        );
        let (led_send_buffer, led_receive_buffer) = resources.LED_BUFFERS.split_at_mut(1);
        let led_serial = Serial::new(led_usart, &mut led_send_buffer[0]);
        let mut led = Led::new(led_serial, &mut led_receive_buffer[0], gpioc.pc15);
        led.poke(&core.SYST).unwrap();
        led.theme_mode().unwrap();

        let bluetooth_usart = BluetoothUsart::new(
            device.USART2,
            gpioa.pa1,
            gpioa.pa2,
            gpioa.pa3,
            dma.6,
            dma.7,
            &mut device.RCC,
        );
        let (bt_send_buffer, bt_receive_buffer) = resources.BLUETOOTH_BUFFERS.split_at_mut(1);
        let bluetooth_serial = Serial::new(bluetooth_usart, &mut bt_send_buffer[0]);
        let bluetooth = Bluetooth::new(bluetooth_serial, &mut bt_receive_buffer[0]);

        let usb = Usb::new(device.USB, &mut device.RCC, &mut device.SYSCFG);

        init::LateResources {
            BLUETOOTH: bluetooth,
            KEY_MATRIX: key_matrix,
            LED: led,
            SYST: core.SYST,
            EXTI: device.EXTI,
            USB: usb,
        }
    }

    #[exception(resources = [BLUETOOTH, LED, KEY_MATRIX, SYST, KEYBOARD, USB])]
    fn SysTick() {
        resources.KEY_MATRIX.sample(&resources.SYST);
        resources.KEYBOARD.process(
            &resources.KEY_MATRIX.state,
            &mut resources.BLUETOOTH,
            &mut resources.LED,
            &mut resources.USB,
        );
    }

    #[idle]
    fn idle() -> ! {
        loop {
            cortex_m::asm::wfi();
        }
    }

    #[interrupt(priority = 1, resources = [USB])]
    fn USB_LP() {
        resources.USB.interrupt()
    }

    #[interrupt(binds = DMA1_CHANNEL2, resources = [LED])]
    fn led_tx() {
        resources.LED.serial.tx_interrupt()
    }

    #[interrupt(binds = DMA1_CHANNEL3, resources = [LED, KEYBOARD])]
    fn led_rx() {
        resources.LED.poll()
    }

    #[interrupt(binds = DMA1_CHANNEL6, resources = [BLUETOOTH, KEY_MATRIX, LED, KEYBOARD])]
    fn bluetooth_rx() {
        resources
            .BLUETOOTH
            .poll(&mut resources.LED, &mut resources.KEYBOARD)
    }

    #[interrupt(binds = DMA1_CHANNEL7, resources = [BLUETOOTH])]
    fn bluetooth_tx() {
        resources.BLUETOOTH.serial.tx_interrupt()
    }

    #[interrupt(resources = [EXTI])]
    fn EXTI9_5() {
        // this (plus other exti) are key presses,
        // maybe use them instead of timer based scanning?

        // maybe only clear set bits? or ones from 9-5?
        unsafe { resources.EXTI.pr.write(|w| w.bits(0xffff)) };
    }
    #[interrupt(resources = [EXTI])]
    fn EXTI0() {
        unsafe { resources.EXTI.pr.write(|w| w.bits(0xffff)) };
    }
    #[interrupt(resources = [EXTI])]
    fn EXTI1() {
        unsafe { resources.EXTI.pr.write(|w| w.bits(0xffff)) };
    }
    #[interrupt(resources = [EXTI])]
    fn EXTI2() {
        unsafe { resources.EXTI.pr.write(|w| w.bits(0xffff)) };
    }
    #[interrupt(resources = [EXTI])]
    fn EXTI3() {
        unsafe { resources.EXTI.pr.write(|w| w.bits(0xffff)) };
    }
    #[interrupt(resources = [EXTI])]
    fn EXTI4() {
        unsafe { resources.EXTI.pr.write(|w| w.bits(0xffff)) };
    }
};
