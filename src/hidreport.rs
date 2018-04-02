use core::slice;

#[repr(packed)]
#[derive(Default)]
pub struct HidReport {
    pub modifiers: u8,
    _unused: u8,
    pub keys: [u8; 6],
}

impl HidReport {
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            let p: *const HidReport = self;
            slice::from_raw_parts(p as *const u8, 8)
        }
    }
}
