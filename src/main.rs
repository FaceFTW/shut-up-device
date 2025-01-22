#![no_std]
#![no_main]
#![feature(inline_const)]

mod display;

use display::SSD1306Display;
use panic_halt as _;
pub use unwrap_infallible::UnwrapInfallible as _;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        50000,
    );

    let mut led = pins.d13.into_output();

    let mut display = SSD1306Display::new(&mut i2c).expect("no");
    display.write_str(&mut i2c, "Hello World");
    display.set_cursor(&mut i2c, 0, 1);
    display.write_str(&mut i2c, "0123456789");

    loop {
        led.toggle();
        arduino_hal::delay_ms(1000);
    }
}
