pub mod bluetooth_usart;
pub mod led_usart;

use nb;
use super::protocol::{Message, MsgType};


pub struct Serial<'a, USART>
    where USART: DmaUsart
{
    usart: USART,
    receive_stage: ReceiveStage,
    send_buffer: &'a mut[u8; 0x20],
    receive_buffer: &'a mut[u8; 0x20],
    send_buffer_pos: u16,
}

pub trait DmaUsart {
    // TODO: naming of these isn't quite perfect
    // TODO: better types?
    fn is_receive_pending(&mut self) -> bool;
    fn receive(&mut self, length: u16, buffer: u32);
    fn is_send_ready(&mut self) -> bool;
    fn send(&mut self, buffer: u32, len: u16);
    fn ack_wakeup(&mut self);
    fn tx_interrupt(&mut self);
}

enum ReceiveStage {
    Header,
    Body,
}

const HEADER_SIZE: u16 = 2;

impl<'a, USART> Serial<'a, USART>
    where USART: DmaUsart
{
    pub fn new(mut usart: USART, buffers: &'a mut[[u8; 0x20]; 2])
        -> Serial<'a, USART> {
        let (send_buffer, receive_buffer) = buffers.split_at_mut(1);
        let receive_ptr = receive_buffer[0].as_mut_ptr() as u32;

        usart.receive(HEADER_SIZE, receive_ptr);

        Serial {
            usart: usart,
            receive_stage: ReceiveStage::Header,
            send_buffer: &mut send_buffer[0],
            receive_buffer: &mut receive_buffer[0],
            send_buffer_pos: 0,
        }
    }

    pub fn receive<F>(&mut self, callback: F)
        where F: FnOnce(&Message)
    {
        if self.usart.is_receive_pending() {
            match self.receive_stage {
                ReceiveStage::Header => {
                    self.receive_stage = ReceiveStage::Body;
                    self.usart.receive(u16::from(self.receive_buffer[1]),
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
                                self.usart.ack_wakeup();
                                self.send_buffer_pos = 0;
                            },
                            _ => callback(&message)
                        }
                    }

                    self.usart.receive(HEADER_SIZE, self.receive_buffer.as_mut_ptr() as u32);
                }
            }
        }
    }

    pub fn send(
        &mut self,
        message_type: MsgType,
        operation: u8, // TODO: make this typed?
        data: &[u8]) -> nb::Result<(), !> {
        let tx_len = 3 + data.len() as u16;
        if self.usart.is_send_ready() && self.send_buffer_pos + tx_len < self.send_buffer.len() as u16 {
            // TODO: put this into buffer, but then increase buffer offset
            // keep counter, use counter when calling send()
            let pos = self.send_buffer_pos as usize;
            self.send_buffer[pos] = message_type as u8;
            self.send_buffer[pos + 1] = 1 + data.len() as u8;
            self.send_buffer[pos + 2] = operation;
            self.send_buffer[pos + 3..pos + tx_len as usize].clone_from_slice(data);

            self.send_buffer_pos += tx_len;

            self.usart.send(self.send_buffer.as_mut_ptr() as u32, self.send_buffer_pos);
            self.receive_stage = ReceiveStage::Header;

            return Ok(())
        } else {
            return Err(nb::Error::WouldBlock)
        }
    }

    pub fn tx_interrupt(&mut self) {
        self.send_buffer_pos = 0;
        self.usart.tx_interrupt();
    }
}
