use std::{collections::HashMap, sync::LazyLock};

use colored::Colorize;
use mini_machine::machine::Machine;
use mini_machine::radbyte::RadixByte;
use num_enum::TryFromPrimitive;

type Pentyte = RadixByte<5, 3>;

const PENTYTE_TABLE: [[[char; 5]; 5]; 5] = [
    [
        ['\0', '\x01', '\x02', '\x03', '\x04'],
        ['\x05', '\x06', '\x07', '\x09', '\x0b'],
        ['\x0c', '\x0E', '\x0F', '\x10', '\x11'],
        ['\x08', '\x12', '\x13', '\x14', '\x15'],
        ['\n', '\r', '\x16', '\x17', '\x18'],
    ],
    [
        ['a', 'b', 'c', 'd', 'e'],
        ['f', 'g', 'h', 'i', 'j'],
        ['k', 'l', 'm', 'n', 'o'],
        ['p', 'q', 'r', 's', 't'],
        ['u', 'v', 'w', 'x', 'y'],
    ],
    [
        ['z', ' ', '~', '@', '?'],
        ['!', '"', '#', '$', '_'],
        ['%', '&', '\'', '(', ')'],
        ['*', '+', ',', '-', '.'],
        [':', ';', '<', '=', '>'],
    ],
    [
        ['A', 'B', 'C', 'D', 'E'],
        ['F', 'G', 'H', 'I', 'J'],
        ['K', 'L', 'M', 'N', 'O'],
        ['P', 'Q', 'R', 'S', 'T'],
        ['U', 'V', 'W', 'X', 'Y'],
    ],
    [
        ['Z', '\x19', '\x1a', '\x1b', '\x1c'],
        ['[', '\\', '/', ']', '^'],
        ['0', '1', '2', '3', '4'],
        ['5', '6', '7', '8', '9'],
        ['`', '{', '|', '}', '\x7f'],
    ],
];

static CHAR_TO_PENTYTE: LazyLock<HashMap<char, [u8; 3]>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for (page, rows) in PENTYTE_TABLE.iter().enumerate() {
        for (row, cols) in rows.iter().enumerate() {
            for (col, &c) in cols.iter().enumerate() {
                map.insert(c, [page as u8, row as u8, col as u8]);
            }
        }
    }
    map
});

fn pentyte2char(p: Pentyte) -> char {
    let [page, row, col] = p.digits();
    PENTYTE_TABLE[page as usize][row as usize][col as usize]
}

fn char2pentyte(c: char) -> Option<Pentyte> {
    CHAR_TO_PENTYTE.get(&c).map(|&d| Pentyte::from_digits(d))
}

fn addr(op: Pentyte) -> usize {
    (op.get() % 25) as usize
}

#[derive(PartialEq)]
struct DebugPentyte(Pentyte);

impl std::fmt::Debug for DebugPentyte {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let [hi, mi, lo] = self.0.digits();
        write!(f, "0p{hi}{mi}{lo}")
    }
}

impl std::fmt::Display for DebugPentyte {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.get())
    }
}

fn pentyte_inc(p: Pentyte) -> Pentyte {
    p.wrapping_add(Pentyte::new(1))
}

fn pentyte_dec(p: Pentyte) -> Pentyte {
    p.wrapping_sub(Pentyte::new(1))
}

fn pentyte_flip(p: Pentyte) -> Pentyte {
    Pentyte::from_digits(p.digits().map(|d| 4 - d))
}

struct Glyph(Pentyte);

impl std::fmt::Debug for Glyph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = pentyte2char(self.0);
        if c.is_ascii_graphic() || c == ' ' {
            write!(f, " '{c}' ")
        } else {
            write!(f, "{:?}", DebugPentyte(self.0))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u8)]
enum Opcode {
    DON = 0,  // x00
    PRT = 1,  // x01
    INC = 2,  // x02
    DEC = 3,  // x03
    FLP = 4,  // x04
    JMP = 5,  // x10
    JIZ = 6,  // x11
    JNZ = 7,  // x12
    ADD = 8,  // x13
    SUB = 9,  // x14
    MUL = 10, // x20
    DIV = 11, // x21
    JLT = 12, // x22
    JGT = 13, // x23
    JEQ = 14, // x24
    END = 24, // x44
}

impl Opcode {
    fn takes_operand(self) -> bool {
        !matches!(self, Opcode::DON | Opcode::END)
    }

    fn takes_operand2(self) -> bool {
        matches!(
            self,
            Opcode::JIZ
                | Opcode::JNZ
                | Opcode::ADD
                | Opcode::SUB
                | Opcode::MUL
                | Opcode::DIV
                | Opcode::JLT
                | Opcode::JGT
                | Opcode::JEQ
        )
    }

    fn takes_operand3(self) -> bool {
        matches!(self, Opcode::JLT | Opcode::JGT | Opcode::JEQ)
    }
}

struct Instruction {
    opcode: Opcode,
    operand: Option<Pentyte>,
    operand2: Option<Pentyte>,
    operand3: Option<Pentyte>,
}

impl Instruction {
    fn decode(mem: &[Pentyte], ip: &mut usize) -> Result<Self, String> {
        let mut next = || {
            let p = mem[*ip];
            *ip = (*ip + 1) % 25;
            p
        };

        let raw = next().get();
        let opcode = Opcode::try_from_primitive((raw % 25) as u8).map_err(|_| {
            let raw_p = Pentyte::new(raw);
            let masked_p = Pentyte::new(raw % 25);
            if pentyte2char(raw_p) == '\n' {
                format!(
                    "Expected a valid opcode but got {:?} ({:?})\nNote: your editor may have ended the file with a newline and your program is actually 24 characters.\nNote: in this case, just place a DON at the end.",
                    DebugPentyte(raw_p),
                    DebugPentyte(masked_p)
                )
            } else {
                format!(
                    "Expected a valid opcode but got {:?} ({:?})",
                    DebugPentyte(raw_p),
                    DebugPentyte(masked_p)
                )
            }
        })?;
        Ok(Instruction {
            opcode,
            operand: opcode.takes_operand().then(&mut next),
            operand2: opcode.takes_operand2().then(&mut next),
            operand3: opcode.takes_operand3().then(&mut next),
        })
    }
}

impl std::fmt::Debug for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.opcode)?;
        for operand in [self.operand, self.operand2, self.operand3]
            .into_iter()
            .flatten()
        {
            write!(
                f,
                " {:?}(#{:?})",
                DebugPentyte(operand),
                DebugPentyte(Pentyte::new(operand.get() % 25))
            )?;
        }
        Ok(())
    }
}

struct N3M25b5 {
    mem: Vec<Pentyte>,
}

impl N3M25b5 {
    fn format_mem(&self, highlight: Option<usize>) -> String {
        let mut out = String::new();
        for (i, p) in self.mem.iter().enumerate() {
            if i % 5 == 0 {
                out.push('\n');
                out.push_str(&format!("0{}x: ", i / 5));
            }
            let cell = format!("{:?}", Glyph(*p));
            if Some(i) == highlight {
                out.push_str(&format!("{} ", cell.black().on_white()));
            } else {
                out.push_str(&format!("{cell} "));
            }
        }
        out
    }
}

impl Machine for N3M25b5 {
    type Inst = Instruction;

    fn decode(&self, ip: &mut usize) -> Result<Self::Inst, String> {
        Instruction::decode(&self.mem, ip)
    }

    fn execute(&mut self, inst: Self::Inst, ip: &mut usize) -> bool {
        match inst.opcode {
            Opcode::DON => {}
            Opcode::PRT => print!("{}", pentyte2char(self.mem[addr(inst.operand.unwrap())])),
            Opcode::INC => {
                let a = addr(inst.operand.unwrap());
                self.mem[a] = pentyte_inc(self.mem[a]);
            }
            Opcode::DEC => {
                let a = addr(inst.operand.unwrap());
                self.mem[a] = pentyte_dec(self.mem[a]);
            }
            Opcode::FLP => {
                let a = addr(inst.operand.unwrap());
                self.mem[a] = pentyte_flip(self.mem[a]);
            }
            Opcode::JMP => *ip = addr(inst.operand.unwrap()),
            Opcode::JIZ => {
                if self.mem[addr(inst.operand2.unwrap())] == Pentyte::new(0) {
                    *ip = addr(inst.operand.unwrap());
                }
            }
            Opcode::JNZ => {
                if self.mem[addr(inst.operand2.unwrap())] != Pentyte::new(0) {
                    *ip = addr(inst.operand.unwrap());
                }
            }
            Opcode::ADD => {
                let a = addr(inst.operand.unwrap());
                self.mem[a] = self.mem[a].wrapping_add(inst.operand2.unwrap());
            }
            Opcode::SUB => {
                let a = addr(inst.operand.unwrap());
                self.mem[a] = self.mem[a].wrapping_sub(inst.operand2.unwrap());
            }
            Opcode::MUL => {
                let a = addr(inst.operand.unwrap());
                self.mem[a] = self.mem[a].wrapping_mul(inst.operand2.unwrap());
            }
            Opcode::DIV => {
                let a = addr(inst.operand.unwrap());
                self.mem[a] = match self.mem[a].checked_div(inst.operand2.unwrap()) {
                    Some(v) => v,
                    None => {
                        eprintln!("IP {}: Division by zero", self.format_ip(*ip));
                        std::process::exit(1);
                    }
                };
            }
            Opcode::JLT => {
                if self.mem[addr(inst.operand2.unwrap())] < inst.operand3.unwrap() {
                    *ip = addr(inst.operand.unwrap());
                }
            }
            Opcode::JGT => {
                if self.mem[addr(inst.operand2.unwrap())] > inst.operand3.unwrap() {
                    *ip = addr(inst.operand.unwrap());
                }
            }
            Opcode::JEQ => {
                if self.mem[addr(inst.operand2.unwrap())] == inst.operand3.unwrap() {
                    *ip = addr(inst.operand.unwrap());
                }
            }
            Opcode::END => return true,
        }
        false
    }

    fn debug_dump(&self, old_ip: usize) -> String {
        self.format_mem(Some(old_ip))
    }

    fn format_ip(&self, ip: usize) -> String {
        format!("{:?}", DebugPentyte(Pentyte::new(ip as u16)))
    }
}

fn main() {
    let mut args = std::env::args();
    let prog = args.next().unwrap();
    let Some(path) = args.next().map(std::path::PathBuf::from) else {
        eprintln!("Usage: {prog} <program> [--debug]");
        eprintln!("Expected a path");
        std::process::exit(1);
    };
    if !path.exists() || path.is_dir() {
        eprintln!("Usage: {prog} <program> [--debug]");
        eprintln!("Expected an existent file.");
        std::process::exit(1);
    }
    let dbg = args.next().map_or(false, |s| s == "--debug");
    let mem = match std::fs::read_to_string(path)
        .unwrap()
        .chars()
        .take(25)
        .map(|c| char2pentyte(c).ok_or(c))
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Invalid character for pentASCII: {e:?}");
            std::process::exit(1);
        }
    };
    if mem.len() != 25 {
        eprintln!("Expected 25 pentytes, got {}", mem.len());
        std::process::exit(1);
    }

    let machine = N3M25b5 { mem };

    if dbg {
        println!("{}", machine.format_mem(None));
        println!();
    }

    machine.run(dbg);
}
