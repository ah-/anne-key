use core::fmt::Write;
use rtfm::Threshold;

pub struct Led {}

impl Led {
    pub const fn new() -> Led {
        Led {}
    }

    pub fn init(&self, p: &super::init::Peripherals) {
        p.GPIOB.moder.modify(|_, w| unsafe { w.moder11().bits(0b10) });
        p.GPIOB.pupdr.modify(|_, w| unsafe { w.pupdr11().bits(0b01) });
        p.GPIOB.afrh.modify(|_, w| unsafe { w.afrh11().bits(7) });

        p.RCC.apb1enr.modify(|_, w| w.usart3en().set_bit());
        p.RCC.ahbenr.modify(|_, w| w.dma1en().set_bit());

        p.USART3.brr.modify(|_, w| unsafe { w.bits(417) });
        p.USART3.cr3.modify(|_, w| w.dmar().set_bit());
        p.USART3.cr1.modify(|_, w| {
            w.rxneie().set_bit()
             //.tcie().set_bit()
             .idleie().set_bit()
             .re().set_bit()
             .te().set_bit()
             .ue().set_bit()
        });

        p.DMA1.cpar3.write(|w| unsafe { w.pa().bits(0x4000_4804) });

        //p.DMA1.cmar3.write(|w| {
            //unsafe {
                //w.ma().bits(r.RECV_BUFFER.as_mut_ptr() as u32) 
            //}
        //});

        p.DMA1.ccr3.modify(|_, w| {
            unsafe {
                w.pl().bits(2);
            }
            w.minc().set_bit().tcie().set_bit().en().clear_bit()
        });
    }
}


pub fn receive(_t: &mut Threshold, r: super::USART3::Resources) {
    if r.USART3.sr.read().rxne().bit_is_set() {
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
    write!(r.STDOUT, "x").unwrap()
}
