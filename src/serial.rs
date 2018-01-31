use core::fmt::Write;
use cortex_m_semihosting::hio;
use stm32l151::{DMA1, GPIOA, RCC, USART2, USART3};
use super::protocol::{Message, MsgType};


pub struct Serial<'a, USART>
    where USART: DmaUsart
{
    usart: USART,
    receive_stage: ReceiveStage,
    send_buffer: &'a mut[u8; 0x10],
    receive_buffer: &'a mut[u8; 0x10],
}

enum ReceiveStage {
    Header,
    Body,
}

impl<'a, USART> Serial<'a, USART>
    where USART: DmaUsart
{
    pub fn new(mut usart: USART, dma: &mut DMA1, gpioa: &mut GPIOA, rcc: &mut RCC,
               send_buffer: &'a mut[u8; 0x10], receive_buffer: &'a mut[u8; 0x10]) -> Serial<'a, USART> {
        let send_ptr = send_buffer.as_mut_ptr() as u32;
        let receive_ptr = receive_buffer.as_mut_ptr() as u32;
        usart.init(dma, gpioa, rcc, send_ptr, receive_ptr);
        usart.receive(dma, gpioa, 2, receive_ptr);
        Serial {
            usart: usart,
            receive_stage: ReceiveStage::Header,
            send_buffer: send_buffer,
            receive_buffer: receive_buffer,
        }
    }

    pub fn receive<F>(&mut self, dma: &mut DMA1, gpioa: &mut GPIOA, callback: F)
        where F: FnOnce(&Message)
    {
        if self.usart.receive_pending(dma) {
            match self.receive_stage {
                ReceiveStage::Header => {
                    self.receive_stage = ReceiveStage::Body;
                    self.usart.receive(dma, gpioa, self.receive_buffer[1] as u16, self.receive_buffer.as_mut_ptr() as u32 + 2);
                }
                ReceiveStage::Body => {
                    self.receive_stage = ReceiveStage::Header;
                    
                    {
                        let message = Message {
                            msg_type: self.receive_buffer[0],
                            operation: self.receive_buffer[2],
                            data: &self.receive_buffer[3..3 + self.receive_buffer[1] as usize - 1],
                        };
                        match (message.msg_type, message.operation) {
                            (6, 170) => {
                                // Wakeup acknowledged, send data
                                unsafe { dma.cndtr7.modify(|_, w| w.ndt().bits(0xb)) };
                                dma.ccr7.modify(|_, w| w.en().set_bit());
                            },
                            _ => callback(&message)
                        }
                    }

                    self.usart.receive(dma, gpioa, 2, self.receive_buffer.as_mut_ptr() as u32);
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
        if self.usart.send_ready(dma) {
            self.send_buffer[0] = message_type as u8;
            self.send_buffer[1] = data.len() as u8;
            self.send_buffer[2] = operation;
            self.send_buffer[3..3 + data.len()].clone_from_slice(data);

            self.usart.send(dma, &mut gpioa);
            self.receive_stage = ReceiveStage::Header;
        } else {
            // TODO: return an error instead
            // saying we're busy
            // using https://docs.rs/nb/0.1.1/nb/
            debug!(stdout, "tx busy").ok();
        }
    }

    pub fn tx_interrupt(&self, dma: &mut DMA1) {
        dma.ifcr.write(|w| w.cgif7().set_bit());
        dma.ccr7.modify(|_, w| w.en().clear_bit());
    }
}

pub trait DmaUsart {
    fn init(&mut self, dma: &DMA1, gpioa: &mut GPIOA, rcc: &mut RCC, send_ptr: u32, receive_ptr: u32);
    fn receive_pending(&self, dma: &DMA1) -> bool;
    fn receive(&self, dma: &mut DMA1, gpioa: &mut GPIOA, length: u16, buffer: u32);
    fn send_ready(&self, dma: &DMA1) -> bool;
    fn send(&self, dma: &mut DMA1, gpioa: &mut GPIOA);
}

impl DmaUsart for USART2 {
    fn init(&mut self, dma: &DMA1, gpioa: &mut GPIOA, rcc: &mut RCC, send_ptr: u32, receive_ptr: u32) {
        gpioa.moder.modify(|_, w| unsafe {
            w.moder1().bits(1)
             .moder2().bits(0b10)
             .moder3().bits(0b10)
        });
        gpioa.pupdr.modify(|_, w| unsafe {
            w.pupdr1().bits(0b01)
             .pupdr2().bits(0b01)
             .pupdr3().bits(0b01)
        });
        gpioa.afrl.modify(|_, w| unsafe { w.afrl2().bits(7).afrl3().bits(7) });
        gpioa.odr.modify(|_, w| w.odr1().clear_bit());

        rcc.apb1enr.modify(|_, w| w.usart2en().set_bit());
        rcc.ahbenr.modify(|_, w| w.dma1en().set_bit());

        self.brr.modify(|_, w| unsafe { w.bits(417) });
        self.cr3.modify(|_, w| w.dmat().set_bit()
                                      .dmar().set_bit());
        self.cr1.modify(|_, w| {
            w.rxneie().set_bit()
             .re().set_bit()
             .te().set_bit()
             .ue().set_bit()
        });

        dma.cpar6.write(|w| unsafe { w.pa().bits(0x4000_4404) });
        dma.cmar6.write(|w| unsafe { w.ma().bits(receive_ptr) });
        dma.ccr6.modify(|_, w| {
            unsafe {
                w.pl().bits(2);
            }
            w.minc().set_bit()
             .tcie().set_bit()
        });

        dma.cpar7.write(|w| unsafe { w.pa().bits(0x4000_4404) });
        dma.cmar7.write(|w| unsafe { w.ma().bits(send_ptr) });
        dma.cndtr7.modify(|_, w| unsafe { w.ndt().bits(0x0) });
        dma.ccr7.modify(|_, w| {
            unsafe {
                w.pl().bits(2);
            }
            w.minc().set_bit()
             .dir().set_bit()
             .tcie().set_bit()
             .en().clear_bit()
        });
    }

    fn receive_pending(&self, dma: &DMA1) -> bool {
        dma.isr.read().tcif6().bit_is_set()
    }

    fn receive(&self, dma: &mut DMA1, gpioa: &mut GPIOA, length: u16, buffer: u32) {
        // wakeup complete, reset pa1
        gpioa.bsrr.write(|w| w.br1().set_bit());

        dma.ifcr.write(|w| w.cgif6().set_bit());
        dma.ccr6.modify(|_, w| { w.en().clear_bit() });
        dma.cmar6.write(|w| unsafe { w.ma().bits(buffer) });
        dma.cndtr6.modify(|_, w| unsafe { w.ndt().bits(length) });
        dma.ccr6.modify(|_, w| { w.en().set_bit() });
    }

    fn send_ready(&self, dma: &DMA1) -> bool {
        dma.cndtr7.read().ndt().bits() == 0
    }

    fn send(&self, dma: &mut DMA1, gpioa: &mut GPIOA) {
        // Don't actually send anything yet, just enqueue and wait for wakeup package
        dma.ccr6.modify(|_, w| { w.en().clear_bit() });
        //dma.cmar6.write(|w| unsafe { w.ma().bits(self.receive_buffer.as_mut_ptr() as u32) });
        dma.cndtr6.modify(|_, w| unsafe { w.ndt().bits(2) });
        dma.ccr6.modify(|_, w| { w.en().set_bit() });

        gpioa.odr.modify(|_, w| w.odr1().clear_bit());
        gpioa.odr.modify(|_, w| w.odr1().set_bit());
    }
}

impl DmaUsart for USART3 {
    fn init(&mut self, dma: &DMA1, gpioa: &mut GPIOA, rcc: &mut RCC, send_ptr: u32, receive_ptr: u32) {
    }

    fn receive_pending(&self, dma: &DMA1) -> bool {
        dma.isr.read().tcif3().bit_is_set()
    }

    fn receive(&self, dma: &mut DMA1, gpioa: &mut GPIOA, length: u16, buffer: u32) {
        dma.ifcr.write(|w| w.cgif3().set_bit());
        dma.ccr3.modify(|_, w| { w.en().clear_bit() });
        dma.cmar3.write(|w| unsafe { w.ma().bits(buffer) });
        dma.cndtr3.modify(|_, w| unsafe { w.ndt().bits(length) });
        dma.ccr3.modify(|_, w| { w.en().set_bit() });
    }

    fn send_ready(&self, dma: &DMA1) -> bool {
        dma.cndtr2.read().ndt().bits() == 0
    }

    fn send(&self, dma: &mut DMA1, gpioa: &mut GPIOA) {
        unsafe { dma.cndtr2.modify(|_, w| w.ndt().bits(0xb)) };
        dma.ccr2.modify(|_, w| w.en().set_bit());
    }
}
