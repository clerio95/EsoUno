//! Stage 1 firmware: a Brainfuck prompt over USB serial.
//!
//! Type a program and press Enter. `.` writes a byte to the serial console
//! and `,` waits for a byte from it. The on-board LED (D13) is lit while a
//! program runs. A program that never ends is stopped with the reset button.
//!
//! No kit parts are wired yet; the keypad, matrix, LCD and sensors come next
//! as other `Io` implementations (see the README).
#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use eso_core::{check, is_command, BracketError, Io, Machine};
use panic_halt as _;

/// Longest program the prompt accepts, in commands.
const PROGRAM_MAX: usize = 768;
/// Tape cells. Program + tape must leave room for the stack in 2 KB of RAM.
const TAPE: usize = 256;

type Serial = arduino_hal::hal::usart::Usart0<arduino_hal::DefaultClock>;

struct SerialIo<'a> {
    serial: &'a mut Serial,
}

impl Io for SerialIo<'_> {
    fn read(&mut self) -> u8 {
        self.serial.read_byte()
    }

    fn write(&mut self, byte: u8) {
        self.serial.write_byte(byte);
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut led = pins.d13.into_output();

    let mut program = [0u8; PROGRAM_MAX];
    let mut machine: Machine<TAPE> = Machine::new();

    ufmt::uwriteln!(&mut serial, "EsoUno: type Brainfuck, Enter runs it\r").unwrap_infallible();

    loop {
        ufmt::uwrite!(&mut serial, "> ").unwrap_infallible();

        // Read one line, keeping only the eight commands.
        let mut len = 0;
        let mut overflow = false;
        loop {
            let byte = serial.read_byte();
            if byte == b'\r' || byte == b'\n' {
                break;
            }
            serial.write_byte(byte); // echo
            if is_command(byte) {
                if len < PROGRAM_MAX {
                    program[len] = byte;
                    len += 1;
                } else {
                    overflow = true;
                }
            }
        }
        ufmt::uwriteln!(&mut serial, "\r").unwrap_infallible();

        let program = &program[..len];
        if overflow {
            ufmt::uwriteln!(&mut serial, "program too long (max {})\r", PROGRAM_MAX)
                .unwrap_infallible();
            continue;
        }
        match check(program) {
            Err(BracketError::UnmatchedOpen(i)) => {
                ufmt::uwriteln!(&mut serial, "'[' at {} is never closed\r", i).unwrap_infallible();
                continue;
            }
            Err(BracketError::UnmatchedClose(i)) => {
                ufmt::uwriteln!(&mut serial, "']' at {} has no '['\r", i).unwrap_infallible();
                continue;
            }
            Ok(()) => {}
        }

        led.set_high();
        machine.reset();
        machine.run(
            program,
            &mut SerialIo {
                serial: &mut serial,
            },
        );
        led.set_low();
        ufmt::uwriteln!(&mut serial, "\r").unwrap_infallible();
    }
}
