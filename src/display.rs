use arduino_hal::{i2c::Error, I2c};
use embedded_hal::i2c::{self, I2c as BaseI2c, Operation, Operation::Write};

///Width of the display. Change this if your display has different dimensions
const DISPLAY_WIDTH: u8 = 128;
///Height of the display. Change this if your display has different dimensions
const DISPLAY_HEIGHT: u8 = 64;
///Column Offset to use with the device.
// const COL_OFFSET: u8 = 0;

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


const I2C_ADDR: u8 = 0x30;

// enum SSD1306WriteMode {
//     Command = 0, //writeCmd
//     RAM = 1,
//     RAMBuf = 2,
// }

pub struct SSD1306Display {
    col: u8,
    row: u8,
    wire: &'static mut I2c, //SAFETY, this is constructed by HAL, and should last remainder of program
}

impl SSD1306Display {
    pub fn new(wire: &'static mut arduino_hal::I2c) -> Result<Self, Error> {
        //The Command Sequence to send to the display over the wire for initialization
        //
        // This includes the bytes to indicate each value is a command so there is no need
        // to go through the [SSD1306Display::write_cmd] function with all the error handling.
        // If we fail here, we should actually abort!
        #[rustfmt::skip]
        let mut display_init_seq: [Operation;50] = [
            Write(&[0x00]), Write(&[SSD1306_DISPLAYOFF]),
            Write(&[0x00]), Write(&[SSD1306_SETDISPLAYCLOCKDIV]),Write(&[0x00]),Write(&[0x80]),     // the suggested ratio 0x80
            Write(&[0x00]), Write(&[SSD1306_SETMULTIPLEX]),Write(&[0x00]),Write(&[0x3F]),           // ratio 64
            Write(&[0x00]), Write(&[SSD1306_SETDISPLAYOFFSET]),Write(&[0x00]),Write(&[0x0]),        // no offset
            Write(&[0x00]), Write(&[SSD1306_SETSTARTLINE]),                                         // line #0
            Write(&[0x00]), Write(&[SSD1306_CHARGEPUMP]),Write(&[0x00]),Write(&[0x14]),             // internal vcc
            Write(&[0x00]), Write(&[SSD1306_MEMORYMODE]),Write(&[0x00]),Write(&[0x02]),             // page mode
            Write(&[0x00]), Write(&[SSD1306_SEGREMAP | 0x1]),                                       // column 127 mapped to SEG0
            Write(&[0x00]), Write(&[SSD1306_COMSCANDEC]),                                           // column scan direction reversed
            Write(&[0x00]), Write(&[SSD1306_SETCOMPINS]),Write(&[0x00]),Write(&[0x12]),             // alt COMWrite(&[pins]), disable remap
            Write(&[0x00]), Write(&[SSD1306_SETCONTRAST]),Write(&[0x00]),Write(&[0x7F]),            // contrast level 127
            Write(&[0x00]), Write(&[SSD1306_SETPRECHARGE]),Write(&[0x00]),Write(&[0xF1]),           // pre-charge period (1, 15)
            Write(&[0x00]), Write(&[SSD1306_SETVCOMDETECT]),Write(&[0x00]),Write(&[0x40]),          // vcomh regulator level
            Write(&[0x00]), Write(&[SSD1306_DISPLAYALLON_RESUME]),
            Write(&[0x00]), Write(&[SSD1306_NORMALDISPLAY]),
            Write(&[0x00]), Write(&[SSD1306_DISPLAYON]),
        ];
        match wire.transaction(I2C_ADDR, &mut display_init_seq) {
            Ok(_) => Ok(Self {
                col: 0,
                row: 0,
                wire,
            }),
            Err(err) => Err(err),
        }
    }

    pub fn write(ch: char) -> usize {
        todo!()
    }

    fn write_cmd(&mut self, byte: u8) {
        match self.wire.transaction(
            I2C_ADDR,
            &mut [Operation::Write(&[0x00]), Operation::Write(&[byte])],
        ) {
            Ok(_) => (),
            Err(_) => (), //TODO Better err handling!!!
        }
    }

    fn write_ram(&mut self, byte: u8) {
        match self.wire.transaction(
            I2C_ADDR,
            &mut [Operation::Write(&[0x40]), Operation::Write(&[byte])],
        ) {
            Ok(_) => (),
            Err(_) => (), //TODO Better err handling!!!
        }
    }

    fn write_ram_buf(&mut self, bytes: &[u8]) {
        //This is "optimized" since the buffer mode allows for 16 bytes to be processed in a single command
        bytes.chunks(16).for_each(|chunk| {
            match self.wire.transaction(
                I2C_ADDR,
                &mut [Operation::Write(&[0x40]), Operation::Write(chunk)],
            ) {
                Ok(_) => (),
                Err(_) => (), //TODO Better err handling!!!
            }
        });
    }

    pub fn clear(&mut self) {
        for row in 0..DISPLAY_HEIGHT {
            self.set_cursor(0, row);
            for _ in 0..DISPLAY_WIDTH / 16 {
                self.write_ram_buf(&[0x00; 16]);
            }
        }

        self.set_cursor(0, 0);
    }

    pub fn display_remap(mode: bool) {
        todo!()
    }

    pub fn home() {}

    pub fn set_cursor(&mut self, col: u8, row: u8) {
        //Set row
        if row < DISPLAY_HEIGHT {
            self.row = row;
            self.write_cmd(SSD1306_SETSTARTPAGE | row);
        }
        //Set col
        if col < DISPLAY_WIDTH {
            self.col = col;
            self.write_cmd(SSD1306_SETLOWCOLUMN | (col & 0xF));
            self.write_cmd(SSD1306_SETHIGHCOLUMN | (col >> 4));
        }
    }
}
