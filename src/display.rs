const DISPLAY_WIDTH: u8 = 128;
const DISPLAY_HEIGHT: u8 = 64;

/** Set Lower Column Start Address for Page Addressing Mode. */
const SSD1306_SETLOWCOLUMN: u8 = 0x00;
/** Set Higher Column Start Address for Page Addressing Mode. */
const SSD1306_SETHIGHCOLUMN: u8 = 0x10;
/** Set Memory Addressing Mode. */
const SSD1306_MEMORYMODE: u8 = 0x20;
/** Set display RAM display start line register from 0 - 63. */
const SSD1306_SETSTARTLINE: u8 = 0x40;
/** Set Display Contrast to one of 256 steps. */
const SSD1306_SETCONTRAST: u8 = 0x81;
/** Enable or disable charge pump.  Follow with 0X14 enable, 0X10 disable. */
const SSD1306_CHARGEPUMP: u8 = 0x8D;
/** Set Segment Re-map between data column and the segment driver. */
const SSD1306_SEGREMAP: u8 = 0xA0;
/** Resume display from GRAM content. */
const SSD1306_DISPLAYALLON_RESUME: u8 = 0xA4;
/** Force display on regardless of GRAM content. */
const SSD1306_DISPLAYALLON: u8 = 0xA5;
/** Set Normal Display. */
const SSD1306_NORMALDISPLAY: u8 = 0xA6;
/** Set Inverse Display. */
const SSD1306_INVERTDISPLAY: u8 = 0xA7;
/** Set Multiplex Ratio from 16 to 63. */
const SSD1306_SETMULTIPLEX: u8 = 0xA8;
/** Set Display off. */
const SSD1306_DISPLAYOFF: u8 = 0xAE;
/** Set Display on. */
const SSD1306_DISPLAYON: u8 = 0xAF;
/**Set GDDRAM Page Start Address. */
const SSD1306_SETSTARTPAGE: u8 = 0xB0;
/** Set COM output scan direction normal. */
const SSD1306_COMSCANINC: u8 = 0xC0;
/** Set COM output scan direction reversed. */
const SSD1306_COMSCANDEC: u8 = 0xC8;
/** Set Display Offset. */
const SSD1306_SETDISPLAYOFFSET: u8 = 0xD3;
/** Sets COM signals pin configuration to match the OLED panel layout. */
const SSD1306_SETCOMPINS: u8 = 0xDA;
/** This command adjusts the VCOMH regulator output. */
const SSD1306_SETVCOMDETECT: u8 = 0xDB;
/** Set Display Clock Divide Ratio/ Oscillator Frequency. */
const SSD1306_SETDISPLAYCLOCKDIV: u8 = 0xD5;
/** Set Pre-charge Period */
const SSD1306_SETPRECHARGE: u8 = 0xD9;
/** Deactivate scroll */
const SSD1306_DEACTIVATE_SCROLL: u8 = 0x2E;
/** No Operation Command. */
const SSD1306_NOP: u8 = 0xE3;
//------------------------------------------------------------------------------
/** Set Pump voltage value: (30H~33H) 6.4, 7.4, 8.0 (POR), 9.0. */
const SH1106_SET_PUMP_VOLTAGE: u8 = 0x30;
/** First byte of set charge pump mode */
const SH1106_SET_PUMP_MODE: u8 = 0xAD;
/** Second byte charge pump on. */
const SH1106_PUMP_ON: u8 = 0x8B;
/** Second byte charge pump off. */
const SH1106_PUMP_OFF: u8 = 0x8A;

#[rustfmt::skip]
const DISPLAY_INIT_SEQ: [u8;25] = [
    SSD1306_DISPLAYOFF,
    SSD1306_SETDISPLAYCLOCKDIV, 0x80,  // the suggested ratio 0x80
    SSD1306_SETMULTIPLEX, 0x3F,        // ratio 64
    SSD1306_SETDISPLAYOFFSET, 0x0,     // no offset
    SSD1306_SETSTARTLINE,              // line #0
    SSD1306_CHARGEPUMP, 0x14,          // internal vcc
    SSD1306_MEMORYMODE, 0x02,          // page mode
    SSD1306_SEGREMAP | 0x1,            // column 127 mapped to SEG0
    SSD1306_COMSCANDEC,                // column scan direction reversed
    SSD1306_SETCOMPINS, 0x12,          // alt COM pins, disable remap
    SSD1306_SETCONTRAST, 0x7F,         // contrast level 127
    SSD1306_SETPRECHARGE, 0xF1,        // pre-charge period (1, 15)
    SSD1306_SETVCOMDETECT, 0x40,       // vcomh regulator level
    SSD1306_DISPLAYALLON_RESUME,
    SSD1306_NORMALDISPLAY,
    SSD1306_DISPLAYON
];

pub struct SSD1306Display {
    col: u8,
    row: u8,
    col_offset: u8,
    letter_spacing: u8,
    skip: u8,
    invert_mask: bool,
    mag_factor: u8,
}

impl SSD1306Display {
    pub fn write(ch: char) -> usize {
        todo!()
    }

    fn write_display(byte: u8, mode: u8) {
        todo!()
    }

    pub fn clear() {
        todo!()
    }

    pub fn clear_region(c0: u8, c1: u8, r0: u8, r1: u8) {
        todo!()
    }

    pub fn col() -> u8 {
        col
    }

    pub fn display_remap(mode: bool) {
        todo!()
    }

    pub fn home(){

    }

    pub fn set_cursor(col:u8, row:u8){
        todo!()
    }

    pub fn ssd1306_write_cmd()
}
