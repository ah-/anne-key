use core::fmt::Write;
use cortex_m_semihosting::hio;
use embedded_hal::digital::OutputPin;
use rtfm::Threshold;
use stm32l151::DMA1;
use hal::gpio::{Input, Output};
use hal::gpio::gpioc::PC15;
use super::protocol::{Message, MsgType, LedOp};
use super::serial::Serial;
use super::serial::led_usart::LedUsart;


pub struct Led<'a> {
    pub serial: Serial<'a, LedUsart>,
    pub pc15: PC15<Output>,
}

impl<'a> Led<'a> {
    pub fn new(serial: Serial<'a, LedUsart>, pc15: PC15<Input>) -> Led<'a> {
        Led {
            serial: serial,
            pc15: pc15.into_output().pull_up(),
        }
    }

    pub fn on(&mut self) {
        self.pc15.set_high();
    }

    pub fn off(&mut self) {
        self.pc15.set_low();
    }

    // next_* cycles through themes/brightness/speed
    pub fn next_theme(&mut self, dma1: &mut DMA1, stdout: &mut Option<hio::HStdout>) {
        self.serial.send(MsgType::Led, LedOp::ConfigCmd as u8, &[1, 0, 0], dma1, stdout);
    }

    pub fn next_brightness(&mut self, dma1: &mut DMA1, stdout: &mut Option<hio::HStdout>) {
        self.serial.send(MsgType::Led, LedOp::ConfigCmd as u8, &[0, 1, 0], dma1, stdout);
    }

    pub fn next_animation_speed(&mut self, dma1: &mut DMA1, stdout: &mut Option<hio::HStdout>) {
        self.serial.send(MsgType::Led, LedOp::ConfigCmd as u8, &[0, 0, 1], dma1, stdout);
    }

    pub fn set_theme(&mut self, theme: u8, dma1: &mut DMA1, stdout: &mut Option<hio::HStdout>) {
        self.serial.send(MsgType::Led, LedOp::ThemeMode as u8, &[theme], dma1, stdout);
    }

    pub fn send_keys(&mut self, keys: &[u8], dma1: &mut DMA1, stdout: &mut Option<hio::HStdout>) {
        self.serial.send(MsgType::Led, LedOp::Key as u8, keys, dma1, stdout);
    }

    pub fn send_music(&mut self, keys: &[u8], dma1: &mut DMA1, stdout: &mut Option<hio::HStdout>) {
        self.serial.send(MsgType::Led, LedOp::Music as u8, keys, dma1, stdout);
    }

    pub fn get_theme_id(&mut self, dma1: &mut DMA1, stdout: &mut Option<hio::HStdout>) {
        // responds with with [ThemeId]
        self.serial.send(MsgType::Led, LedOp::GetThemeId as u8, &[], dma1, stdout);
    }

    pub fn receive(message: &Message, stdout: &mut Option<hio::HStdout>) {
        match message.msg_type {
            MsgType::Led => {
                match LedOp::from(message.operation) {
                    LedOp::AckThemeMode => {
                        // data: [theme id]
                        //debug!(stdout, "Led AckThemeMode {:?}", message.data).ok();
                    },
                    LedOp::AckConfigCmd => {
                        // data: [theme id, brightness, animation speed]
                        //debug!(stdout, "Led AckConfigCmd {:?}", message.data).ok();
                    },
                    _ => {
                        debug!(stdout, "lmsg: {:?} {} {:?}", message.msg_type, message.operation, message.data).ok();
                    }
                }
            },
            _ => {
                debug!(stdout, "lmsg: {:?} {} {:?}", message.msg_type, message.operation, message.data).ok();
            }
        }
    }
}

pub fn rx(_t: &mut Threshold, mut r: super::DMA1_CHANNEL3::Resources) {
    let stdout: &mut Option<hio::HStdout> = &mut r.STDOUT;
    let callback = |msg: &Message| Led::receive(msg, stdout);
    r.LED.serial.receive(&mut r.DMA1, callback);
}

pub fn tx(_t: &mut Threshold, mut r: super::DMA1_CHANNEL2::Resources) {
    r.LED.serial.tx_interrupt(&mut r.DMA1);
}
