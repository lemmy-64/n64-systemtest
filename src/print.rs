use core::fmt::Write;
use spinning_top::Spinlock;
use crate::FramebufferConsole;

/// Global Text Writer. This is a static mut as we don't use Threading. If we ever support that,
/// we'll need to protect this with e.g. a spinlock
static TEXT_WRITER: Spinlock<Writer> = Spinlock::new(Writer {});

pub struct Writer {

}

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        // Write to terminal (emulator)
        super::isviewer::text_out(s);

        // Write to framebuffer_console (so that it is printed in the framebuffer)
        FramebufferConsole::instance().lock().append(s);

        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    TEXT_WRITER.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::print::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

