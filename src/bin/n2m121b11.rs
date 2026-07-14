use std::{collections::HashMap, io::Write, sync::LazyLock};

use mini_machine::machine::Machine;
use mini_machine::radbyte::RadixByte;

type Undigyte = RadixByte<11, 2>;

#[rustfmt::skip]
const UNDIGYTE_TABLE: [[char; 11]; 11] = [
    ['\0', '\n', '\r', '\u{A0}', '¢', '£', '¥', '§', '¶', '«', '»'],
    [' ', '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*'],
    ['+', ',', '-', '.', '/', ':', ';', '<', '=', '>', '?'],
    ['@', '[', '\\', ']', '^', '_', '`', '{', '|', '}', '~'],
    ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K'],
    ['L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V'],
    ['W', 'X', 'Y', 'Z', '¡', '¿', '™', '…', '·', 'µ', '¬'],
    ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k'],
    ['l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v'],
    ['w', 'x', 'y', 'z', '¼', '½', '¾', '¹', '²', '³', '¦'],
    ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '±'],
];

static CHAR_TO_UNDIGYTE: LazyLock<HashMap<char, [u8; 2]>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for (hi, cols) in UNDIGYTE_TABLE.iter().enumerate() {
        for (lo, &ch) in cols.iter().enumerate() {
            map.insert(ch, [hi as u8, lo as u8]);
        }
    }
    map
});

fn undigyte2char(u: Undigyte) -> char {
    let [hi, lo] = u.digits();
    UNDIGYTE_TABLE[hi as usize][lo as usize]
}

fn char2undigyte(c: char) -> Option<Undigyte> {
    CHAR_TO_UNDIGYTE.get(&c).map(|&d| Undigyte::from_digits(d))
}

struct DebugUndigyte(Undigyte);

impl std::fmt::Debug for DebugUndigyte {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let [hi, lo] = self.0.digits();
        let digit = |d: u8| if d == 10 { 'X' } else { (b'0' + d) as char };
        write!(f, "0u{}{}", digit(hi), digit(lo))
    }
}

impl std::fmt::Display for DebugUndigyte {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.get())
    }
}

#[derive(Debug)]
struct ParseUndigyteError(String);

impl std::fmt::Display for ParseUndigyteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid undigyte: {}", self.0)
    }
}

impl std::error::Error for ParseUndigyteError {}

fn parse_undigyte(s: &str) -> Result<Undigyte, ParseUndigyteError> {
    let s = s.trim();

    if let Some(digits) = s.strip_prefix("0u").or_else(|| s.strip_prefix("0U")) {
        let mut chars = digits.chars();
        let (Some(hi), Some(lo), None) = (chars.next(), chars.next(), chars.next()) else {
            return Err(ParseUndigyteError(format!(
                "'{s}' must have exactly 2 undigits after 0u"
            )));
        };
        let parse_undigit = |c: char| match c {
            '0'..='9' => Ok(c as u8 - b'0'),
            'X' | 'x' => Ok(10),
            _ => Err(ParseUndigyteError(format!(
                "invalid undigit '{c}' in '{s}'"
            ))),
        };
        let hi = parse_undigit(hi)?;
        let lo = parse_undigit(lo)?;
        return Ok(Undigyte::from_digits([hi, lo]));
    }

    let n: u16 = s
        .parse()
        .map_err(|_| ParseUndigyteError(format!("'{s}' is not a valid number")))?;
    if n > 120 {
        return Err(ParseUndigyteError(format!("{n} out of range 0..=120")));
    }
    Ok(Undigyte::new(n))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Reg {
    RA,
    RB,
    RC,
    RD,
    RE,
    SP,
    BP,
}

impl Reg {
    fn decode(n: u8) -> Result<Self, String> {
        match n {
            0 => Ok(Reg::RA),
            1 => Ok(Reg::RB),
            2 => Ok(Reg::RC),
            3 => Ok(Reg::RD),
            4 => Ok(Reg::RE),
            5 => Ok(Reg::SP),
            6 => Ok(Reg::BP),
            10 => Err("Invalid register id: X".to_string()),
            r => Err(format!("Invalid register id: {r}")),
        }
    }
}

impl std::fmt::Debug for Reg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RA => write!(f, "ra"),
            Self::RB => write!(f, "rb"),
            Self::RC => write!(f, "rc"),
            Self::RD => write!(f, "rd"),
            Self::RE => write!(f, "re"),
            Self::SP => write!(f, "sp"),
            Self::BP => write!(f, "bp"),
        }
    }
}

struct Regs {
    ra: Undigyte,
    rb: Undigyte,
    rc: Undigyte,
    rd: Undigyte,
    re: Undigyte,
    sp: Undigyte,
    bp: Undigyte,
}

impl Default for Regs {
    fn default() -> Self {
        Regs {
            ra: Undigyte::new(0),
            rb: Undigyte::new(0),
            rc: Undigyte::new(0),
            rd: Undigyte::new(0),
            re: Undigyte::new(0),
            sp: Undigyte::new(120),
            bp: Undigyte::new(0),
        }
    }
}

impl std::fmt::Debug for Regs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "ra: {} ({:?})",
            DebugUndigyte(self.ra),
            DebugUndigyte(self.ra)
        )?;
        writeln!(
            f,
            "rb: {} ({:?})",
            DebugUndigyte(self.rb),
            DebugUndigyte(self.rb)
        )?;
        writeln!(
            f,
            "rc: {} ({:?})",
            DebugUndigyte(self.rc),
            DebugUndigyte(self.rc)
        )?;
        writeln!(
            f,
            "rd: {} ({:?})",
            DebugUndigyte(self.rd),
            DebugUndigyte(self.rd)
        )?;
        writeln!(
            f,
            "re: {} ({:?})",
            DebugUndigyte(self.re),
            DebugUndigyte(self.re)
        )?;
        writeln!(
            f,
            "sp: {} ({:?})",
            DebugUndigyte(self.sp),
            DebugUndigyte(self.sp)
        )?;
        write!(
            f,
            "bp: {} ({:?})",
            DebugUndigyte(self.bp),
            DebugUndigyte(self.bp)
        )
    }
}

impl std::ops::Index<Reg> for Regs {
    type Output = Undigyte;
    fn index(&self, r: Reg) -> &Self::Output {
        match r {
            Reg::RA => &self.ra,
            Reg::RB => &self.rb,
            Reg::RC => &self.rc,
            Reg::RD => &self.rd,
            Reg::RE => &self.re,
            Reg::SP => &self.sp,
            Reg::BP => &self.bp,
        }
    }
}

impl std::ops::IndexMut<Reg> for Regs {
    fn index_mut(&mut self, r: Reg) -> &mut Self::Output {
        match r {
            Reg::RA => &mut self.ra,
            Reg::RB => &mut self.rb,
            Reg::RC => &mut self.rc,
            Reg::RD => &mut self.rd,
            Reg::RE => &mut self.re,
            Reg::SP => &mut self.sp,
            Reg::BP => &mut self.bp,
        }
    }
}

enum Inst {
    NOP,
    HLT,
    MOV { dst: Reg, src: Reg },
    IMOV { dst: Reg, src: Undigyte },
    ADD { dst: Reg, src: Reg },
    SUB { dst: Reg, src: Reg },
    MUL { dst: Reg, src: Reg },
    DIV { dst: Reg, src: Reg },
    MOD { dst: Reg, src: Reg },
    IADD { dst: Reg, src: Undigyte },
    ISUB { dst: Reg, src: Undigyte },
    IMUL { dst: Reg, src: Undigyte },
    IDIV { dst: Reg, src: Undigyte },
    IMOD { dst: Reg, src: Undigyte },
    JMP { abs: Undigyte },
    LDR { dst: Reg, abs: Undigyte },
    LDRREG { dst: Reg, reg: Reg },
    STR { abs: Undigyte, src: Reg },
    STRREG { reg: Reg, src: Reg },
    PUSH { src: Reg },
    POP { dst: Reg },
    CALL { abs: Undigyte },
    RET,
    JIZ { reg: Reg, then: Undigyte },
    JNZ { reg: Reg, then: Undigyte },
    JEQ { a: Reg, b: Reg, then: Undigyte },
    JNE { a: Reg, b: Reg, then: Undigyte },
    JLT { a: Reg, b: Reg, then: Undigyte },
    JGT { a: Reg, b: Reg, then: Undigyte },
    JLTI { a: Reg, b: Reg, then: Undigyte },
    JGTI { a: Reg, b: Reg, then: Undigyte },
    OUTU { reg: Reg },
    OUTD { reg: Reg },
    OUTS { ptr: Reg, len: Reg },
    IOUTS { ptr: Reg, len: Undigyte },
}

impl Inst {
    fn decode(mem: &[Undigyte], ip: &mut usize) -> Result<Self, String> {
        let mut next = || {
            let u = mem[*ip];
            *ip = (*ip + 1) % 121;
            u
        };

        let op = next();
        match op.get() {
            0 => Ok(Inst::NOP),
            1 => Ok(Inst::HLT),
            2 => {
                let regs = next().get();
                let dst = Reg::decode((regs / 11) as u8)?;
                let src = Reg::decode((regs % 11) as u8)?;
                Ok(Inst::MOV { dst, src })
            }
            3 => {
                let dst = Reg::decode((next().get() % 11) as u8)?;
                let src = next();
                Ok(Inst::IMOV { dst, src })
            }
            4 => {
                let regs = next().get();
                let dst = Reg::decode((regs / 11) as u8)?;
                let src = Reg::decode((regs % 11) as u8)?;
                Ok(Inst::ADD { dst, src })
            }
            5 => {
                let regs = next().get();
                let dst = Reg::decode((regs / 11) as u8)?;
                let src = Reg::decode((regs % 11) as u8)?;
                Ok(Inst::SUB { dst, src })
            }
            6 => {
                let regs = next().get();
                let dst = Reg::decode((regs / 11) as u8)?;
                let src = Reg::decode((regs % 11) as u8)?;
                Ok(Inst::MUL { dst, src })
            }
            7 => {
                let regs = next().get();
                let dst = Reg::decode((regs / 11) as u8)?;
                let src = Reg::decode((regs % 11) as u8)?;
                Ok(Inst::DIV { dst, src })
            }
            8 => {
                let regs = next().get();
                let dst = Reg::decode((regs / 11) as u8)?;
                let src = Reg::decode((regs % 11) as u8)?;
                Ok(Inst::MOD { dst, src })
            }
            9 => {
                let dst = Reg::decode((next().get() % 11) as u8)?;
                let src = next();
                Ok(Inst::IADD { dst, src })
            }
            10 => {
                let dst = Reg::decode((next().get() % 11) as u8)?;
                let src = next();
                Ok(Inst::ISUB { dst, src })
            }
            11 => {
                let dst = Reg::decode((next().get() % 11) as u8)?;
                let src = next();
                Ok(Inst::IMUL { dst, src })
            }
            12 => {
                let dst = Reg::decode((next().get() % 11) as u8)?;
                let src = next();
                Ok(Inst::IDIV { dst, src })
            }
            13 => {
                let dst = Reg::decode((next().get() % 11) as u8)?;
                let src = next();
                Ok(Inst::IMOD { dst, src })
            }
            15 => Ok(Inst::JMP { abs: next() }),
            16 => {
                let dst = Reg::decode((next().get() % 11) as u8)?;
                let abs = next();
                Ok(Inst::LDR { dst, abs })
            }
            17 => {
                let regs = next().get();
                let dst = Reg::decode((regs / 11) as u8)?;
                let reg = Reg::decode((regs % 11) as u8)?;
                Ok(Inst::LDRREG { dst, reg })
            }
            18 => {
                let abs = next();
                let src = Reg::decode((next().get() % 11) as u8)?;
                Ok(Inst::STR { abs, src })
            }
            19 => {
                let regs = next().get();
                let reg = Reg::decode((regs / 11) as u8)?;
                let src = Reg::decode((regs % 11) as u8)?;
                Ok(Inst::STRREG { reg, src })
            }
            20 => Ok(Inst::PUSH {
                src: Reg::decode((next().get() % 11) as u8)?,
            }),
            21 => Ok(Inst::POP {
                dst: Reg::decode((next().get() % 11) as u8)?,
            }),
            22 => Ok(Inst::CALL { abs: next() }),
            23 => Ok(Inst::RET),
            24 => Ok(Inst::JIZ {
                reg: Reg::decode((next().get() % 11) as u8)?,
                then: next(),
            }),
            25 => Ok(Inst::JNZ {
                reg: Reg::decode((next().get() % 11) as u8)?,
                then: next(),
            }),
            26 => {
                let regs = next().get();
                let a = Reg::decode((regs / 11) as u8)?;
                let b = Reg::decode((regs % 11) as u8)?;
                Ok(Inst::JEQ { a, b, then: next() })
            }
            27 => {
                let regs = next().get();
                let a = Reg::decode((regs / 11) as u8)?;
                let b = Reg::decode((regs % 11) as u8)?;
                Ok(Inst::JNE { a, b, then: next() })
            }
            28 => {
                let regs = next().get();
                let a = Reg::decode((regs / 11) as u8)?;
                let b = Reg::decode((regs % 11) as u8)?;
                Ok(Inst::JLT { a, b, then: next() })
            }
            29 => {
                let regs = next().get();
                let a = Reg::decode((regs / 11) as u8)?;
                let b = Reg::decode((regs % 11) as u8)?;
                Ok(Inst::JGT { a, b, then: next() })
            }
            30 => {
                let regs = next().get();
                let a = Reg::decode((regs / 11) as u8)?;
                let b = Reg::decode((regs % 11) as u8)?;
                Ok(Inst::JLTI { a, b, then: next() })
            }
            31 => {
                let regs = next().get();
                let a = Reg::decode((regs / 11) as u8)?;
                let b = Reg::decode((regs % 11) as u8)?;
                Ok(Inst::JGTI { a, b, then: next() })
            }
            32 => Ok(Inst::OUTU {
                reg: Reg::decode((next().get() % 11) as u8)?,
            }),
            33 => Ok(Inst::OUTD {
                reg: Reg::decode((next().get() % 11) as u8)?,
            }),
            34 => {
                let regs = next().get();
                let ptr = Reg::decode((regs / 11) as u8)?;
                let len = Reg::decode((regs % 11) as u8)?;
                Ok(Inst::OUTS { ptr, len })
            }
            35 => Ok(Inst::IOUTS {
                ptr: Reg::decode((next().get() % 11) as u8)?,
                len: next(),
            }),
            _ => Err(format!("Invalid opcode number: {:?}", DebugUndigyte(op))),
        }
    }
}

impl std::fmt::Debug for Inst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let u = |x: Undigyte| DebugUndigyte(x);
        match self {
            Inst::NOP => write!(f, "nop"),
            Inst::HLT => write!(f, "hlt"),
            Inst::MOV { dst, src } => write!(f, "mov {dst:?}, {src:?}"),
            Inst::IMOV { dst, src } => write!(f, "mov {dst:?}, {} ({:?})", u(*src), u(*src)),
            Inst::ADD { dst, src } => write!(f, "add {dst:?}, {src:?}"),
            Inst::SUB { dst, src } => write!(f, "sub {dst:?}, {src:?}"),
            Inst::MUL { dst, src } => write!(f, "mul {dst:?}, {src:?}"),
            Inst::DIV { dst, src } => write!(f, "div {dst:?}, {src:?}"),
            Inst::MOD { dst, src } => write!(f, "mod {dst:?}, {src:?}"),
            Inst::IADD { dst, src } => write!(f, "add {dst:?}, {} ({:?})", u(*src), u(*src)),
            Inst::ISUB { dst, src } => write!(f, "sub {dst:?}, {} ({:?})", u(*src), u(*src)),
            Inst::IMUL { dst, src } => write!(f, "mul {dst:?}, {} ({:?})", u(*src), u(*src)),
            Inst::IDIV { dst, src } => write!(f, "div {dst:?}, {} ({:?})", u(*src), u(*src)),
            Inst::IMOD { dst, src } => write!(f, "mod {dst:?}, {} ({:?})", u(*src), u(*src)),
            Inst::JMP { abs } => write!(f, "jmp {:?}", u(*abs)),
            Inst::LDR { dst, abs } => write!(f, "ldr {dst:?}, {:?}", u(*abs)),
            Inst::LDRREG { dst, reg } => write!(f, "ldr {dst:?}, {reg:?}"),
            Inst::STR { abs, src } => write!(f, "str {:?}, {src:?}", u(*abs)),
            Inst::STRREG { reg, src } => write!(f, "str {reg:?}, {src:?}"),
            Inst::PUSH { src } => write!(f, "push {src:?}"),
            Inst::POP { dst } => write!(f, "push {dst:?}"),
            Inst::CALL { abs } => write!(f, "call {:?}", u(*abs)),
            Inst::RET => write!(f, "ret"),
            Inst::JIZ { reg, then } => write!(f, "jiz {reg:?}, {:?}", u(*then)),
            Inst::JNZ { reg, then } => write!(f, "jnz {reg:?}, {:?}", u(*then)),
            Inst::JEQ { a, b, then } => write!(f, "jeq {a:?}, {b:?}, {:?}", u(*then)),
            Inst::JNE { a, b, then } => write!(f, "jne {a:?}, {b:?}, {:?}", u(*then)),
            Inst::JLT { a, b, then } => write!(f, "jlt {a:?}, {b:?}, {:?}", u(*then)),
            Inst::JGT { a, b, then } => write!(f, "jgt {a:?}, {b:?}, {:?}", u(*then)),
            Inst::JLTI { a, b, then } => write!(f, "jlti {a:?}, {b:?}, {:?}", u(*then)),
            Inst::JGTI { a, b, then } => write!(f, "jgti {a:?}, {b:?}, {:?}", u(*then)),
            Inst::OUTU { reg } => write!(f, "outu {reg:?}"),
            Inst::OUTD { reg } => write!(f, "outd {reg:?}"),
            Inst::OUTS { ptr, len } => write!(f, "outs {ptr:?}, {len:?}"),
            Inst::IOUTS { ptr, len } => write!(f, "outs {ptr:?}, {} ({:?})", u(*len), u(*len)),
        }
    }
}

fn parse_program(source: &str) -> Result<Vec<Undigyte>, String> {
    let mut mem = Vec::new();
    for (line_no, raw_line) in source.lines().enumerate() {
        let line = raw_line.split(';').next().unwrap_or("").trim();
        for tok in line.split_whitespace() {
            let n = parse_undigyte(tok).map_err(|e| format!("line {}: {e}", line_no + 1))?;
            mem.push(n);
        }
    }
    if mem.len() > 121 {
        return Err(format!(
            "program is {} undigytes, exceeds 121-cell memory",
            mem.len()
        ));
    }
    Ok(mem)
}

struct N2M121b11 {
    mem: Vec<Undigyte>,
    regs: Regs,
}

impl N2M121b11 {
    fn fail(&self, ip: usize, msg: &str) -> ! {
        eprintln!("IP {}: {msg}", self.format_ip(ip));
        std::process::exit(1);
    }
}

impl Machine for N2M121b11 {
    type Inst = Inst;

    fn decode(&self, ip: &mut usize) -> Result<Self::Inst, String> {
        Inst::decode(&self.mem, ip)
    }

    fn execute(&mut self, inst: Self::Inst, ip: &mut usize) -> bool {
        let old_ip = *ip;
        let regs = &mut self.regs;
        let mem = &mut self.mem;
        match inst {
            Inst::NOP => {}
            Inst::HLT => return true,
            Inst::MOV { dst, src } => regs[dst] = regs[src],
            Inst::IMOV { dst, src } => regs[dst] = src,
            Inst::ADD { dst, src } => regs[dst] = regs[dst].wrapping_add(regs[src]),
            Inst::SUB { dst, src } => regs[dst] = regs[dst].wrapping_sub(regs[src]),
            Inst::MUL { dst, src } => regs[dst] = regs[dst].wrapping_mul(regs[src]),
            Inst::DIV { dst, src } => {
                regs[dst] = match regs[dst].checked_div(regs[src]) {
                    Some(v) => v,
                    None => self.fail(old_ip, "Division by zero"),
                }
            }
            Inst::MOD { dst, src } => {
                regs[dst] = match regs[dst].checked_mod(regs[src]) {
                    Some(v) => v,
                    None => self.fail(old_ip, "Division by zero"),
                }
            }
            Inst::IADD { dst, src } => regs[dst] = regs[dst].wrapping_add(src),
            Inst::ISUB { dst, src } => regs[dst] = regs[dst].wrapping_sub(src),
            Inst::IMUL { dst, src } => regs[dst] = regs[dst].wrapping_mul(src),
            Inst::IDIV { dst, src } => {
                regs[dst] = match regs[dst].checked_div(src) {
                    Some(v) => v,
                    None => self.fail(old_ip, "Division by zero"),
                }
            }
            Inst::IMOD { dst, src } => {
                regs[dst] = match regs[dst].checked_mod(src) {
                    Some(v) => v,
                    None => self.fail(old_ip, "Division by zero"),
                }
            }
            Inst::JMP { abs } => *ip = abs.get() as usize,
            Inst::LDR { dst, abs } => regs[dst] = mem[abs.get() as usize],
            Inst::LDRREG { dst, reg } => regs[dst] = mem[regs[reg].get() as usize],
            Inst::STR { abs, src } => mem[abs.get() as usize] = regs[src],
            Inst::STRREG { reg, src } => mem[regs[reg].get() as usize] = regs[src],
            Inst::PUSH { src } => {
                regs.sp = regs.sp.wrapping_sub(Undigyte::new(1));
                mem[regs.sp.get() as usize] = regs[src];
            }
            Inst::POP { dst } => {
                if regs.sp.get() == 120 {
                    self.fail(old_ip, "Stack underflow");
                }
                regs[dst] = mem[regs.sp.get() as usize];
                regs.sp = regs.sp.wrapping_add(Undigyte::new(1));
            }
            Inst::CALL { abs } => {
                regs.sp = regs.sp.wrapping_sub(Undigyte::new(1));
                mem[regs.sp.get() as usize] = Undigyte::new(*ip as u16);
                *ip = abs.get() as usize;
            }
            Inst::RET => {
                if regs.sp.get() == 120 {
                    self.fail(old_ip, "Stack underflow");
                }
                *ip = mem[regs.sp.get() as usize].get() as usize;
                regs.sp = regs.sp.wrapping_add(Undigyte::new(1));
            }
            Inst::JIZ { reg, then } => {
                if regs[reg].get() == 0 {
                    *ip = then.get() as usize;
                }
            }
            Inst::JNZ { reg, then } => {
                if regs[reg].get() != 0 {
                    *ip = then.get() as usize;
                }
            }
            Inst::JEQ { a, b, then } => {
                if regs[a] == regs[b] {
                    *ip = then.get() as usize;
                }
            }
            Inst::JNE { a, b, then } => {
                if regs[a] != regs[b] {
                    *ip = then.get() as usize;
                }
            }
            Inst::JLT { a, b, then } => {
                if regs[a] < regs[b] {
                    *ip = then.get() as usize;
                }
            }
            Inst::JGT { a, b, then } => {
                if regs[a] > regs[b] {
                    *ip = then.get() as usize;
                }
            }
            Inst::JLTI { a, b, then } => {
                if regs[a].as_signed() < regs[b].as_signed() {
                    *ip = then.get() as usize;
                }
            }
            Inst::JGTI { a, b, then } => {
                if regs[a].as_signed() > regs[b].as_signed() {
                    *ip = then.get() as usize;
                }
            }
            Inst::OUTU { reg } => print!("{:?}", DebugUndigyte(regs[reg])),
            Inst::OUTD { reg } => print!("{}", DebugUndigyte(regs[reg])),
            Inst::OUTS { ptr, len } => {
                let ptr = regs[ptr].get() as usize;
                let len = regs[len].get() as usize;
                for i in 0..len {
                    print!("{}", undigyte2char(mem[(ptr + i) % 121]));
                }
            }
            Inst::IOUTS { ptr, len } => {
                let ptr = regs[ptr].get() as usize;
                let len = len.get() as usize;
                for i in 0..len {
                    print!("{}", undigyte2char(mem[(ptr + i) % 121]));
                }
            }
        }
        std::io::stdout().flush().ok();
        false
    }

    fn debug_dump(&self, _old_ip: usize) -> String {
        format!("{:?}", self.regs)
    }

    fn format_ip(&self, ip: usize) -> String {
        format!("{:?}", DebugUndigyte(Undigyte::new(ip as u16)))
    }
}

fn main() {
    let mut path: Option<String> = None;
    let mut debug = false;
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--debug" => debug = true,
            other => path = Some(other.to_string()),
        }
    }

    let Some(path) = path else {
        eprintln!("usage: undigyte [--debug] <program-file>");
        std::process::exit(1);
    };

    let source = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("failed to read '{path}': {e}");
            std::process::exit(1);
        }
    };

    let mut mem = match parse_program(&source) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("parse error: {e}");
            std::process::exit(1);
        }
    };
    mem.resize(121, Undigyte::new(0));

    let machine = N2M121b11 {
        mem,
        regs: Regs::default(),
    };

    machine.run(debug);
}
