//! A small Brainfuck machine with no allocation and no `std`.
//!
//! The same crate runs on the laptop (tests, `eso-cli`) and on the Uno.
//! The program is a byte slice of ASCII commands; any other byte is a comment.
//! Cells are `u8` and wrap, and the data pointer wraps around the tape.
#![cfg_attr(not(test), no_std)]

/// Where `,` reads from and `.` writes to. On the Uno these become the
/// keypad, sensors, LCD, buzzer and so on.
pub trait Io {
    fn read(&mut self) -> u8;
    fn write(&mut self, byte: u8);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Running,
    Halted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BracketError {
    /// A `[` at this program index has no matching `]`.
    UnmatchedOpen(usize),
    /// A `]` at this program index has no matching `[`.
    UnmatchedClose(usize),
}

/// True for the eight Brainfuck commands.
pub fn is_command(byte: u8) -> bool {
    matches!(byte, b'>' | b'<' | b'+' | b'-' | b'.' | b',' | b'[' | b']')
}

/// Checks that every bracket has a partner. Run this before stepping.
pub fn check(program: &[u8]) -> Result<(), BracketError> {
    let mut depth = 0usize;
    let mut last_open = 0usize;
    for (i, &b) in program.iter().enumerate() {
        match b {
            b'[' => {
                if depth == 0 {
                    last_open = i;
                }
                depth += 1;
            }
            b']' => {
                if depth == 0 {
                    return Err(BracketError::UnmatchedClose(i));
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    if depth == 0 {
        Ok(())
    } else {
        Err(BracketError::UnmatchedOpen(last_open))
    }
}

/// A machine with a tape of `N` cells. It does not own the program, so the
/// firmware can keep the program in one buffer and step at its own pace.
pub struct Machine<const N: usize> {
    tape: [u8; N],
    ptr: usize,
    pc: usize,
}

impl<const N: usize> Default for Machine<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Machine<N> {
    pub const fn new() -> Self {
        Self {
            tape: [0; N],
            ptr: 0,
            pc: 0,
        }
    }

    pub fn reset(&mut self) {
        self.tape = [0; N];
        self.ptr = 0;
        self.pc = 0;
    }

    pub fn tape(&self) -> &[u8; N] {
        &self.tape
    }

    /// Index of the current cell.
    pub fn ptr(&self) -> usize {
        self.ptr
    }

    /// Index of the next program byte to execute.
    pub fn pc(&self) -> usize {
        self.pc
    }

    /// Executes one command (skipping comments first). A program with
    /// unbalanced brackets halts at the bad bracket; use [`check`] up front
    /// to report it properly.
    pub fn step(&mut self, program: &[u8], io: &mut impl Io) -> Status {
        while self.pc < program.len() && !is_command(program[self.pc]) {
            self.pc += 1;
        }
        let Some(&op) = program.get(self.pc) else {
            return Status::Halted;
        };
        match op {
            b'>' => self.ptr = (self.ptr + 1) % N,
            b'<' => self.ptr = (self.ptr + N - 1) % N,
            b'+' => self.tape[self.ptr] = self.tape[self.ptr].wrapping_add(1),
            b'-' => self.tape[self.ptr] = self.tape[self.ptr].wrapping_sub(1),
            b'.' => io.write(self.tape[self.ptr]),
            b',' => self.tape[self.ptr] = io.read(),
            b'[' if self.tape[self.ptr] == 0 => match matching_close(program, self.pc) {
                Some(close) => self.pc = close,
                None => return self.halt(program),
            },
            b']' if self.tape[self.ptr] != 0 => match matching_open(program, self.pc) {
                Some(open) => self.pc = open,
                None => return self.halt(program),
            },
            _ => {}
        }
        self.pc += 1;
        Status::Running
    }

    /// Runs until the program ends.
    pub fn run(&mut self, program: &[u8], io: &mut impl Io) {
        while self.step(program, io) == Status::Running {}
    }

    fn halt(&mut self, program: &[u8]) -> Status {
        self.pc = program.len();
        Status::Halted
    }
}

// Brackets are matched by scanning instead of a jump table: slower, but it
// costs no RAM, and the Uno only has 2 KB.
fn matching_close(program: &[u8], open: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (i, &b) in program.iter().enumerate().skip(open) {
        match b {
            b'[' => depth += 1,
            b']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

fn matching_open(program: &[u8], close: usize) -> Option<usize> {
    let mut depth = 0usize;
    for i in (0..=close).rev() {
        match program[i] {
            b']' => depth += 1,
            b'[' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Buf {
        input: Vec<u8>,
        output: Vec<u8>,
    }

    impl Io for Buf {
        fn read(&mut self) -> u8 {
            if self.input.is_empty() {
                0
            } else {
                self.input.remove(0)
            }
        }
        fn write(&mut self, byte: u8) {
            self.output.push(byte);
        }
    }

    fn run(program: &str, input: &[u8]) -> Vec<u8> {
        let mut io = Buf {
            input: input.to_vec(),
            output: Vec::new(),
        };
        let mut m: Machine<256> = Machine::new();
        check(program.as_bytes()).unwrap();
        m.run(program.as_bytes(), &mut io);
        io.output
    }

    #[test]
    fn hello_world() {
        let p = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";
        assert_eq!(run(p, b""), b"Hello World!\n");
    }

    #[test]
    fn cat_until_zero() {
        assert_eq!(run(",[.,]", b"abc"), b"abc");
    }

    #[test]
    fn comments_are_ignored() {
        assert_eq!(run("add three\n+++ then print\n.", b""), [3]);
    }

    #[test]
    fn cells_and_pointer_wrap() {
        assert_eq!(run("-.", b""), [255]);
        assert_eq!(run("<+.>.", b""), [1, 0]);
        let mut m: Machine<4> = Machine::new();
        let mut io = Buf {
            input: vec![],
            output: vec![],
        };
        m.run(b"<", &mut io);
        assert_eq!(m.ptr(), 3);
    }

    #[test]
    fn nested_loops_multiply() {
        // 6 * 7 = 42
        assert_eq!(run("++++++[>+++++++<-]>.", b""), [42]);
    }

    #[test]
    fn skipped_loop_with_nesting() {
        assert_eq!(run("[[-]+[.]]+.", b""), [1]);
    }

    #[test]
    fn bracket_check() {
        assert_eq!(check(b"+[-]"), Ok(()));
        assert_eq!(check(b"+[[-]"), Err(BracketError::UnmatchedOpen(1)));
        assert_eq!(check(b"+]"), Err(BracketError::UnmatchedClose(1)));
    }

    #[test]
    fn unbalanced_program_halts_instead_of_panicking() {
        let mut m: Machine<8> = Machine::new();
        let mut io = Buf {
            input: vec![],
            output: vec![],
        };
        m.run(b"[+", &mut io);
        m.reset();
        m.run(b"+]", &mut io);
    }

    #[test]
    fn step_reports_progress() {
        let mut m: Machine<8> = Machine::new();
        let mut io = Buf {
            input: vec![],
            output: vec![],
        };
        assert_eq!(m.step(b"+", &mut io), Status::Running);
        assert_eq!(m.tape()[0], 1);
        assert_eq!(m.pc(), 1);
        assert_eq!(m.step(b"+", &mut io), Status::Halted);
    }
}
