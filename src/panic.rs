use core::panic::PanicInfo;
use core::fmt::Write;

struct PanicWriter {}

impl core::fmt::Write for PanicWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        crate::isviewer::text_out(s);
        Ok(())
    }
}

fn panic_print(args: core::fmt::Arguments) {
    // If we used println!, we'd risk not being able to open the mutex. So instead we'll use a local
    // instance here
    let mut writer = PanicWriter { };
    writer.write_fmt(args).unwrap();
}

macro_rules! panic_print {
    ($($arg:tt)*) => (panic_print(format_args!($($arg)*)));
}

macro_rules! panic_println {
    () => (panic_print!("\n\n"));
    ($($arg:tt)*) => (panic_print!("\n{}\n\n", format_args!($($arg)*)));
}

#[panic_handler]
#[no_mangle]
fn panic(_info: &PanicInfo<'_>) -> ! {
    panic_println!("{}", _info);

    loop {}
}
