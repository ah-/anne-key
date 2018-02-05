pub mod bluetooth_usart;
pub mod led_usart;

use core::fmt::Write;
use cortex_m_semihosting::hio;
use stm32l151::{DMA1, GPIOA};
use super::protocol::{Message, MsgType};


pub struct Serial<'a, USART>
    where USART: DmaUsart
{
    usart: USART,
    receive_stage: ReceiveStage,
    send_buffer: &'a mut[u8; 0x10],
    receive_buffer: &'a mut[u8; 0x10],
}

pub trait DmaUsart {
    // TODO: naming of these isn't quite perfect
    fn is_receive_pending(&self, dma: &DMA1) -> bool;
    fn receive(&self, dma: &mut DMA1, gpioa: &mut GPIOA, length: u16, buffer: u32);
    fn is_send_ready(&self, dma: &DMA1) -> bool;
    fn send(&self, dma: &mut DMA1, gpioa: &mut GPIOA, buffer: u32, len: u16);
    fn tx_interrupt(&self, dma: &mut DMA1);
}

enum ReceiveStage {
    Header,
    Body,
}

const HEADER_SIZE: u16 = 2;

impl<'a, USART> Serial<'a, USART>
    where USART: DmaUsart
{
    pub fn new(usart: USART, dma: &mut DMA1, gpioa: &mut GPIOA,
               buffers: &'a mut[[u8; 0x10]; 2]) -> Serial<'a, USART> {
        let (send_buffer, receive_buffer) = buffers.split_at_mut(1);
        let receive_ptr = receive_buffer[0].as_mut_ptr() as u32;

        usart.receive(dma, gpioa, HEADER_SIZE, receive_ptr);

        Serial {
            usart: usart,
            receive_stage: ReceiveStage::Header,
            send_buffer: &mut send_buffer[0],
            receive_buffer: &mut receive_buffer[0],
        }
    }

    pub fn receive<F>(&mut self, dma: &mut DMA1, gpioa: &mut GPIOA, callback: F)
        where F: FnOnce(&Message)
    {
        if self.usart.is_receive_pending(dma) {
            match self.receive_stage {
                ReceiveStage::Header => {
                    self.receive_stage = ReceiveStage::Body;
                    self.usart.receive(dma, gpioa, u16::from(self.receive_buffer[1]),
                        self.receive_buffer.as_mut_ptr() as u32 + u32::from(HEADER_SIZE));
                }
                ReceiveStage::Body => {
                    self.receive_stage = ReceiveStage::Header;
                    
                    {
                        let message = Message {
                            msg_type: MsgType::from(self.receive_buffer[0]),
                            operation: self.receive_buffer[2],
                            data: &self.receive_buffer[3..3 + self.receive_buffer[1] as usize - 1],
                        };
                        match (message.msg_type, message.operation) {
                            (MsgType::Ble, 170) => {
                                // Wakeup acknowledged, send data
                                unsafe { dma.cndtr7.modify(|_, w| w.ndt().bits(0xb)) };
                                dma.ccr7.modify(|_, w| w.en().set_bit());
                            },
                            _ => callback(&message)
                        }
                    }

                    self.usart.receive(dma, gpioa, HEADER_SIZE, self.receive_buffer.as_mut_ptr() as u32);
                }
            }
        }
    }

    pub fn send(
        &mut self,
        message_type: MsgType,
        operation: u8, // TODO: make this typed?
        data: &[u8],
        dma: &mut DMA1,
        stdout: &mut Option<hio::HStdout>,
        mut gpioa: &mut GPIOA) {
        if self.usart.is_send_ready(dma) {
            self.send_buffer[0] = message_type as u8;
            self.send_buffer[1] = 1 + data.len() as u8;
            self.send_buffer[2] = operation;
            self.send_buffer[3..3 + data.len()].clone_from_slice(data);

            self.usart.send(dma, &mut gpioa, self.send_buffer.as_mut_ptr() as u32, 3 + data.len() as u16);
            self.receive_stage = ReceiveStage::Header;
        } else {
            // TODO: return an error instead
            // saying we're busy
            // using https://docs.rs/nb/0.1.1/nb/
            debug!(stdout, "tx busy").ok();
        }
    }

    pub fn tx_interrupt(&self, dma: &mut DMA1) {
        self.usart.tx_interrupt(dma)
    }
}
