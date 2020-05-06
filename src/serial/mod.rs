pub mod bluetooth_usart;
pub mod led_usart;

use crate::protocol::MsgType;
use core::convert::Infallible;
use core::marker::Unsize;

pub struct Serial<USART, T: 'static>
where
    USART: DmaUsart,
{
    pub usart: USART,
    send_buffer: &'static mut T,
    pub send_buffer_pos: u16,
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

pub struct Transfer<T: 'static> {
    pub buffer: &'static mut T,
    receive_stage: ReceiveStage,
}

impl<T> Transfer<T>
where
    T: Unsize<[u8]>,
{
    pub fn poll<USART>(&mut self, usart: &mut USART) -> nb::Result<(), Infallible>
    where
        USART: DmaUsart,
    {
        if usart.is_receive_pending() {
            match self.receive_stage {
                ReceiveStage::Header => {
                    let buffer: &[u8] = self.buffer;
                    self.receive_stage = ReceiveStage::Body;
                    usart.receive(
                        u16::from(buffer[1]),
                        buffer.as_ptr() as u32 + u32::from(HEADER_SIZE),
                    );

                    Err(nb::Error::WouldBlock)
                }
                ReceiveStage::Body => Ok(()),
            }
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    pub fn finish(self) -> &'static mut T {
        self.buffer
    }
}

impl<USART, T> Serial<USART, T>
where
    USART: DmaUsart,
    T: Unsize<[u8]>,
{
    pub fn new(usart: USART, send_buffer: &'static mut T) -> Serial<USART, T> {
        Serial {
            usart,
            send_buffer,
            send_buffer_pos: 0,
        }
    }

    pub fn receive(&mut self, recv_buffer: &'static mut T) -> Transfer<T> {
        {
            let buffer: &mut [u8] = recv_buffer;
            self.usart.receive(HEADER_SIZE, buffer.as_mut_ptr() as u32);
        }

        Transfer {
            buffer: recv_buffer,
            receive_stage: ReceiveStage::Header,
        }
    }

    pub fn send(
        &mut self,
        message_type: MsgType,
        operation: u8, // TODO: make this typed?
        data: &[u8],
    ) -> nb::Result<(), Infallible> {
        let tx_len = 3 + data.len() as u16;
        let send_buffer: &mut [u8] = self.send_buffer;
        if self.usart.is_send_ready() && self.send_buffer_pos + tx_len < send_buffer.len() as u16 {
            // TODO: put this into buffer, but then increase buffer offset
            // keep counter, use counter when calling send()
            let pos = self.send_buffer_pos as usize;
            send_buffer[pos] = message_type as u8;
            send_buffer[pos + 1] = 1 + data.len() as u8;
            send_buffer[pos + 2] = operation;
            send_buffer[pos + 3..pos + tx_len as usize].clone_from_slice(data);

            self.send_buffer_pos += tx_len;

            self.usart
                .send(send_buffer.as_ptr() as u32, self.send_buffer_pos);

            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    pub fn tx_interrupt(&mut self) {
        self.send_buffer_pos = 0;
        self.usart.tx_interrupt();
    }
}
