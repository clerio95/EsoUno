# EsoUno

A standalone esoteric-language computer on an Arduino Uno R3 starter kit.
Brainfuck first; the sensors and displays of the kit become its input and output.

## Layout

| Path | What it is |
|---|---|
| `eso-core/` | `no_std` Brainfuck machine, shared by laptop and Uno. Tests live here. |
| `eso-cli/` | `eso` binary: runs a `.bf` file on the laptop with the same core. |
| `firmware/` | Uno firmware (`avr-hal`, nightly). Stage 1 is a Brainfuck prompt over USB serial. |
| `examples/` | Sample programs. |

## On the laptop

```fish
cargo test
cargo run --bin eso -- examples/hello.bf
```

## On the Uno

One-time setup (the AVR compiler and flasher are system packages):

```fish
sudo apt install gcc-avr avr-libc avrdude libudev-dev pkg-config
cargo install ravedude
sudo usermod -aG dialout $USER   # then log out and back in
```

Plug the board in by USB, then:

```fish
cd firmware
cargo run --release
```

This flashes the board and opens a serial console at 57600 baud. Type a program
and press Enter; `.` prints to the console and `,` reads a key from it. The
on-board LED is lit while a program runs. An endless program is stopped with the
reset button.

If the board is a clone with a CH340 USB chip it shows up as `/dev/ttyUSB0`
instead of `/dev/ttyACM0`; ravedude finds either.

## Limits

- Tape: 256 cells of `u8`, wrapping. Pointer wraps around the tape.
- Program: up to 768 commands at the serial prompt.
- Brackets are matched by scanning, not a jump table, to save RAM.

## Planned stages

1. **Serial prompt** (done, untested on hardware).
2. **Buzzer and ultrasonic I/O**: `.` plays a note, `,` reads distance, so
   `examples/theremin.bf` is a theremin.
3. **Live tape** on the 8x8 matrix, with single-step and a speed potentiometer.
4. **Keypad entry and LCD**, programs saved in EEPROM.
5. A custom 16-opcode language, a 2D language, the 8x8 raycaster demo.

## Pin plan (draft)

To be confirmed once the kit is on the desk, mainly whether the 8x8 matrix has a
MAX7219 driver board (3 pins) or is bare (16 pins, needs the 74HC595).

| Part | Pins |
|---|---|
| USB serial | D0, D1 |
| 4x4 keypad | D2 to D9 |
| 8x8 matrix (MAX7219) | D10 (CS), D11 (DIN), D13 (CLK) |
| Passive buzzer | D12 |
| LCD 16x2 (4-bit) | A0 to A5 |
| HC-SR04, joystick, potentiometer | share pins with the LCD or keypad by mode; not all fit at once |
