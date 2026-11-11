# EsoUno

A standalone esoteric-language computer built from an Arduino Uno R3 starter kit.
Brainfuck first, with the kit's sensors and displays as the language's input and output.
The README has the layout, setup commands, limits and staged plan; read it first.

## State

- `eso-core` and `eso-cli` work and are tested on the laptop.
- `firmware` (stage 1, Brainfuck prompt over USB serial) type-checks for AVR but has
  never been linked, flashed or run on a board. Treat it as unverified until it has.
- Nothing from the kit is wired yet. The pin plan in the README is a draft.

## Commands

- Laptop: `cargo test`, `cargo clippy --all-targets`, `cargo run --bin eso -- examples/hello.bf`
  from the repo root (stable toolchain).
- Firmware: run cargo from `firmware/`. It is excluded from the workspace, pins
  nightly through `rust-toolchain.toml`, and builds `core` for `avr-none`.
  `cargo check` works without the AVR compiler; `cargo build` and `cargo run`
  need `gcc-avr`, `avr-libc`, `avrdude` and `ravedude`.
- Clippy is not installed for nightly, so the firmware is only checked with `cargo check`.

## Design rules

- All language logic lives in `eso-core`, which is `no_std` with no allocation, so
  it can be tested on the laptop. The firmware only supplies `Io` implementations
  and hardware glue.
- New hardware (buzzer, ultrasonic sensor, keypad, matrix, LCD) is added as an
  `Io` implementation or a display of machine state, not as changes to the interpreter.
- The Uno has 2 KB of RAM, 1 KB of EEPROM and 32 KB of flash. Program buffer plus
  tape must leave room for the stack; brackets are matched by scanning for that reason.
- `arduino-hal` is a git dependency pinned by revision. Keep it pinned.

## The kit

Uno R3 clone (ATmega328P, 16 MHz) with USB cable, breadboard and jumper wires, plus:

- Displays: LCD 16x2, 8x8 LED matrix, 1-digit and 4-digit 7-segment displays, LEDs.
- Input: 4x4 keypad, joystick, tactile buttons, 10K potentiometer, IR remote and receiver.
- Sensors: HC-SR04 ultrasonic, DHT11, LM35, thermistor, microphone, vibration switch,
  water level, RC522 RFID reader.
- Output: passive and active buzzers, SG90 servo, stepper with ULN2003 driver, 5V relay.
- Other: DS1302 clock, 74HC595 shift register, resistors (220R, 1K, 10K).

Open questions until the kit is in hand:

- Whether the 8x8 matrix has a MAX7219 driver board (3 pins) or is bare (16 pins).
- Whether the board's USB chip is a CH340 (`/dev/ttyUSB0`) or an ATmega16U2 (`/dev/ttyACM0`).

Hardware cautions: the RC522 runs on 3.3 V, not 5 V. The relay is for low-voltage loads only.

## Ideas parked for later

A custom 16-opcode language for the keypad, a 2D Befunge-style language on the
8x8 matrix, a Subleq machine, and an 8x8 raycaster demo.
