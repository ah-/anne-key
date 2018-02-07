use core::fmt::Write;
use cortex_m_semihosting::hio;
use rtfm::Threshold;
use stm32l151::{DMA1, GPIOA, GPIOC};
use super::protocol::{Message, MsgType, LedOp};
use super::serial::Serial;
use super::serial::led_usart::LedUsart;


pub struct Led<'a> {
    pub serial: Serial<'a, LedUsart>,
}

impl<'a> Led<'a> {
    pub fn new(serial: Serial<'a, LedUsart>, gpioc: &mut GPIOC) -> Led<'a> {
        gpioc.moder.modify(|_, w| unsafe {
            w.moder15().bits(1)
        });
        gpioc.pupdr.modify(|_, w| unsafe {
            w.pupdr15().bits(0b01)
        });

        Led {
            serial: serial,
        }
    }

    pub fn on(&self, gpioc: &mut GPIOC) {
        gpioc.odr.modify(|_, w| w.odr15().set_bit());
    }

    pub fn off(&self, gpioc: &mut GPIOC) {
        gpioc.odr.modify(|_, w| w.odr15().clear_bit());
    }

    pub fn next_theme(&mut self, dma1: &mut DMA1, stdout: &mut Option<hio::HStdout>, gpioa: &mut GPIOA) {
        self.serial.send(MsgType::Led, LedOp::ConfigCmd as u8, &[1, 0, 0], dma1, stdout, gpioa);
    }

    pub fn next_animation_speed(&mut self, dma1: &mut DMA1, stdout: &mut Option<hio::HStdout>, gpioa: &mut GPIOA) {
        self.serial.send(MsgType::Led, LedOp::ConfigCmd as u8, &[0, 1, 0], dma1, stdout, gpioa);
    }

    pub fn next_brightness(&mut self, dma1: &mut DMA1, stdout: &mut Option<hio::HStdout>, gpioa: &mut GPIOA) {
        self.serial.send(MsgType::Led, LedOp::ConfigCmd as u8, &[0, 0, 1], dma1, stdout, gpioa);
    }

    pub fn set_theme(&mut self, theme: u8, dma1: &mut DMA1, stdout: &mut Option<hio::HStdout>, gpioa: &mut GPIOA) {
        self.serial.send(MsgType::Led, LedOp::ThemeMode as u8, &[theme], dma1, stdout, gpioa);
    }

    pub fn send_keys(&mut self, keys: &[u8], dma1: &mut DMA1, stdout: &mut Option<hio::HStdout>, gpioa: &mut GPIOA) {
        self.serial.send(MsgType::Led, LedOp::Key as u8, keys, dma1, stdout, gpioa);
    }

    pub fn send_music(&mut self, keys: &[u8], dma1: &mut DMA1, stdout: &mut Option<hio::HStdout>, gpioa: &mut GPIOA) {
        self.serial.send(MsgType::Led, LedOp::Music as u8, keys, dma1, stdout, gpioa);
    }

    pub fn get_theme_id(&mut self, dma1: &mut DMA1, stdout: &mut Option<hio::HStdout>, gpioa: &mut GPIOA) {
        // responds with AckConfigCmd with [ThemeId]
        self.serial.send(MsgType::Led, LedOp::GetThemeId as u8,
                         &[], dma1, stdout, gpioa);
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
    r.LED.serial.receive(&mut r.DMA1, &mut r.GPIOA, callback);
}

pub fn tx(_t: &mut Threshold, mut r: super::DMA1_CHANNEL2::Resources) {
    r.LED.serial.tx_interrupt(&mut r.DMA1);
}

pub fn usart3(_t: &mut Threshold, mut r: super::USART3::Resources) {
    // not quite sure when and why this interrupt happens
    debug!(r.STDOUT, "usart3").ok();
}
