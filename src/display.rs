///This is based off of the [SSD1306Ascii Library](https://github.com/greiman/SSD1306Ascii) written in C/C++ for Arduino, but
/// with most of the fat cut out. We don't need it for this project.
use arduino_hal::i2c::Error;
use embedded_hal::i2c::{I2c as BaseI2c, Operation, Operation::Write};

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
// const SSD1306_DISPLAYALLON: u8 = 0xA5;
/** Set Normal Display. */
const SSD1306_NORMALDISPLAY: u8 = 0xA6;
/** Set Inverse Display. */
// const SSD1306_INVERTDISPLAY: u8 = 0xA7;
/** Set Multiplex Ratio from 16 to 63. */
const SSD1306_SETMULTIPLEX: u8 = 0xA8;
/** Set Display off. */
const SSD1306_DISPLAYOFF: u8 = 0xAE;
/** Set Display on. */
const SSD1306_DISPLAYON: u8 = 0xAF;
/**Set GDDRAM Page Start Address. */
const SSD1306_SETSTARTPAGE: u8 = 0xB0;
/** Set COM output scan direction normal. */
// const SSD1306_COMSCANINC: u8 = 0xC0;
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
// const SSD1306_DEACTIVATE_SCROLL: u8 = 0x2E;
/** No Operation Command. */
// const SSD1306_NOP: u8 = 0xE3;

const I2C_ADDR: u8 = 0x3C;

// enum SSD1306WriteMode {
//     Command = 0, //writeCmd
//     RAM = 1,
//     RAMBuf = 2,
// }

pub struct SSD1306Display {
    col: u8,
    row: u8,
    // wire: &'static mut I2c, //SAFETY, this is constructed by HAL, and should last remainder of program
}

impl SSD1306Display {
    pub fn new(wire: &mut arduino_hal::I2c) -> Result<Self, Error> {
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
            Ok(_) => Ok(Self { col: 0, row: 0 }),
            Err(err) => Err(err),
        }
    }

    pub fn write(&mut self, wire: &mut arduino_hal::I2c, ch: char) -> usize {
        match ch {
            '\r' => {
                //Carriage return only sets cursor to beginning of row similar to typewriter (hence why ms-dos uses \r\n)
                self.col = 0;
                1
            }
            '\n' => {
                self.col = 0;
                self.row += 1; //New page
                1
            }
            ' ' => {
                //Handle as a "non-font" space (i.e. not 5px long)
                //I'm choosing two as a nice choice
                self.write_ram_buf(wire, &[0x00, 0x00]);
                self.col += 2;
                1
            }
            ch if (ch as u8) > 0x20 && (ch as u8) < 0x80 => {
                let ascii = ch as u8 - 0x20; //Space is covered in prev branch
                let byte_seq = FONT_DATA[ascii as usize];
                //In theory, this should just work as is given we have no significant modification
                self.write_ram_buf(wire, &byte_seq);
                self.col += 5;
                1
            }
            _ => 0, //Just ignore the character
        }
    }

    pub fn write_str(&mut self, wire: &mut arduino_hal::I2c, str: &str) -> usize {
        let mut count = 0;
        for ch in str.chars() {
            count += self.write(wire, ch);
        }
        count
    }

    fn write_cmd(&mut self, wire: &mut arduino_hal::I2c, byte: u8) {
        match wire.transaction(
            I2C_ADDR,
            &mut [Operation::Write(&[0x00]), Operation::Write(&[byte])],
        ) {
            Ok(_) => (),
            Err(_) => (), //TODO Better err handling!!!
        }
    }

    fn write_cmd_buf(&mut self, wire: &mut arduino_hal::I2c, bytes: &[u8]) {
        match wire.transaction(
            I2C_ADDR,
            &mut [Operation::Write(&[0x00]), Operation::Write(&[byte])],
        ) {
            Ok(_) => (),
            Err(_) => (), //TODO Better err handling!!!
        }
    }

    // fn write_ram(&mut self, byte: u8) {
    //     match self.wire.transaction(
    //         I2C_ADDR,
    //         &mut [Operation::Write(&[0x40]), Operation::Write(&[byte])],
    //     ) {
    //         Ok(_) => (),
    //         Err(_) => (), //TODO Better err handling!!!
    //     }
    // }

    fn write_ram_buf(&mut self, wire: &mut arduino_hal::I2c, bytes: &[u8]) {
        //This is "optimized" since the buffer mode allows for 16 bytes to be processed in a single command
        bytes.chunks(16).for_each(|chunk| {
            match wire.transaction(
                I2C_ADDR,
                &mut [Operation::Write(&[0x40]), Operation::Write(chunk)],
            ) {
                Ok(_) => (),
                Err(_) => (), //TODO Better err handling!!!
            }
        });
    }

    pub fn clear(&mut self, wire: &mut arduino_hal::I2c) {
        for row in 0..DISPLAY_HEIGHT {
            self.set_cursor(wire, 0, row);
            for _ in 0..DISPLAY_WIDTH / 16 {
                self.write_ram_buf(wire, &[0x00; 16]);
            }
        }

        self.set_cursor(wire, 0, 0);
    }

    pub fn set_cursor(&mut self, wire: &mut arduino_hal::I2c, col: u8, row: u8) {
        //Set row
        if row < DISPLAY_HEIGHT / 8 {
            self.row = row;
            self.write_cmd(wire, SSD1306_SETSTARTPAGE | row);
        }
        //Set col
        if col < DISPLAY_WIDTH {
            self.col = col;
            self.write_cmd(wire, SSD1306_SETLOWCOLUMN | (col & 0xF));
            self.write_cmd(wire, SSD1306_SETHIGHCOLUMN | (col >> 4));
        }
    }
}

//Assumes 0 = ASCII 0x20 or ASCII 32
const FONT_DATA: [[u8; 5]; 96] = [
    [0x00, 0x00, 0x00, 0x00, 0x00], // (space)
    [0x00, 0x00, 0x5F, 0x00, 0x00], // !
    [0x00, 0x07, 0x00, 0x07, 0x00], // "
    [0x14, 0x7F, 0x14, 0x7F, 0x14], // #
    [0x24, 0x2A, 0x7F, 0x2A, 0x12], // $
    [0x23, 0x13, 0x08, 0x64, 0x62], // %
    [0x36, 0x49, 0x55, 0x22, 0x50], // &
    [0x00, 0x05, 0x03, 0x00, 0x00], // '
    [0x00, 0x1C, 0x22, 0x41, 0x00], // (
    [0x00, 0x41, 0x22, 0x1C, 0x00], // )
    [0x08, 0x2A, 0x1C, 0x2A, 0x08], // *
    [0x08, 0x08, 0x3E, 0x08, 0x08], // +
    [0x00, 0x50, 0x30, 0x00, 0x00], // ,
    [0x08, 0x08, 0x08, 0x08, 0x08], // -
    [0x00, 0x60, 0x60, 0x00, 0x00], // .
    [0x20, 0x10, 0x08, 0x04, 0x02], // /
    [0x3E, 0x51, 0x49, 0x45, 0x3E], // 0
    [0x00, 0x42, 0x7F, 0x40, 0x00], // 1
    [0x42, 0x61, 0x51, 0x49, 0x46], // 2
    [0x21, 0x41, 0x45, 0x4B, 0x31], // 3
    [0x18, 0x14, 0x12, 0x7F, 0x10], // 4
    [0x27, 0x45, 0x45, 0x45, 0x39], // 5
    [0x3C, 0x4A, 0x49, 0x49, 0x30], // 6
    [0x01, 0x71, 0x09, 0x05, 0x03], // 7
    [0x36, 0x49, 0x49, 0x49, 0x36], // 8
    [0x06, 0x49, 0x49, 0x29, 0x1E], // 9
    [0x00, 0x36, 0x36, 0x00, 0x00], // :
    [0x00, 0x56, 0x36, 0x00, 0x00], // ;
    [0x00, 0x08, 0x14, 0x22, 0x41], // <
    [0x14, 0x14, 0x14, 0x14, 0x14], // =
    [0x41, 0x22, 0x14, 0x08, 0x00], // >
    [0x02, 0x01, 0x51, 0x09, 0x06], // ?
    [0x32, 0x49, 0x79, 0x41, 0x3E], // @
    [0x7E, 0x11, 0x11, 0x11, 0x7E], // A
    [0x7F, 0x49, 0x49, 0x49, 0x36], // B
    [0x3E, 0x41, 0x41, 0x41, 0x22], // C
    [0x7F, 0x41, 0x41, 0x22, 0x1C], // D
    [0x7F, 0x49, 0x49, 0x49, 0x41], // E
    [0x7F, 0x09, 0x09, 0x01, 0x01], // F
    [0x3E, 0x41, 0x41, 0x51, 0x32], // G
    [0x7F, 0x08, 0x08, 0x08, 0x7F], // H
    [0x00, 0x41, 0x7F, 0x41, 0x00], // I
    [0x20, 0x40, 0x41, 0x3F, 0x01], // J
    [0x7F, 0x08, 0x14, 0x22, 0x41], // K
    [0x7F, 0x40, 0x40, 0x40, 0x40], // L
    [0x7F, 0x02, 0x04, 0x02, 0x7F], // M
    [0x7F, 0x04, 0x08, 0x10, 0x7F], // N
    [0x3E, 0x41, 0x41, 0x41, 0x3E], // O
    [0x7F, 0x09, 0x09, 0x09, 0x06], // P
    [0x3E, 0x41, 0x51, 0x21, 0x5E], // Q
    [0x7F, 0x09, 0x19, 0x29, 0x46], // R
    [0x46, 0x49, 0x49, 0x49, 0x31], // S
    [0x01, 0x01, 0x7F, 0x01, 0x01], // T
    [0x3F, 0x40, 0x40, 0x40, 0x3F], // U
    [0x1F, 0x20, 0x40, 0x20, 0x1F], // V
    [0x7F, 0x20, 0x18, 0x20, 0x7F], // W
    [0x63, 0x14, 0x08, 0x14, 0x63], // X
    [0x03, 0x04, 0x78, 0x04, 0x03], // Y
    [0x61, 0x51, 0x49, 0x45, 0x43], // Z
    [0x00, 0x00, 0x7F, 0x41, 0x41], // [
    [0x02, 0x04, 0x08, 0x10, 0x20], // "\"
    [0x41, 0x41, 0x7F, 0x00, 0x00], // ]
    [0x04, 0x02, 0x01, 0x02, 0x04], // ^
    [0x40, 0x40, 0x40, 0x40, 0x40], // _
    [0x00, 0x01, 0x02, 0x04, 0x00], // `
    [0x20, 0x54, 0x54, 0x54, 0x78], // a
    [0x7F, 0x48, 0x44, 0x44, 0x38], // b
    [0x38, 0x44, 0x44, 0x44, 0x20], // c
    [0x38, 0x44, 0x44, 0x48, 0x7F], // d
    [0x38, 0x54, 0x54, 0x54, 0x18], // e
    [0x08, 0x7E, 0x09, 0x01, 0x02], // f
    [0x08, 0x14, 0x54, 0x54, 0x3C], // g
    [0x7F, 0x08, 0x04, 0x04, 0x78], // h
    [0x00, 0x44, 0x7D, 0x40, 0x00], // i
    [0x20, 0x40, 0x44, 0x3D, 0x00], // j
    [0x00, 0x7F, 0x10, 0x28, 0x44], // k
    [0x00, 0x41, 0x7F, 0x40, 0x00], // l
    [0x7C, 0x04, 0x18, 0x04, 0x78], // m
    [0x7C, 0x08, 0x04, 0x04, 0x78], // n
    [0x38, 0x44, 0x44, 0x44, 0x38], // o
    [0x7C, 0x14, 0x14, 0x14, 0x08], // p
    [0x08, 0x14, 0x14, 0x18, 0x7C], // q
    [0x7C, 0x08, 0x04, 0x04, 0x08], // r
    [0x48, 0x54, 0x54, 0x54, 0x20], // s
    [0x04, 0x3F, 0x44, 0x40, 0x20], // t
    [0x3C, 0x40, 0x40, 0x20, 0x7C], // u
    [0x1C, 0x20, 0x40, 0x20, 0x1C], // v
    [0x3C, 0x40, 0x30, 0x40, 0x3C], // w
    [0x44, 0x28, 0x10, 0x28, 0x44], // x
    [0x0C, 0x50, 0x50, 0x50, 0x3C], // y
    [0x44, 0x64, 0x54, 0x4C, 0x44], // z
    [0x00, 0x08, 0x36, 0x41, 0x00], // {
    [0x00, 0x00, 0x7F, 0x00, 0x00], // |
    [0x00, 0x41, 0x36, 0x08, 0x00], // }
    [0x08, 0x08, 0x2A, 0x1C, 0x08], // ->
    [0x08, 0x1C, 0x2A, 0x08, 0x08], // <-
];
