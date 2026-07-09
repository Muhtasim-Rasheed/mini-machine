use colored::Colorize;
use num_enum::TryFromPrimitive;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Pentits {
    lo: u8,
    mi: u8,
    hi: u8,
}

/// [0, 125)
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Pentyte(u8);

impl Pentyte {
    const MIN: Self = Pentyte(0);
    const MAX: Self = Pentyte(124);

    const fn from_pentits(pentits: Pentits) -> Self {
        Pentyte((pentits.hi * 5 + pentits.mi) * 5 + pentits.lo)
    }

    fn inc(&mut self) {
        self.0 += 1;
        if *self > Self::MAX {
            self.0 = 0;
        }
    }

    fn dec(&mut self) {
        if *self == Self::MIN {
            self.0 = 124;
        }
        self.0 = self.0.wrapping_sub(1);
    }

    fn add(&mut self, rhs: Pentyte) {
        self.0 = ((u16::from(self.0) + u16::from(rhs.0)) % 125) as u8;
    }

    fn sub(&mut self, rhs: Pentyte) {
        self.0 = ((125 + self.0 as u16 - rhs.0 as u16) % 125) as u8;
    }

    fn mul(&mut self, rhs: Pentyte) {
        self.0 = ((u16::from(self.0) * u16::from(rhs.0)) % 125) as u8;
    }

    fn div(&mut self, rhs: Pentyte) {
        self.0 = ((u16::from(self.0) / u16::from(rhs.0)) % 125) as u8;
    }

    fn flp(&mut self) {
        let Pentits { lo, mi, hi } = (*self).into();
        *self = Pentits {
            lo: 4 - lo,
            mi: 4 - mi,
            hi: 4 - hi,
        }
        .into();
    }
}

impl From<Pentyte> for Pentits {
    fn from(pentyte: Pentyte) -> Self {
        let (d2, pentyte) = (pentyte.0 / 25, pentyte.0 % 25);
        let (d1, d0) = (pentyte / 5, pentyte % 5);
        Pentits {
            lo: d0,
            mi: d1,
            hi: d2,
        }
    }
}

impl From<Pentits> for Pentyte {
    fn from(pentits: Pentits) -> Self {
        Self::from_pentits(pentits)
    }
}

macro_rules! pentascii_table {
    () => {
        convert!(
            '\0' => 000, '\x01' => 001, '\x02' => 002, '\x03' => 003, '\x04' => 004,
            '\x05' => 010, '\x06' => 011, '\x07' => 012, '\x09' => 013, '\x0B' => 014,
            '\x0C' => 020, '\x0E' => 021, '\x0F' => 022, '\x10' => 023, '\x11' => 024,
            '\x08' => 030, '\x12' => 031, '\x13' => 032, '\x14' => 033, '\x15' => 034,
            '\n' => 040, '\r' => 041, '\x16' => 042, '\x17' => 043, '\x18' => 044,
            'a' => 100, 'b' => 101, 'c' => 102, 'd' => 103, 'e' => 104,
            'f' => 110, 'g' => 111, 'h' => 112, 'i' => 113, 'j' => 114,
            'k' => 120, 'l' => 121, 'm' => 122, 'n' => 123, 'o' => 124,
            'p' => 130, 'q' => 131, 'r' => 132, 's' => 133, 't' => 134,
            'u' => 140, 'v' => 141, 'w' => 142, 'x' => 143, 'y' => 144,
            'z' => 200, ' ' => 201, '~' => 202, '@' => 203, '?' => 204,
            '!' => 210, '"' => 211, '#' => 212, '$' => 213, '_' => 214,
            '%' => 220, '&' => 221, '\'' => 222, '(' => 223, ')' => 224,
            '*' => 230, '+' => 231, ',' => 232, '-' => 233, '.' => 234,
            ':' => 240, ';' => 241, '<' => 242, '=' => 243, '>' => 244,
            'A' => 300, 'B' => 301, 'C' => 302, 'D' => 303, 'E' => 304,
            'F' => 310, 'G' => 311, 'H' => 312, 'I' => 313, 'J' => 314,
            'K' => 320, 'L' => 321, 'M' => 322, 'N' => 323, 'O' => 324,
            'P' => 330, 'Q' => 331, 'R' => 332, 'S' => 333, 'T' => 334,
            'U' => 340, 'V' => 341, 'W' => 342, 'X' => 343, 'Y' => 344,
            'Z' => 400, '\x19' => 401, '\x1A' => 402, '\x1B' => 403, '\x1C' => 404,
            '[' => 410, '\\' => 411, '/' => 412, ']' => 413, '^' => 414,
            '0' => 420, '1' => 421, '2' => 422, '3' => 423, '4' => 424,
            '5' => 430, '6' => 431, '7' => 432, '8' => 433, '9' => 434,
            '`' => 440, '{' => 441, '|' => 442, '}' => 443, '\x7F' => 444,
        )
    };
}

impl TryFrom<char> for Pentyte {
    type Error = char;
    fn try_from(c: char) -> Result<Self, Self::Error> {
        macro_rules! convert {
            ($($ch:literal => $d:literal),* $(,)?) => {
                match c {
                    $(
                        $ch => Ok(const { Pentyte::from_pentits(Pentits { hi: ($d / 100) as u8, mi: ($d / 10 % 10) as u8, lo: ($d % 10) as u8 }) }),
                    )*
                    _ => Err(c),
                }
            };
        }

        pentascii_table!()
    }
}

impl From<Pentyte> for char {
    fn from(p: Pentyte) -> Self {
        macro_rules! convert {
            ($($ch:literal => $d:literal),* $(,)?) => {
                match p {
                    $(
                        p if const { Pentyte::from_pentits(Pentits { hi: ($d / 100) as u8, mi: ($d / 10 % 10) as u8, lo: ($d % 10) as u8 }) } == p => $ch,
                    )*
                    p => unreachable!("somehow reached {p:?} ({p})"),
                }
            };
        }

        pentascii_table!()
    }
}

impl std::fmt::Debug for Pentyte {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Pentits { lo, mi, hi } = (*self).into();
        write!(f, "0p{hi}{mi}{lo}")
    }
}

impl std::fmt::Display for Pentyte {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

struct Glyph(Pentyte);

impl std::fmt::Debug for Glyph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = char::from(self.0);
        if c.is_ascii_graphic() || c == ' ' {
            write!(f, " '{c}' ")
        } else {
            write!(f, "{:?}", self.0)
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

        let raw = next().0;
        let opcode = Opcode::try_from_primitive(raw % 25).map_err(|_| {
            if char::from(Pentyte(raw)) == '\n' {
                format!(
                    "Expected a valid opcode but got {:?} ({:?})\nNote: your editor may have ended the file with a newline and your program is actually 24 characters.\nNote: in this case, just place a DON at the end.",
                    Pentyte(raw),
                    Pentyte(raw % 25)
                )
            } else {
                format!(
                    "Expected a valid opcode but got {:?} ({:?})",
                    Pentyte(raw),
                    Pentyte(raw % 25)
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
    let mut mem = match std::fs::read_to_string(path)
        .unwrap()
        .chars()
        .take(25)
        .map(|c| Pentyte::try_from(c))
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
    if dbg {
        for (i, p) in mem.iter().enumerate() {
            if i % 5 == 0 {
                println!();
                print!("0{}x: ", i / 5);
            }
            print!("{:?} ", Glyph(*p));
        }
        println!();
        println!();
    }
    let mut ip = 0;
    let mut halt = false;
    let mut dbg_input = String::new();
    while !halt {
        let old_ip = ip;
        if dbg {
            print!("[Executing] IP {:?}: ", Pentyte(ip as u8));
        }
        let inst = match Instruction::decode(&mem, &mut ip) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Illegal instruction: {e}");
                std::process::exit(1);
            }
        };
        if dbg {
            if let (Some(operand), Some(operand2), Some(operand3)) =
                (inst.operand, inst.operand2, inst.operand3)
            {
                println!(
                    "{:?} {operand:?}(#{:?}) {operand2:?}(#{:?}) {operand3:?}(#{:?})",
                    inst.opcode,
                    Pentyte(operand.0 % 25),
                    Pentyte(operand2.0 % 25),
                    Pentyte(operand3.0 % 25),
                );
            } else if let (Some(operand), Some(operand2)) = (inst.operand, inst.operand2) {
                println!(
                    "{:?} {operand:?}(#{:?}) {operand2:?}(#{:?})",
                    inst.opcode,
                    Pentyte(operand.0 % 25),
                    Pentyte(operand2.0 % 25),
                );
            } else if let Some(operand) = inst.operand {
                println!(
                    "{:?} {operand:?}(#{:?})",
                    inst.opcode,
                    Pentyte(operand.0 % 25)
                );
            } else {
                println!("{:?}", inst.opcode);
            }
        }
        match inst.opcode {
            Opcode::DON => {}
            Opcode::PRT => print!("{}", char::from(mem[inst.operand.unwrap().0 as usize % 25])),
            Opcode::INC => mem[inst.operand.unwrap().0 as usize % 25].inc(),
            Opcode::DEC => mem[inst.operand.unwrap().0 as usize % 25].dec(),
            Opcode::FLP => mem[inst.operand.unwrap().0 as usize % 25].flp(),
            Opcode::JMP => ip = inst.operand.unwrap().0 as usize % 25,
            Opcode::JIZ => {
                if mem[inst.operand2.unwrap().0 as usize % 25] == Pentyte(0) {
                    ip = inst.operand.unwrap().0 as usize % 25;
                }
            }
            Opcode::JNZ => {
                if mem[inst.operand2.unwrap().0 as usize % 25] != Pentyte(0) {
                    ip = inst.operand.unwrap().0 as usize % 25;
                }
            }
            Opcode::ADD => mem[inst.operand.unwrap().0 as usize % 25].add(inst.operand2.unwrap()),
            Opcode::SUB => mem[inst.operand.unwrap().0 as usize % 25].sub(inst.operand2.unwrap()),
            Opcode::MUL => mem[inst.operand.unwrap().0 as usize % 25].mul(inst.operand2.unwrap()),
            Opcode::DIV => mem[inst.operand.unwrap().0 as usize % 25].div(inst.operand2.unwrap()),
            Opcode::JLT => {
                let addr = inst.operand2.unwrap().0 as usize % 25;
                if mem[addr] < inst.operand3.unwrap() {
                    ip = inst.operand.unwrap().0 as usize % 25;
                }
            }
            Opcode::JGT => {
                let addr = inst.operand2.unwrap().0 as usize % 25;
                if mem[addr] > inst.operand3.unwrap() {
                    ip = inst.operand.unwrap().0 as usize % 25;
                }
            }
            Opcode::JEQ => {
                let addr = inst.operand2.unwrap().0 as usize % 25;
                if mem[addr] == inst.operand3.unwrap() {
                    ip = inst.operand.unwrap().0 as usize % 25;
                }
            }
            Opcode::END => halt = true,
        }
        if dbg {
            for (i, p) in mem.iter().enumerate() {
                if i % 5 == 0 {
                    println!();
                    print!("0{}x: ", i / 5);
                }
                if i == old_ip {
                    print!("{} ", format!("{:?}", Glyph(*p)).black().on_white());
                } else {
                    print!("{} ", format!("{:?}", Glyph(*p)));
                }
            }
            println!();
            dbg_input.clear();
            std::io::stdin().read_line(&mut dbg_input).unwrap();
            if dbg_input.trim().to_lowercase() == "q" {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pentyte_conversion_test() {
        for i in 0..125 {
            assert_eq!(Pentyte(i), <Pentits>::from(Pentyte(i)).into());
        }
        for i in 0..125 {
            assert_eq!(Ok(Pentyte(i)), Pentyte::try_from(char::from(Pentyte(i))));
        }
    }
}
