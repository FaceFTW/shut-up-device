#![no_std]
#![no_main]
#![feature(inline_const)]

mod display;

use core::{
    cmp::{max, min},
    ops::Range,
};

use display::SSD1306Display;
use panic_halt as _;
pub use unwrap_infallible::UnwrapInfallible as _;

// #[panic_handler]
// fn panic(info: &core::panic::PanicInfo) -> ! {
//     // disable interrupts - firmware has panicked so no ISRs should continue running
//     avr_device::interrupt::disable();

//     // get the peripherals so we can access serial and the LED.
//     //
//     // SAFETY: Because main() already has references to the peripherals this is an unsafe
//     // operation - but because no other code can run after the panic handler was called,
//     // we know it is okay.
//     let dp = unsafe { arduino_hal::Peripherals::steal() };
//     let pins = arduino_hal::pins!(dp);
//     let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

//     // Print out panic location
//     ufmt::uwriteln!(&mut serial, "Firmware panic!\r").unwrap_infallible();
//     if let Some(loc) = info.location() {
//         ufmt::uwriteln!(
//             &mut serial,
//             "  At {}:{}:{}\r",
//             loc.file(),
//             loc.line(),
//             loc.column(),
//         )
//         .unwrap_infallible();
//     }

//     // Blink LED rapidly
//     let mut led = pins.d13.into_output();
//     led.set_high();
//     loop {} //So it's infallible type ret.
// }

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    //Setup MCU utils
    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        400000,
    );
    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    // let mut led = pins.d13.into_output();
    let mic = pins.a0.into_analog_input(&mut adc);

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
    display.clear(&mut i2c).expect("init failed!!");
    display.write_str(&mut i2c, "Hello World");
    display
        .set_cursor(&mut i2c, 0, 1)
        .expect("cursor_set_failed");
    display.write_str(&mut i2c, "0123456789");
    arduino_hal::delay_ms(3000);

    //Since we are in no_std land, allocate a buffer for the display strings, then we can use ufmt
    //heapless crate is based as hell
    let mut oled_buf1: heapless::String<32> = heapless::String::new();
    let mut oled_buf2: heapless::String<32> = heapless::String::new();

    const ENUM_RANGE: Range<u16> = 0..1000;
    loop {
        //sure, we could do async but that's a headache
        //Timings here are faster than most human reaction speeds, so we should be fine
        let (min_ms, max_ms) = ENUM_RANGE.fold((1024, 0), |(min_val, max_val), _| {
            let measured = arduino_hal::Adc::read_blocking(&mut adc, &mic);
            (min(min_val, measured), max(max_val, measured))
        });
        let delta = max_ms - min_ms;

        ufmt::uwriteln!(
            &mut serial,
            "Min:{}, Max:{}, Delta:{}\r",
            min_ms,
            max_ms,
            delta
        )
        .unwrap_infallible();

        ufmt::uwrite!(&mut oled_buf1, "Min: {}, Max: {}\n", min_ms, max_ms).unwrap();
        ufmt::uwrite!(&mut oled_buf2, "Delta: {}\n", delta).unwrap();

        display.set_cursor(&mut i2c, 0, 0).expect("oopsies");
        display.write_str(&mut i2c, &oled_buf1.as_str());
        display.write_str(&mut i2c, &oled_buf2.as_str());

        // led.toggle();
        arduino_hal::delay_ms(1000);
    }
}
