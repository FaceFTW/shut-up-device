//This is based on the megaavr version of the Arduino API's
//`tone()` and `noTone()` functions
//Reference: https://github.com/arduino/ArduinoCore-megaavr/blob/master/cores/arduino/Tone.cpp
//Some liberties are taken to simplify use

use arduino_hal::{
    hal::port::PB1,
    pac::tc2::{ocr2a::OCR2A_SPEC, tccr2a::TCCR2A_SPEC, tccr2b::TCCR2B_SPEC, timsk2::TIMSK2_SPEC},
    port::mode::Output,
};
use avr_device::interrupt::Mutex;

//SAFETY: Only used in this module, no multi-buzzer setup expected
static mut TIMER_TOGGLE_COUNT: Mutex<u32> = Mutex::new(0);
static mut BUZZER_PIN_PORT: *mut u8 = core::ptr::null_mut(); //Set when tone is called
const BUZZER_PIN_MASK: u8 = 0b00000010; //Mask for pin 9a
static mut TIMSK2_ADDR: *mut u8 = core::ptr::null_mut(); //Set when tone is called
const TIMSK2_OCIE2A: u8 = 0b00000010; //Mask fo

// static TIMSK2_ADDR: *const Reg<TIMSK2_SPEC> = Unini

const F_CPU: u32 = 1000000; //Assume 1MHz cpu clock, afaik this isn't configured by code I wrote

/// Struct to extract the register abstractions + pointers from a [arduino_hal::Peripherals]
/// struct. Prevents other borrowing issues by taking only what we need. Assumes
/// that all registers in peripherals live as long as peripherals, which should trivially
/// hold.
pub struct BuzzerRegisters<'a> {
    buzzer_pin_port: *mut u8,
    timsk2_addr: *mut u8,
    timsk2: &'a avr_device::generic::Reg<TIMSK2_SPEC>,
    tccr2a: &'a avr_device::generic::Reg<TCCR2A_SPEC>,
    tccr2b: &'a avr_device::generic::Reg<TCCR2B_SPEC>,
    ocr2a: &'a avr_device::generic::Reg<OCR2A_SPEC>,
}

impl<'a> BuzzerRegisters<'a> {
    pub fn new(peripherals: &'a arduino_hal::Peripherals) -> Self {
        Self {
            buzzer_pin_port: { peripherals.PORTB.pinb.as_ptr() },
            timsk2_addr: { peripherals.TC2.timsk2.as_ptr() },
            timsk2: &peripherals.TC2.timsk2,
            tccr2a: &peripherals.TC2.tccr2a,
            tccr2b: &peripherals.TC2.tccr2b,
            ocr2a: &peripherals.TC2.ocr2a,
        }
    }
}

///Plays a tone assuming a buzzer connected to digital pin 9
pub fn tone<'a>(
    // peripherals: &arduino_hal::Peripherals,
    _pin: &arduino_hal::port::Pin<Output, PB1>, //Nice assertion to ensure that the pin is set as we expect
    registers: BuzzerRegisters<'a>,
    frequency: u16,
    duration: u32,
) {
    //SAFETY: Raw addresses are given so the ISR can actaully do the toggling we want
    if unsafe { BUZZER_PIN_PORT.is_null() } {
        unsafe {
            BUZZER_PIN_PORT = registers.buzzer_pin_port;
        }
    }

    if unsafe { TIMSK2_ADDR.is_null() } {
        unsafe {
            TIMSK2_ADDR = registers.timsk2_addr;
        }
    }

    //Setup the timer
    registers.tccr2a.write(|w| w.wgm2().ctc());
    registers.tccr2b.write(|w| w.cs2().direct());

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

    registers
        .tccr2b
        .write(|w| w.cs2().bits(prescalar_bits.into()));

    let toggle_count: u32 = 2 * frequency as u32 * duration as u32 / 1000;

    //SAFETY: only other place this is accessed is the ISR, plus mutex guards.
    //This should be dropped before ISR is called with the inner scoping
    {
        #[allow(static_mut_refs)]
        let toggle_mutex_inner = unsafe { TIMER_TOGGLE_COUNT.get_mut() };
        *toggle_mutex_inner = toggle_count;
        // drop(toggle_mutex_inner);
    }

    registers.ocr2a.write(|w| w.bits(ocr));
    registers.timsk2.write(|w| w.ocie2a().set_bit());
}

#[avr_device::interrupt(atmega328p)]
fn TIMER2_COMPA() {
    //SAFETY: static mut ref should be dropped when exiting ISR
    #[allow(static_mut_refs)]
    let toggle_mutex_inner = unsafe { TIMER_TOGGLE_COUNT.get_mut() };
    if *toggle_mutex_inner > 0 {
        unsafe {
            *BUZZER_PIN_PORT ^= BUZZER_PIN_MASK; //toggle the port
        }
        *toggle_mutex_inner -= 1;
    } else {
        //HACK don't care about PWM on the pin AFAIK
        //unset interrupt mask
        //SAFETY Only accesssed when interrupt is being toggled off.
        unsafe {
            *TIMSK2_ADDR &= TIMSK2_OCIE2A;
            *BUZZER_PIN_PORT &= !BUZZER_PIN_MASK; //Ensure it is set off
        }
    }
}
