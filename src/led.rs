use core::fmt::Write;
use rtfm::Threshold;
use stm32l151;

pub struct Led {
    usart: stm32l151::USART3,
}

impl Led {
    pub const fn new(usart: stm32l151::USART3) -> Led {
        Led {
            usart: usart
        }
    }

    pub fn init(&mut self, dma: &stm32l151::DMA1, gpiob: &mut stm32l151::GPIOB, rcc: &mut stm32l151::RCC) {
        gpiob.moder.modify(|_, w| unsafe { w.moder11().bits(0b10) });
        gpiob.pupdr.modify(|_, w| unsafe { w.pupdr11().bits(0b01) });
        gpiob.afrh.modify(|_, w| unsafe { w.afrh11().bits(7) });

        rcc.apb1enr.modify(|_, w| w.usart3en().set_bit());
        rcc.ahbenr.modify(|_, w| w.dma1en().set_bit());

        self.usart.brr.modify(|_, w| unsafe { w.bits(417) });
        self.usart.cr3.modify(|_, w| w.dmar().set_bit());
        self.usart.cr1.modify(|_, w| {
            w.rxneie().set_bit()
             //.tcie().set_bit()
             .idleie().set_bit()
             .re().set_bit()
             .te().set_bit()
             .ue().set_bit()
        });

        dma.cpar3.write(|w| unsafe { w.pa().bits(0x4000_4804) });

        //p.DMA1.cmar3.write(|w| {
            //unsafe {
                //w.ma().bits(r.RECV_BUFFER.as_mut_ptr() as u32) 
            //}
        //});

        dma.ccr3.modify(|_, w| {
            unsafe {
                w.pl().bits(2);
            }
            w.minc().set_bit().tcie().set_bit().en().clear_bit()
        });
    }

    pub fn receive(&mut self) {
        if self.usart.sr.read().rxne().bit_is_set() {
            //let bits = r.USART3.dr.read().bits() as u16;

            /*
            if bits != 9 && bits != 0 {
                **r.RECV_LEN = bits;
                r.DMA1.cndtr3.modify(|_, w| {
                    unsafe { w.ndt().bits(bits) }
                });
                r.DMA1.ccr3.modify(|_, w| {
                    w.en().set_bit()
                });
            }
            */
        }
        //write!(r.STDOUT, "x").unwrap()
    }
}


pub fn receive(_t: &mut Threshold, mut r: super::USART3::Resources) {
    r.LED.receive();
}
