use std::{collections::HashMap, ops::*, sync::LazyLock};

/// Representation of [`Undigyte`], but separated into undigits
#[derive(Clone, Copy, PartialEq, Eq)]
struct Undigits {
    lo: u8,
    hi: u8,
}

impl Undigits {
    const fn as_undigyte(self) -> Undigyte {
        Undigyte(self.hi * 11 + self.lo)
    }
}

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

static CHAR_TO_UNDIGYTE: LazyLock<HashMap<char, (u8, u8)>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for (row, cols) in UNDIGYTE_TABLE.iter().enumerate() {
        for (col, &ch) in cols.iter().enumerate() {
            map.insert(ch, (row as u8, col as u8));
        }
    }
    map
});

/// A value in [0, 121)
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Undigyte(u8);

impl Undigyte {
    const MIN: Self = Undigyte(0);
    const MAX: Self = Undigyte(120);
    const MODULUS: u8 = 121;

    const fn as_undigits(self) -> Undigits {
        let lo = self.0 % 11;
        let hi = self.0 / 11;
        Undigits { lo, hi }
    }

    const fn as_char(self) -> char {
        let ud = self.as_undigits();
        UNDIGYTE_TABLE[ud.hi as usize][ud.lo as usize]
    }

    fn from_char(c: char) -> Option<Self> {
        CHAR_TO_UNDIGYTE
            .get(&c)
            .map(|tup| Undigyte(tup.0 * 11 + tup.1))
    }

    const fn wrapping_add(self, rhs: Self) -> Self {
        Undigyte((self.0 + rhs.0) % Self::MODULUS)
    }

    const fn wrapping_sub(self, rhs: Self) -> Self {
        // + MODULUS avoids underflow before the mod
        Undigyte((self.0 + Self::MODULUS - rhs.0) % Self::MODULUS)
    }

    const fn wrapping_mul(self, rhs: Self) -> Self {
        // u8 * u8 fits in u16, so no overflow before the mod
        Undigyte(((self.0 as u16 * rhs.0 as u16) % Self::MODULUS as u16) as u8)
    }

    const fn checked_div(self, rhs: Self) -> Option<Self> {
        if rhs.0 == 0 {
            return None;
        }
        Some(Undigyte((self.0 / rhs.0) % Self::MODULUS))
    }

    const fn checked_mod(self, rhs: Self) -> Option<Self> {
        if rhs.0 == 0 {
            return None;
        }
        Some(Undigyte(self.0 % rhs.0))
    }

    const fn wrapping_neg(self) -> Self {
        Undigyte((Self::MODULUS - self.0) % Self::MODULUS)
    }

    const fn as_signed(self) -> i8 {
        if self.0 <= 60 {
            self.0 as i8
        } else {
            self.0 as i8 - Self::MODULUS as i8
        }
    }
}

impl Add for Undigyte {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        self.wrapping_add(rhs)
    }
}

impl Sub for Undigyte {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        self.wrapping_sub(rhs)
    }
}

impl Mul for Undigyte {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        self.wrapping_mul(rhs)
    }
}

impl Neg for Undigyte {
    type Output = Self;
    fn neg(self) -> Self {
        self.wrapping_neg()
    }
}

impl std::fmt::Debug for Undigyte {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ud = self.as_undigits();
        match (ud.hi, ud.lo) {
            (10, 10) => write!(f, "0uXX"),
            (10, l) => write!(f, "0uX{l}"),
            (h, 10) => write!(f, "0u{h}X"),
            (h, l) => write!(f, "0u{h}{l}"),
        }
    }
}

impl std::fmt::Display for Undigyte {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
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

impl std::str::FromStr for Undigyte {
    type Err = ParseUndigyteError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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
            return Ok(Undigyte(hi * 11 + lo));
        }

        let n: u16 = s
            .parse()
            .map_err(|_| ParseUndigyteError(format!("'{s}' is not a valid number")))?;
        if n > 120 {
            return Err(ParseUndigyteError(format!("{n} out of range 0..=120")));
        }
        Ok(Undigyte(n as u8))
    }
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
            ra: Undigyte(0),
            rb: Undigyte(0),
            rc: Undigyte(0),
            rd: Undigyte(0),
            re: Undigyte(0),
            sp: Undigyte(120),
            bp: Undigyte(0),
        }
    }
}

impl std::fmt::Debug for Regs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "ra: {} ({:?})", self.ra, self.ra)?;
        writeln!(f, "rb: {} ({:?})", self.rb, self.rb)?;
        writeln!(f, "rc: {} ({:?})", self.rc, self.rc)?;
        writeln!(f, "rd: {} ({:?})", self.rd, self.rd)?;
        writeln!(f, "re: {} ({:?})", self.re, self.re)?;
        writeln!(f, "sp: {} ({:?})", self.sp, self.sp)?;
        write!(f, "bp: {} ({:?})", self.bp, self.bp)
    }
}

impl Index<Reg> for Regs {
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

impl IndexMut<Reg> for Regs {
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
        match op.0 {
            0 => Ok(Inst::NOP),
            1 => Ok(Inst::HLT),
            2 => {
                let regs = next();
                let dst = Reg::decode(regs.0 / 11)?;
                let src = Reg::decode(regs.0 % 11)?;
                Ok(Inst::MOV { dst, src })
            }
            3 => {
                let dst = Reg::decode(next().0 % 11)?;
                let src = next();
                Ok(Inst::IMOV { dst, src })
            }
            4 => {
                let regs = next();
                let dst = Reg::decode(regs.0 / 11)?;
                let src = Reg::decode(regs.0 % 11)?;
                Ok(Inst::ADD { dst, src })
            }
            5 => {
                let regs = next();
                let dst = Reg::decode(regs.0 / 11)?;
                let src = Reg::decode(regs.0 % 11)?;
                Ok(Inst::SUB { dst, src })
            }
            6 => {
                let regs = next();
                let dst = Reg::decode(regs.0 / 11)?;
                let src = Reg::decode(regs.0 % 11)?;
                Ok(Inst::MUL { dst, src })
            }
            7 => {
                let regs = next();
                let dst = Reg::decode(regs.0 / 11)?;
                let src = Reg::decode(regs.0 % 11)?;
                Ok(Inst::DIV { dst, src })
            }
            8 => {
                let regs = next();
                let dst = Reg::decode(regs.0 / 11)?;
                let src = Reg::decode(regs.0 % 11)?;
                Ok(Inst::MOD { dst, src })
            }
            9 => {
                let dst = Reg::decode(next().0 % 11)?;
                let src = next();
                Ok(Inst::IADD { dst, src })
            }
            10 => {
                let dst = Reg::decode(next().0 % 11)?;
                let src = next();
                Ok(Inst::ISUB { dst, src })
            }
            11 => {
                let dst = Reg::decode(next().0 % 11)?;
                let src = next();
                Ok(Inst::IMUL { dst, src })
            }
            12 => {
                let dst = Reg::decode(next().0 % 11)?;
                let src = next();
                Ok(Inst::IDIV { dst, src })
            }
            13 => {
                let dst = Reg::decode(next().0 % 11)?;
                let src = next();
                Ok(Inst::IMOD { dst, src })
            }
            15 => Ok(Inst::JMP { abs: next() }),
            16 => {
                let dst = Reg::decode(next().0 % 11)?;
                let abs = next();
                Ok(Inst::LDR { dst, abs })
            }
            17 => {
                let regs = next();
                let dst = Reg::decode(regs.0 / 11)?;
                let reg = Reg::decode(regs.0 % 11)?;
                Ok(Inst::LDRREG { dst, reg })
            }
            18 => {
                let abs = next();
                let src = Reg::decode(next().0 % 11)?;
                Ok(Inst::STR { abs, src })
            }
            19 => {
                let regs = next();
                let reg = Reg::decode(regs.0 / 11)?;
                let src = Reg::decode(regs.0 % 11)?;
                Ok(Inst::STRREG { reg, src })
            }
            20 => Ok(Inst::PUSH {
                src: Reg::decode(next().0 % 11)?,
            }),
            21 => Ok(Inst::POP {
                dst: Reg::decode(next().0 % 11)?,
            }),
            22 => Ok(Inst::CALL { abs: next() }),
            23 => Ok(Inst::RET),
            24 => Ok(Inst::JIZ {
                reg: Reg::decode(next().0 % 11)?,
                then: next(),
            }),
            25 => Ok(Inst::JNZ {
                reg: Reg::decode(next().0 % 11)?,
                then: next(),
            }),
            26 => {
                let regs = next();
                let a = Reg::decode(regs.0 / 11)?;
                let b = Reg::decode(regs.0 % 11)?;
                Ok(Inst::JEQ { a, b, then: next() })
            }
            27 => {
                let regs = next();
                let a = Reg::decode(regs.0 / 11)?;
                let b = Reg::decode(regs.0 % 11)?;
                Ok(Inst::JNE { a, b, then: next() })
            }
            28 => {
                let regs = next();
                let a = Reg::decode(regs.0 / 11)?;
                let b = Reg::decode(regs.0 % 11)?;
                Ok(Inst::JLT { a, b, then: next() })
            }
            29 => {
                let regs = next();
                let a = Reg::decode(regs.0 / 11)?;
                let b = Reg::decode(regs.0 % 11)?;
                Ok(Inst::JGT { a, b, then: next() })
            }
            30 => {
                let regs = next();
                let a = Reg::decode(regs.0 / 11)?;
                let b = Reg::decode(regs.0 % 11)?;
                Ok(Inst::JLTI { a, b, then: next() })
            }
            31 => {
                let regs = next();
                let a = Reg::decode(regs.0 / 11)?;
                let b = Reg::decode(regs.0 % 11)?;
                Ok(Inst::JGTI { a, b, then: next() })
            }
            32 => Ok(Inst::OUTU {
                reg: Reg::decode(next().0 % 11)?,
            }),
            33 => Ok(Inst::OUTD {
                reg: Reg::decode(next().0 % 11)?,
            }),
            34 => {
                let regs = next();
                let ptr = Reg::decode(regs.0 / 11)?;
                let len = Reg::decode(regs.0 % 11)?;
                Ok(Inst::OUTS { ptr, len })
            }
            35 => Ok(Inst::IOUTS {
                ptr: Reg::decode(next().0 % 11)?,
                len: next(),
            }),
            _ => Err(format!("Invalid opcode number: {op:?}")),
        }
    }
}

impl std::fmt::Debug for Inst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Inst::NOP => write!(f, "nop"),
            Inst::HLT => write!(f, "hlt"),
            Inst::MOV { dst, src } => write!(f, "mov {dst:?}, {src:?}"),
            Inst::IMOV { dst, src } => write!(f, "mov {dst:?}, {src} ({src:?})"),
            Inst::ADD { dst, src } => write!(f, "add {dst:?}, {src:?}"),
            Inst::SUB { dst, src } => write!(f, "sub {dst:?}, {src:?}"),
            Inst::MUL { dst, src } => write!(f, "mul {dst:?}, {src:?}"),
            Inst::DIV { dst, src } => write!(f, "div {dst:?}, {src:?}"),
            Inst::MOD { dst, src } => write!(f, "mod {dst:?}, {src:?}"),
            Inst::IADD { dst, src } => write!(f, "add {dst:?}, {src} ({src:?})"),
            Inst::ISUB { dst, src } => write!(f, "sub {dst:?}, {src} ({src:?})"),
            Inst::IMUL { dst, src } => write!(f, "mul {dst:?}, {src} ({src:?})"),
            Inst::IDIV { dst, src } => write!(f, "div {dst:?}, {src} ({src:?})"),
            Inst::IMOD { dst, src } => write!(f, "mod {dst:?}, {src} ({src:?})"),
            Inst::JMP { abs } => write!(f, "jmp {abs:?}"),
            Inst::LDR { dst, abs } => write!(f, "ldr {dst:?}, {abs:?}"),
            Inst::LDRREG { dst, reg } => write!(f, "ldr {dst:?}, {reg:?}"),
            Inst::STR { abs, src } => write!(f, "str {abs:?}, {src:?}"),
            Inst::STRREG { reg, src } => write!(f, "str {reg:?}, {src:?}"),
            Inst::PUSH { src } => write!(f, "push {src:?}"),
            Inst::POP { dst } => write!(f, "push {dst:?}"),
            Inst::CALL { abs } => write!(f, "call {abs:?}"),
            Inst::RET => write!(f, "ret"),
            Inst::JIZ { reg, then } => write!(f, "jiz {reg:?}, {then:?}"),
            Inst::JNZ { reg, then } => write!(f, "jnz {reg:?}, {then:?}"),
            Inst::JEQ { a, b, then } => write!(f, "jeq {a:?}, {b:?}, {then:?}"),
            Inst::JNE { a, b, then } => write!(f, "jne {a:?}, {b:?}, {then:?}"),
            Inst::JLT { a, b, then } => write!(f, "jlt {a:?}, {b:?}, {then:?}"),
            Inst::JGT { a, b, then } => write!(f, "jgt {a:?}, {b:?}, {then:?}"),
            Inst::JLTI { a, b, then } => write!(f, "jlti {a:?}, {b:?}, {then:?}"),
            Inst::JGTI { a, b, then } => write!(f, "jgti {a:?}, {b:?}, {then:?}"),
            Inst::OUTU { reg } => write!(f, "outu {reg:?}"),
            Inst::OUTD { reg } => write!(f, "outd {reg:?}"),
            Inst::OUTS { ptr, len } => write!(f, "outs {ptr:?}, {len:?}"),
            Inst::IOUTS { ptr, len } => write!(f, "outs {ptr:?}, {len} ({len:?})"),
        }
    }
}

fn parse_program(source: &str) -> Result<Vec<Undigyte>, String> {
    let mut mem = Vec::new();
    for (line_no, raw_line) in source.lines().enumerate() {
        let line = raw_line.split(';').next().unwrap_or("").trim();
        for tok in line.split_whitespace() {
            let n: Undigyte = tok
                .parse()
                .map_err(|_| format!("line {}: invalid byte '{tok}'", line_no + 1))?;
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

fn run(mem: &mut [Undigyte], debug: bool) {
    let mut regs = Regs::default();
    let mut ip = 0;
    let mut halted = false;
    while !halted {
        let old_ip = ip;
        let inst = match Inst::decode(mem, &mut ip) {
            Ok(o) => o,
            Err(e) => {
                eprintln!("IP {:?}: {e}", Undigyte(old_ip as u8));
                std::process::exit(1);
            }
        };
        if debug {
            println!("IP {:?}: {inst:?}", Undigyte(old_ip as u8));
        }
        match inst {
            Inst::NOP => {}
            Inst::HLT => halted = true,
            Inst::MOV { dst, src } => regs[dst] = regs[src],
            Inst::IMOV { dst, src } => regs[dst] = src,
            Inst::ADD { dst, src } => regs[dst] = regs[dst] + regs[src],
            Inst::SUB { dst, src } => regs[dst] = regs[dst] - regs[src],
            Inst::MUL { dst, src } => regs[dst] = regs[dst] * regs[src],
            Inst::DIV { dst, src } => {
                regs[dst] = match regs[dst].checked_div(regs[src]) {
                    Some(v) => v,
                    None => {
                        eprintln!("IP {:?}: Division by zero", Undigyte(old_ip as u8));
                        std::process::exit(1);
                    }
                }
            }
            Inst::MOD { dst, src } => {
                regs[dst] = match regs[dst].checked_mod(regs[src]) {
                    Some(v) => v,
                    None => {
                        eprintln!("IP {:?}: Division by zero", Undigyte(old_ip as u8));
                        std::process::exit(1);
                    }
                }
            }
            Inst::IADD { dst, src } => regs[dst] = regs[dst] + src,
            Inst::ISUB { dst, src } => regs[dst] = regs[dst] - src,
            Inst::IMUL { dst, src } => regs[dst] = regs[dst] * src,
            Inst::IDIV { dst, src } => {
                regs[dst] = match regs[dst].checked_div(src) {
                    Some(v) => v,
                    None => {
                        eprintln!("IP {:?}: Division by zero", Undigyte(old_ip as u8));
                        std::process::exit(1);
                    }
                }
            }
            Inst::IMOD { dst, src } => {
                regs[dst] = match regs[dst].checked_mod(src) {
                    Some(v) => v,
                    None => {
                        eprintln!("IP {:?}: Division by zero", Undigyte(old_ip as u8));
                        std::process::exit(1);
                    }
                }
            }
            Inst::JMP { abs } => ip = abs.0 as usize,
            Inst::LDR { dst, abs } => regs[dst] = mem[abs.0 as usize],
            Inst::LDRREG { dst, reg } => regs[dst] = mem[regs[reg].0 as usize],
            Inst::STR { abs, src } => mem[abs.0 as usize] = regs[src],
            Inst::STRREG { reg, src } => mem[regs[reg].0 as usize] = regs[src],
            Inst::PUSH { src } => {
                regs.sp = regs.sp - Undigyte(1);
                mem[regs.sp.0 as usize] = regs[src];
            }
            Inst::POP { dst } => {
                if regs.sp.0 == 120 {
                    eprintln!("IP {:?}: Stack underflow", Undigyte(old_ip as u8));
                    std::process::exit(1);
                }
                regs[dst] = mem[regs.sp.0 as usize];
                regs.sp = regs.sp + Undigyte(1);
            }
            Inst::CALL { abs } => {
                regs.sp = regs.sp - Undigyte(1);
                mem[regs.sp.0 as usize].0 = ip as u8;
                ip = abs.0 as usize;
            }
            Inst::RET => {
                if regs.sp.0 == 120 {
                    eprintln!("IP {:?}: Stack underflow", Undigyte(old_ip as u8));
                    std::process::exit(1);
                }
                ip = mem[regs.sp.0 as usize].0 as usize;
                regs.sp = regs.sp + Undigyte(1);
            }
            Inst::JIZ { reg, then } => {
                if regs[reg].0 == 0 {
                    ip = then.0 as usize;
                }
            }
            Inst::JNZ { reg, then } => {
                if regs[reg].0 != 0 {
                    ip = then.0 as usize;
                }
            }
            Inst::JEQ { a, b, then } => {
                if regs[a] == regs[b] {
                    ip = then.0 as usize;
                }
            }
            Inst::JNE { a, b, then } => {
                if regs[a] != regs[b] {
                    ip = then.0 as usize;
                }
            }
            Inst::JLT { a, b, then } => {
                if regs[a] < regs[b] {
                    ip = then.0 as usize;
                }
            }
            Inst::JGT { a, b, then } => {
                if regs[a] > regs[b] {
                    ip = then.0 as usize;
                }
            }
            Inst::JLTI { a, b, then } => {
                if regs[a].as_signed() < regs[b].as_signed() {
                    ip = then.0 as usize;
                }
            }
            Inst::JGTI { a, b, then } => {
                if regs[a].as_signed() > regs[b].as_signed() {
                    ip = then.0 as usize;
                }
            }
            Inst::OUTU { reg } => print!("{:?}", regs[reg]),
            Inst::OUTD { reg } => print!("{}", regs[reg]),
            Inst::OUTS { ptr, len } => {
                let ptr = regs[ptr].0 as usize;
                let len = regs[len].0 as usize;
                for i in 0..len {
                    let abs = (ptr + i) % 121;
                    print!("{}", mem[abs].as_char());
                }
            }
            Inst::IOUTS { ptr, len } => {
                let ptr = regs[ptr].0 as usize;
                let len = len.0 as usize;
                for i in 0..len {
                    let abs = (ptr + i) % 121;
                    print!("{}", mem[abs].as_char());
                }
            }
        }
        if debug {
            println!("\n{regs:?}");
            if !halted {
                let mut cmd = String::new();
                std::io::stdin().read_line(&mut cmd).unwrap();
                match cmd.to_lowercase().as_str() {
                    "quit" | "q" | "exit" | "halt" | "h" => {
                        println!("User requested halt");
                        halted = true;
                    }
                    _ => {}
                }
            }
        }
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
    mem.resize(121, Undigyte(0));

    run(&mut mem, debug);
}
