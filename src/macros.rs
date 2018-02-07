// TODO: make stdout a global, borrow it temporarily to print, remove all the stdout passing around
// tOdo: make debug behaviour a compile time cfg
// TOdO: install exception handler to deal with hio semiosting not being available

#[macro_export]
macro_rules! debug {
    ($odst: expr, $($arg: tt)*) => {
        match *$odst {
            Some(ref mut dst) => write!(dst, $($arg)*),
            None => Ok(())
        }
    };
}
