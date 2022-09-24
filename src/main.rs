use clap::Parser;
use itertools::Itertools;
use pretty_hex::*;
use std::fs;
use std::io::{stdin, Read};

#[derive(Parser)]
#[clap(version, about)]
struct Args {
    /// Path to file to execute
    #[clap(value_parser)]
    path: String,

    /// Set debug level
    /// -d for stepping
    /// -dd for debug info at step
    #[clap(short, action = clap::ArgAction::Count)]
    debug: u8,
}

fn main() -> Result<(), std::io::Error> {
    println!("carter-emu v{}", env!("CARGO_PKG_VERSION"));

    // load program from file into memory
    let args = Args::parse();

    let mut in_bytes: Vec<u8> = fs::read(&args.path)?;
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

    execute(&mut mem, args.debug);

    println!("{}", pretty_hex(&mem));

    Ok(())
}

fn execute(mem: &mut Vec<u8>, step: u8) {
    // prep registers
    let mut dp = 0; // data pointer
    let mut ipl = 0; // instruction pointer low
    let mut iph = 0; // instruction pointer high
    let mut rpl = 0; // return pointer low
    let mut rph = 0; // return pointer high

    let mut stdin = stdin();

    loop {
        if ipl == 256 {
            return;
        }
        let ins = Instruction::from_byte(mem[ipl]);

        loop {
            let inst = ins[iph];

            if step > 2 {
                print_state(ipl, iph, dp, mem, rpl, rph, inst);
            }
            if step > 1 {
                let _ = stdin.read(&mut [0u8]).unwrap();
            }

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

fn print_state(
    ipl: usize,
    iph: usize,
    dp: usize,
    mem: &Vec<u8>,
    rpl: usize,
    rph: usize,
    inst: Instruction,
) {
    println!(
        "ipl: {:2X} = {}
iph: {:2X}
 dp: {:2X} = {:2X}
rpl: {:2X} = {}
rph: {:2X}
>>> {}",
        ipl,
        Instruction::display_byte(mem[ipl]),
        iph,
        dp,
        mem[dp],
        rpl,
        Instruction::display_byte(mem[rpl]),
        rph,
        inst
    );
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

    fn display_byte(byte: u8) -> String {
        let is = Instruction::from_byte(byte);
        format!("{} {} {} {}", is[0], is[1], is[2], is[3])
    }
}
