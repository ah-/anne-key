use core::slice;

#[repr(packed)]
pub struct HidReport {
    pub modifiers: u8,
    _unused: u8,
    pub keys: [u8; 6]
}

impl HidReport {
    pub fn new() -> HidReport {
        HidReport {
            modifiers: 0,
            _unused: 0,
            keys: [0; 6],
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe { 
            let p : *const HidReport = self;
            slice::from_raw_parts(p as *const u8, 8)
        }
    }
}
