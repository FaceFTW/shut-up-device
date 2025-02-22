//This is based on the megaavr version of the Arduino API's
//`tone()` and `noTone()` functions
//Reference: https://github.com/arduino/ArduinoCore-megaavr/blob/master/cores/arduino/Tone.cpp
//Some liberties are taken to simplify use

//note TIMERB0=2

// use core::sync::atomic::{AtomicU16, AtomicU8};

use arduino_hal::{hal::port::PB1, port::mode::Output};
use avr_device::interrupt::Mutex;

//SAFETY: Only used in this module, no multi-buzzer setup expected
static TIMER_TOGGLE_COUNT: Mutex<u32> = Mutex::new(0);
static mut TIMER_PORT: *mut u8 = core::ptr::null_mut(); //Set when tone is called
static mut TIMER_PIN_MASK: *mut u8 = core::ptr::null_mut();

const F_CPU: u32 = 1000000; //Assume 1MHz cpu clock, afaik this isn't configured by code I wrote

///Plays a tone assuming a buzzer connected to digital pin 9
pub fn tone<PIN>(
    peripherals: &arduino_hal::Peripherals,
    pin: &arduino_hal::port::Pin<Output, PB1>,
    frequency: u16,
    duration: u32,
) {
    //Setup the timer
    peripherals.TC2.tccr2a.write(|w| w.wgm2().ctc());
    peripherals.TC2.tccr2b.write(|w| w.cs2().direct());

    //SAFETY: Raw addresses are given so the ISR can actaully do the toggling we want
    unsafe {
        TIMER_PORT = peripherals.PORTB.pinb.as_ptr();
    }

    let (prescalar_bits, ocr) = {
        let mut ocr_base = F_CPU / frequency as u32 / 2 - 1;
        let mut bits = 0b001;
        //NOTE this was written out as discrete statements in the original arduino code
        // partially due to multi-config targeting, but maybe small perf? (i.e. bad loop opts)
        while ocr_base > 255 && bits < 0b111 {
            ocr_base = (ocr_base + 1) / 2 - 1; //preserve accuracy
            bits = bits << 1;
        }
        (bits, ocr_base as u8)
    };

    peripherals
        .TC2
        .tccr2b
        .write(|w| w.cs2().bits(prescalar_bits.into()));

    let toggle_count: u32 = 2 * frequency as u32 * duration as u32 / 1000;
    peripherals.TC2.ocr2a.write(|w| w.bits(ocr));
    peripherals.TC2.timsk2.write(|w| w.ocie2a().set_bit());
}

// fn timer2_isr()
#[avr_device::interrupt(atmega328p)]
fn TIMER2_COMPA() {}
