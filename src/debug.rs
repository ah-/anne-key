// TODO: install exception handler to deal with hio semihosting not being available
// and just ignore bkpts if no debugger attached

#[cfg(feature = "use_semihosting")]
#[macro_export]
macro_rules! debug {
    ($($arg: tt)*) => {
        match hio::hstdout() {
            Ok(ref mut stdout) => write!(stdout, $($arg)*),
            _ => Ok(())
        };
    }
}

#[cfg(not(feature = "use_semihosting"))]
#[macro_export]
macro_rules! debug {
    ($($arg: tt)*) => {
        {
            let res: Result<(), ()> = Ok(());
            res
        }
    }
}
