use itertools::Itertools;
use pretty_hex::*;
use std::env;
use std::fs;

fn main() -> Result<(), std::io::Error> {
    println!("carter-emu v{}", env!("CARGO_PKG_VERSION"));

    // load program from file into memory
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("usage: ce <filename>");
    }

    let mut in_bytes: Vec<u8> = fs::read(&args[1])?;
    in_bytes.pop(); // remove trailing newline
    let instructions = Instruction::from_chars(in_bytes.into_iter());

    // prep the memory by loading the instructions starting at 0x00, then extending to 256 bytes
    let mut mem: Vec<u8> = instructions
        .into_iter()
        .chunks(4)
        .into_iter()
        .map(|c| Instruction::to_byte(c.collect()))
        .collect();
    if mem.len() > 256 {
        panic!("Error: program length exceeds 256 bytes");
    }
    mem.resize(256, 0);

    execute(&mut mem);

    println!("{}", pretty_hex(&mem));

    Ok(())
}

fn execute(mem: &mut Vec<u8>) {
    // prep registers
    let mut dp = 0; // data pointer
    let mut ipl = 0; // instruction pointer low
    let mut iph = 0; // instruction pointer high
    let mut rpl = 0; // return pointer low
    let mut rph = 0; // return pointer high

    loop {
        if ipl == 256 {
            return;
        }
        let ins = Instruction::from_byte(mem[ipl]);

        loop {
            let inst = ins[iph];
            match inst {
                Instruction::LoopOpen => {
                    rpl = ipl;
                    rph = iph + 1;
                    if rph > 3 {
                        rph = 0;
                        rpl += 1;
                    }
                }
                Instruction::LoopClose => {
                    if mem[dp] != 0 {
                        ipl = rpl;
                        iph = rph;
                        break;
                    }
                }
                Instruction::Increment => {
                    mem[dp] += 1;
                }
                Instruction::ShiftRight => {
                    dp += 1;
                }
            }

            iph += 1;
            if iph > 3 {
                iph = 0;
                ipl += 1;
                break;
            }
        }
    }
}

#[derive(Copy, Clone)]
enum Instruction {
    LoopOpen,
    LoopClose,
    Increment,
    ShiftRight,
}

use std::fmt;
impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Instruction::LoopOpen => '[',
                Instruction::LoopClose => ']',
                Instruction::Increment => '+',
                Instruction::ShiftRight => '>',
            }
        )
    }
}

impl Instruction {
    #[allow(dead_code)]
    fn from_byte(byte: u8) -> Vec<Instruction> {
        // split into 2-bit pairs before mapping
        [
            (byte & 0b11000000) >> 6,
            (byte & 0b00110000) >> 4,
            (byte & 0b00001100) >> 2,
            (byte & 0b00000011),
        ]
        .into_iter()
        .map(|b| match b {
            0b00 => Instruction::LoopOpen,
            0b01 => Instruction::LoopClose,
            0b10 => Instruction::Increment,
            0b11 => Instruction::ShiftRight,
            _ => panic!("if you got here, you invented a new type of number"),
        })
        .collect()
    }

    fn from_chars(chars: impl Iterator<Item = u8>) -> impl Iterator<Item = Instruction> {
        chars.map(|c| match c {
            b'[' => Instruction::LoopOpen,
            b']' => Instruction::LoopClose,
            b'+' => Instruction::Increment,
            b'>' => Instruction::ShiftRight,
            _ => panic!("Invalid instruction '{}'", c),
        })
    }

    fn to_pair(i: Option<&Instruction>) -> u8 {
        match i {
            Some(Instruction::LoopOpen) => 0b00,
            Some(Instruction::LoopClose) => 0b01,
            Some(Instruction::Increment) => 0b10,
            Some(Instruction::ShiftRight) => 0b11,
            None => 0b00,
        }
    }

    fn to_byte(is: Vec<Instruction>) -> u8 {
        Instruction::to_pair(is.get(0)) << 6
            | Instruction::to_pair(is.get(1)) << 4
            | Instruction::to_pair(is.get(2)) << 2
            | Instruction::to_pair(is.get(3))
    }
}
