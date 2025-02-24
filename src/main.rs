#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

mod buzzer;
mod display;

use core::{
    cmp::{max, min},
    ops::Range,
};

use arduino_hal::hal::usart::BaudrateExt;
use buzzer::{tone, BuzzerRegisters};
use display::SSD1306Display;
use panic_halt as _;
pub use unwrap_infallible::UnwrapInfallible as _;

//TODO Save this for the blog, panic handler too lorge
// #[panic_handler]
// fn panic(info: &core::panic::PanicInfo) -> ! {
// disable interrupts - firmware has panicked so no ISRs should continue running
// avr_device::interrupt::disable();

// get the peripherals so we can access serial and the LED.
//
// SAFETY: Because main() already has references to the peripherals this is an unsafe
// operation - but because no other code can run after the panic handler was called,
// we know it is okay.
// let dp = unsafe { arduino_hal::Peripherals::steal() };
// let pins = arduino_hal::pins!(dp);
// let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

// Print out panic location
// ufmt::uwriteln!(&mut serial, "Firmware panic!\r").unwrap_infallible();
// if let Some(loc) = info.location() {
//     ufmt::uwriteln!(
//         &mut serial,
//         "{}\r",
//         // loc.file(),
//         loc.line(),
//         // loc.column(),
//     )
//     .unwrap_infallible();
// }

// if let Some(loc) = info.location() {
//     if loc.file() == "firmware.rs" && loc.line() > 110 {
//         let mut led = pins.d4.into_output();
//         led.set_high();
//     }
// }
// loop {} //So it's infallible type ret.
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
    //using non_macro to work around a partial move issue
    let usart = dp.USART0;
    let mut serial = arduino_hal::Usart::new(
        usart,
        pins.d0,
        pins.d1.into_output(),
        BaudrateExt::into_baudrate(57600),
    );
    // let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    let mut led = pins.d13.into_output();
    led.set_low();
    let mut err_led = pins.d4.into_output();

    // let mut buzzer_timer = dp.TC1.

    let mic = pins.a0.into_analog_input(&mut adc);
    let buzzer = pins.d9.into_output();
    let buzzer_regs = BuzzerRegisters::new(&dp);

    tone(&buzzer, buzzer_regs, 440, 2000);

    let mut display = match SSD1306Display::new(&mut i2c) {
        Ok(disp) => disp,
        Err(_) => {
            ufmt::uwriteln!(&mut serial, "Could not initialize display").unwrap_infallible();
            err_led.set_high();
            panic!() //No point in continuing without a core piece missing
        }
    };
    display.clear(&mut i2c).expect("init failed!!");
    display.write_str(
        &mut i2c,
        "SHUT UP DEVICE\nrev 0.1 pre-alpha\nin-engine footage",
    );

    arduino_hal::delay_ms(2000);
    display.clear(&mut i2c).unwrap();

    //Since we are in no_std land, allocate a buffer for the display strings, then we can use ufmt
    //heapless crate is based as hell
    let mut oled_buf1: heapless::String<64> = heapless::String::new();

    const ENUM_RANGE: Range<u16> = 0..100;
    loop {
        // sure, we could do async but that's a headache
        // Timings here are faster than most human reaction speeds, so we should be fine
        let (min_ms, max_ms) = ENUM_RANGE.fold((1024, 0), |(min_val, max_val), _| {
            let measured = arduino_hal::Adc::read_blocking(&mut adc, &mic);
            (min(min_val, measured), max(max_val, measured))
        });
        let vpp_raw = max_ms - min_ms; //Effectively Vp_p or peak-to-peak voltage in Quantized values
        let vpp = vpp_raw as f32 / 1024.0 * 5.0;

        ufmt::uwriteln!(
            &mut serial,
            "Vp_p: {}V,Raw: {}\r",
            ufmt_float::uFmt_f32::Three(vpp),
            vpp_raw
        )
        .unwrap_infallible();

        //Prevents panic from reaching end of buffer
        //AFAIK uwrite trait can't "seek"
        oled_buf1.clear();
        ufmt::uwrite!(
            &mut oled_buf1,
            "Vp_p: {}V   \nRaw ADC: {}    ", //Spaces to overwrite, cheaper than clear operation
            ufmt_float::uFmt_f32::Three(vpp),
            vpp_raw
        )
        .unwrap();

        //Reset Display + Write to it
        match display.set_cursor(&mut i2c, 0, 0) {
            Ok(_) => (),
            Err(err) => {
                ufmt::uwriteln!(&mut serial, "set_cursor error {:?}", err).unwrap_infallible();
                err_led.set_high();
            }
        };
        display.write_str(&mut i2c, &oled_buf1.as_str());

        // arduino_hal::delay_ms(1000);
    }
}
