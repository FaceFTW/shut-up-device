#![no_std]
#![no_main]
#![feature(inline_const)]

mod display;

use display::SSD1306Display;
// use panic_halt as _;
pub use unwrap_infallible::UnwrapInfallible as _;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // disable interrupts - firmware has panicked so no ISRs should continue running
    avr_device::interrupt::disable();

    // get the peripherals so we can access serial and the LED.
    //
    // SAFETY: Because main() already has references to the peripherals this is an unsafe
    // operation - but because no other code can run after the panic handler was called,
    // we know it is okay.
    let dp = unsafe { arduino_hal::Peripherals::steal() };
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    // Print out panic location
    ufmt::uwriteln!(&mut serial, "Firmware panic!\r").unwrap_infallible();
    if let Some(loc) = info.location() {
        ufmt::uwriteln!(
            &mut serial,
            "  At {}:{}:{}\r",
            loc.file(),
            loc.line(),
            loc.column(),
        )
        .unwrap_infallible();
    }

    // Blink LED rapidly
    let mut led = pins.d13.into_output();
    led.set_high();
    loop {} //So it's infallible type ret.
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        400000,
    );

    let mut led = pins.d13.into_output();

    let mut display = match SSD1306Display::new(&mut i2c) {
        Ok(disp) => disp,
        Err(err) => match err {
            arduino_hal::i2c::Error::ArbitrationLost => panic!("arb lost"),
            arduino_hal::i2c::Error::AddressNack => panic!("addr nack"),
            arduino_hal::i2c::Error::DataNack => panic!("data nack"),
            arduino_hal::i2c::Error::BusError => panic!("bus err"),
            arduino_hal::i2c::Error::Unknown => panic!("uhhhh"),
        },
    };
    display.clear(&mut i2c);
    display.write_str(&mut i2c, "Hello World");
    display.set_cursor(&mut i2c, 0, 1);
    display.write_str(&mut i2c, "0123456789");

    loop {
        led.toggle();
        arduino_hal::delay_ms(1000);
    }
}
