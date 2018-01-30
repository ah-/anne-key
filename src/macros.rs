#[macro_export]
macro_rules! debug {
    ($odst: expr, $($arg: tt)*) => {
        match $odst {
            &mut Some(ref mut dst) => write!(dst, $($arg)*),
            &mut None => Ok(())
        }
    };
}
