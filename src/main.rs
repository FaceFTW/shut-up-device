#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

mod display;

use arduino_hal::clock::Clock;
use core::{
    cmp::{max, min},
    ops::Range,
};
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

/// --------------------------------------
/// Tone Related Functionality
/// --------------------------------------
/// This is based on the avr version of the Arduino API's `tone()` and `noTone()` functions
/// Reference: https://github.com/arduino/ArduinoCore-avr/blob/master/cores/arduino/Tone.cpp
/// Some liberties are taken to simplify use
///
/// ASSUMPTIONS
/// - TCCR2A Wave Generation Mode (WGM2) is set to CTC (010) before [tone] calls
/// - BUZZER_PIN_PORT is initialized in [main]
/// - TIMSK2_REF is initialized in [main] before [tone] calls
/// - Buzzer is on Pin 9 (PB1), it is initialized in [main] before [tone] calls to be [avr_hal_generic::port::mode::Output]

//SAFETY: Only used for the buzzer, no multi-buzzer setup expected
static mut BUZZER_PIN_PORT: *mut u8 = core::ptr::null_mut(); //Set early in main
const BUZZER_PIN_MASK: u8 = 0b00000010; //Mask for pin 9a

fn tone(tc2: &avr_device::atmega328p::TC2, frequency: u16) {
    tc2.tccr2a.write(|w| w.wgm2().ctc());
    // registers.tccr2b.write(|w| w.cs2().direct());

    let (prescalar_bits, ocr) = {
        let mut ocr_base = arduino_hal::DefaultClock::FREQ / frequency as u32 / 2 - 1;
        let mut bits = 0b001;
        //NOTE this was written out as discrete statements in the original arduino code
        // partially due to multi-config targeting, but maybe small perf? (i.e. bad loop opts)
        while ocr_base > 255 && bits < 0b111 {
            ocr_base = (ocr_base + 1) / 2 - 1; //preserve accuracy
            bits = bits << 1;
        }
        (bits, ocr_base as u8)
    };

    tc2.tccr2b.write(|w| w.cs2().bits(prescalar_bits.into()));

    tc2.ocr2a.write(|w| w.bits(ocr));
    tc2.timsk2.write(|w| w.ocie2a().set_bit());
}

fn no_tone(tc2: &avr_device::atmega328p::TC2) {
    tc2.timsk2.write(|w| w.ocie2a().clear_bit());
    //Easier than passing the pin as an arg
    //SAFETY: Only other time it is accessed this way is through ISR, which is now disabled
    unsafe {
        *BUZZER_PIN_PORT &= !BUZZER_PIN_MASK; //Turn the port off
    }
}

fn tone_duration(tc2: &avr_device::atmega328p::TC2, frequency: u16, delay: u16) {
    tone(tc2, frequency);
    arduino_hal::delay_ms(delay);
    no_tone(tc2);
}

/// Really basic ISR to toggle the buzzer pin, intentionally lightweight since
/// AVR doesn't have interrupt nesting, so it doesn't take enough timer cycles.
/// Source: https://github.com/Rahix/avr-hal/issues/75#issuecomment-706031854
/// SAFETY ASSUMPTION - ISR Mask is disabled outside of interrupt space
#[avr_device::interrupt(atmega328p)]
fn TIMER2_COMPA() {
    unsafe {
        *BUZZER_PIN_PORT ^= BUZZER_PIN_MASK; //toggle the port
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    }
}

/// ---------------------------------------
/// Firmware Entry
/// ----------------------------------------

#[arduino_hal::entry]
fn main() -> ! {
    arduino_hal::delay_ms(3000); //Reprogramming Window

    let dp = arduino_hal::Peripherals::take().unwrap();
    //SAFETY - Initialization of the global
    unsafe {
        BUZZER_PIN_PORT = dp.PORTB.portb.as_ptr().clone();
    }
    let pins = arduino_hal::pins!(dp);

    //Setup MCU Subsystems
    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        400000,
    );
    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    //Setup Specific Pins
    let mut led = pins.d13.into_output();
    led.set_low();
    let mut err_led = pins.d4.into_output();
    let _buzzer = pins.d9.into_output();
    let mic = pins.a0.into_analog_input(&mut adc);

    tone_duration(&dp.TC2, 440, 2000);

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
