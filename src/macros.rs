#[macro_export]
macro_rules! debug {
    ($odst: expr, $($arg: tt)*) => {
        match *$odst {
            Some(ref mut dst) => write!(dst, $($arg)*),
            None => Ok(())
        }
    };
}
