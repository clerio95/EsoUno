use std::io::{Read, Write};
use std::process::ExitCode;

use eso_core::{check, BracketError, Io, Machine};

/// Tape size of the Uno build, so programs behave the same on both.
const TAPE: usize = 256;

struct Stdio {
    stdin: std::io::Stdin,
    stdout: std::io::Stdout,
}

impl Io for Stdio {
    fn read(&mut self) -> u8 {
        let _ = self.stdout.flush();
        let mut byte = [0u8];
        match self.stdin.read(&mut byte) {
            Ok(1) => byte[0],
            _ => 0, // end of input reads as zero
        }
    }

    fn write(&mut self, byte: u8) {
        let _ = self.stdout.write_all(&[byte]);
    }
}

fn main() -> ExitCode {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("usage: eso <program.bf>");
        return ExitCode::from(2);
    };
    let program = match std::fs::read(&path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("eso: {path}: {e}");
            return ExitCode::FAILURE;
        }
    };
    if let Err(e) = check(&program) {
        match e {
            BracketError::UnmatchedOpen(i) => eprintln!("eso: '[' at byte {i} is never closed"),
            BracketError::UnmatchedClose(i) => eprintln!("eso: ']' at byte {i} has no '['"),
        }
        return ExitCode::FAILURE;
    }

    let mut io = Stdio {
        stdin: std::io::stdin(),
        stdout: std::io::stdout(),
    };
    let mut machine: Machine<TAPE> = Machine::new();
    machine.run(&program, &mut io);
    let _ = io.stdout.flush();
    ExitCode::SUCCESS
}
