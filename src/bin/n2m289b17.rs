use std::{collections::HashMap, sync::LazyLock};

use mini_machine::{machine::Machine, radbyte::RadixByte};

type Septendigyte = RadixByte<17, 2>;

struct DebugSeptendigyte(Septendigyte);

impl std::fmt::Debug for DebugSeptendigyte {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const DIGITS: [char; 17] = [
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G',
        ];
        let [hi, lo] = self.0.digits();
        write!(f, "0s{}{}", DIGITS[hi as usize], DIGITS[lo as usize])
    }
}

impl std::fmt::Display for DebugSeptendigyte {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.get())
    }
}

static CHAR_ENCODING: LazyLock<[[char; 17]; 17]> = LazyLock::new(|| {
    const SYMBOLS: [char; 33] = [
        '═', '║', '╔', '╗', '╚', '╝', '╠', '╣', '╦', '╩', '╬', '█', '▀', '▄', '▌', '▐', '∴', '─',
        '│', '┌', '┐', '└', '┘', '├', '┤', '┬', '┴', '┼', '∵', '≤', '≥', '≠', '≈',
    ];
    let mut grid = [['\0'; 17]; 17];
    for i in 0..289usize {
        let ch = if i < 256 {
            char::from(i as u8)
        } else {
            SYMBOLS[i - 256]
        };
        grid[i / 17][i % 17] = ch;
    }
    grid
});

static CHAR_TO_SEPTENDIGYTE: LazyLock<HashMap<char, [u8; 2]>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for (hi, cols) in CHAR_ENCODING.iter().enumerate() {
        for (lo, &ch) in cols.iter().enumerate() {
            map.insert(ch, [hi as u8, lo as u8]);
        }
    }
    map
});

fn septendigyte2char(u: Septendigyte) -> char {
    let [hi, lo] = u.digits();
    CHAR_ENCODING[hi as usize][lo as usize]
}

fn char2septendigyte(c: char) -> Option<Septendigyte> {
    CHAR_TO_SEPTENDIGYTE
        .get(&c)
        .map(|&d| Septendigyte::from_digits(d))
}

#[derive(Debug)]
struct ParseSeptendigyteError(String);

impl std::fmt::Display for ParseSeptendigyteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid septendigyte: {}", self.0)
    }
}

fn parse_septendigyte(s: &str) -> Result<Septendigyte, ParseSeptendigyteError> {
    let s = s.trim();

    if let Some(digits) = s.strip_prefix("0s").or_else(|| s.strip_prefix("0S")) {
        let mut chars = digits.chars();
        let (Some(hi), Some(lo), None) = (chars.next(), chars.next(), chars.next()) else {
            return Err(ParseSeptendigyteError(format!(
                "'{s}' must have exactly 2 septendigits after 0s"
            )));
        };
        let parse_septendigit = |c: char| match c {
            '0'..='9' => Ok(c as u8 - b'0'),
            'A' | 'a' => Ok(10),
            'B' | 'b' => Ok(11),
            'C' | 'c' => Ok(12),
            'D' | 'd' => Ok(13),
            'E' | 'e' => Ok(14),
            'F' | 'f' => Ok(15),
            'G' | 'g' => Ok(16),
            _ => Err(ParseSeptendigyteError(format!(
                "invalid septendigit '{c}' in '{s}'"
            ))),
        };
        let hi = parse_septendigit(hi)?;
        let lo = parse_septendigit(lo)?;
        return Ok(Septendigyte::from_digits([hi, lo]));
    }

    let n: u16 = s
        .parse()
        .map_err(|_| ParseSeptendigyteError(format!("'{s}' is not a valid number")))?;
    if n > 288 {
        return Err(ParseSeptendigyteError(format!("{n} out of range 0..=288")));
    }
    Ok(Septendigyte::new(n))
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Reg {
    R(usize), // 0s0 - 0sE
    SP,       // 0sF
    BP,       // 0sG
}

impl Reg {
    const ALL: [Reg; 17] = [
        Reg::R(00),
        Reg::R(01),
        Reg::R(02),
        Reg::R(03),
        Reg::R(04),
        Reg::R(05),
        Reg::R(06),
        Reg::R(07),
        Reg::R(08),
        Reg::R(09),
        Reg::R(10),
        Reg::R(11),
        Reg::R(12),
        Reg::R(13),
        Reg::R(14),
        Reg::SP,
        Reg::BP,
    ];

    fn decode(v: u8) -> Reg {
        match v {
            0..=14 => Reg::R(v as usize),
            15 => Reg::SP,
            16 => Reg::BP,
            _ => unreachable!(),
        }
    }
}

impl std::fmt::Debug for Reg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Reg::R(v) => write!(
                f,
                "r{}",
                [
                    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e'
                ][*v]
            ),
            Reg::SP => write!(f, "sp"),
            Reg::BP => write!(f, "bp"),
        }
    }
}

struct Regs {
    r: [Septendigyte; 15],
    sp: Septendigyte,
    bp: Septendigyte,
}

impl Default for Regs {
    fn default() -> Self {
        Regs {
            r: [Septendigyte::MIN; 15],
            sp: Septendigyte::MAX, // stack is at the very end
            bp: Septendigyte::MIN,
        }
    }
}

impl std::ops::Index<Reg> for Regs {
    type Output = Septendigyte;
    fn index(&self, reg: Reg) -> &Self::Output {
        match reg {
            Reg::R(v) => &self.r[v],
            Reg::SP => &self.sp,
            Reg::BP => &self.bp,
        }
    }
}

impl std::ops::IndexMut<Reg> for Regs {
    fn index_mut(&mut self, reg: Reg) -> &mut Self::Output {
        match reg {
            Reg::R(v) => &mut self.r[v],
            Reg::SP => &mut self.sp,
            Reg::BP => &mut self.bp,
        }
    }
}

impl std::fmt::Debug for Regs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            Reg::ALL
                .iter()
                .map(|&v| format!(
                    "{v:?}: {} {:?}",
                    DebugSeptendigyte(self[v]),
                    DebugSeptendigyte(self[v])
                ))
                .fold(String::new(), |acc, s| {
                    if acc.is_empty() {
                        format!("{s}")
                    } else {
                        format!("{acc}\n{s}")
                    }
                })
        )
    }
}

enum Instruction {
    NOP,                                         // 0s00
    HLT,                                         // 0s01
    MOV { dst: Reg, src: Reg },                  // 0s10
    IMOV { dst: Reg, imm: Septendigyte },        // 0s11
    LDR { dst: Reg, addr: Septendigyte },        // 0s12
    LDRREG { dst: Reg, reg: Reg },               // 0s13
    STR { addr: Septendigyte, src: Reg },        // 0s14
    STRREG { reg: Reg, src: Reg },               // 0s15
    PUSH { src: Reg },                           // 0s16
    IPUSH { src: Septendigyte },                 // 0s17
    POP { dst: Reg },                            // 0s18
    ADD { dst: Reg, src: Reg },                  // 0s20
    SUB { dst: Reg, src: Reg },                  // 0s21
    MUL { dst: Reg, src: Reg },                  // 0s22
    DIV { dst: Reg, src: Reg },                  // 0s23
    MOD { dst: Reg, src: Reg },                  // 0s24
    IADD { dst: Reg, imm: Septendigyte },        // 0s25
    ISUB { dst: Reg, imm: Septendigyte },        // 0s26
    IMUL { dst: Reg, imm: Septendigyte },        // 0s27
    IDIV { dst: Reg, imm: Septendigyte },        // 0s28
    IMOD { dst: Reg, imm: Septendigyte },        // 0s29
    NEG { dst: Reg },                            // 0s2A
    DNOT { dst: Reg },                           // 0s2B
    DMIN { dst: Reg, src: Reg },                 // 0s2C
    DMAX { dst: Reg, src: Reg },                 // 0s2D
    IDMIN { dst: Reg, imm: Septendigyte },       // 0s2E
    IDMAX { dst: Reg, imm: Septendigyte },       // 0s2F
    JIZ { reg: Reg, then: Septendigyte },        // 0s30
    JNZ { reg: Reg, then: Septendigyte },        // 0s31
    JEQ { a: Reg, b: Reg, then: Septendigyte },  // 0s32
    JNE { a: Reg, b: Reg, then: Septendigyte },  // 0s33
    JLT { a: Reg, b: Reg, then: Septendigyte },  // 0s34
    JGT { a: Reg, b: Reg, then: Septendigyte },  // 0s35
    JLTI { a: Reg, b: Reg, then: Septendigyte }, // 0s36
    JGTI { a: Reg, b: Reg, then: Septendigyte }, // 0s37
    JMP { addr: Septendigyte },                  // 0s40
    CALL { addr: Septendigyte },                 // 0s41
    RET,                                         // 0s42
    OUT17 { reg: Reg },                          // 0s50
    OUT10 { reg: Reg },                          // 0s51
    OUTC { reg: Reg },                           // 0s52
    IOUTC { char: Septendigyte },                // 0s53
    OUTS { ptr: Reg, len: Reg },                 // 0s54
    IOUTS { ptr: Reg, len: Septendigyte },       // 0s55
}

impl Instruction {
    fn decode(mem: &[Septendigyte], ip: &mut usize) -> Result<Self, String> {
        let mut next = || {
            let u = mem[*ip];
            *ip = (*ip + 1) % 289;
            u
        };

        let op = next();
        match op.get() {
            0 => Ok(Self::NOP),
            1 => Ok(Self::HLT),
            17 => Ok({
                let [dst, src] = next().digits().map(|v| Reg::decode(v));
                Self::MOV { dst, src }
            }),
            18 => Ok(Self::IMOV {
                dst: Reg::decode(next().digits()[1]),
                imm: next(),
            }),
            19 => Ok(Self::LDR {
                dst: Reg::decode(next().digits()[1]),
                addr: next(),
            }),
            20 => Ok({
                let [dst, reg] = next().digits().map(|v| Reg::decode(v));
                Self::LDRREG { dst, reg }
            }),
            21 => Ok(Self::STR {
                addr: next(),
                src: Reg::decode(next().digits()[1]),
            }),
            22 => Ok({
                let [reg, src] = next().digits().map(|v| Reg::decode(v));
                Self::STRREG { reg, src }
            }),
            23 => Ok(Self::PUSH {
                src: Reg::decode(next().digits()[1]),
            }),
            24 => Ok(Self::IPUSH { src: next() }),
            25 => Ok(Self::POP {
                dst: Reg::decode(next().digits()[1]),
            }),
            34 => Ok({
                let [dst, src] = next().digits().map(|v| Reg::decode(v));
                Self::ADD { dst, src }
            }),
            35 => Ok({
                let [dst, src] = next().digits().map(|v| Reg::decode(v));
                Self::SUB { dst, src }
            }),
            36 => Ok({
                let [dst, src] = next().digits().map(|v| Reg::decode(v));
                Self::MUL { dst, src }
            }),
            37 => Ok({
                let [dst, src] = next().digits().map(|v| Reg::decode(v));
                Self::DIV { dst, src }
            }),
            38 => Ok({
                let [dst, src] = next().digits().map(|v| Reg::decode(v));
                Self::MOD { dst, src }
            }),
            39 => Ok(Self::IADD {
                dst: Reg::decode(next().digits()[1]),
                imm: next(),
            }),
            40 => Ok(Self::ISUB {
                dst: Reg::decode(next().digits()[1]),
                imm: next(),
            }),
            41 => Ok(Self::IMUL {
                dst: Reg::decode(next().digits()[1]),
                imm: next(),
            }),
            42 => Ok(Self::IDIV {
                dst: Reg::decode(next().digits()[1]),
                imm: next(),
            }),
            43 => Ok(Self::IMOD {
                dst: Reg::decode(next().digits()[1]),
                imm: next(),
            }),
            44 => Ok(Self::NEG {
                dst: Reg::decode(next().digits()[1]),
            }),
            45 => Ok(Self::DNOT {
                dst: Reg::decode(next().digits()[1]),
            }),
            46 => Ok({
                let [dst, src] = next().digits().map(|v| Reg::decode(v));
                Self::DMIN { dst, src }
            }),
            47 => Ok({
                let [dst, src] = next().digits().map(|v| Reg::decode(v));
                Self::DMAX { dst, src }
            }),
            48 => Ok(Self::IDMIN {
                dst: Reg::decode(next().digits()[1]),
                imm: next(),
            }),
            49 => Ok(Self::IDMAX {
                dst: Reg::decode(next().digits()[1]),
                imm: next(),
            }),
            51 => Ok(Self::JIZ {
                reg: Reg::decode(next().digits()[1]),
                then: next(),
            }),
            52 => Ok(Self::JNZ {
                reg: Reg::decode(next().digits()[1]),
                then: next(),
            }),
            53 => Ok({
                let [a, b] = next().digits().map(|v| Reg::decode(v));
                Self::JEQ { a, b, then: next() }
            }),
            54 => Ok({
                let [a, b] = next().digits().map(|v| Reg::decode(v));
                Self::JNE { a, b, then: next() }
            }),
            55 => Ok({
                let [a, b] = next().digits().map(|v| Reg::decode(v));
                Self::JLT { a, b, then: next() }
            }),
            56 => Ok({
                let [a, b] = next().digits().map(|v| Reg::decode(v));
                Self::JGT { a, b, then: next() }
            }),
            57 => Ok({
                let [a, b] = next().digits().map(|v| Reg::decode(v));
                Self::JLTI { a, b, then: next() }
            }),
            58 => Ok({
                let [a, b] = next().digits().map(|v| Reg::decode(v));
                Self::JGTI { a, b, then: next() }
            }),
            68 => Ok(Self::JMP { addr: next() }),
            69 => Ok(Self::CALL { addr: next() }),
            70 => Ok(Self::RET),
            85 => Ok(Self::OUT17 {
                reg: Reg::decode(next().digits()[1]),
            }),
            86 => Ok(Self::OUT10 {
                reg: Reg::decode(next().digits()[1]),
            }),
            87 => Ok(Self::OUTC {
                reg: Reg::decode(next().digits()[1]),
            }),
            88 => Ok(Self::IOUTC { char: next() }),
            89 => Ok({
                let [ptr, len] = next().digits().map(|v| Reg::decode(v));
                Self::OUTS { ptr, len }
            }),
            90 => Ok(Self::IOUTS {
                ptr: Reg::decode(next().digits()[1]),
                len: next(),
            }),
            _ => Err(format!("Invalid opcode: {:?}", DebugSeptendigyte(op))),
        }
    }

    fn name(&self) -> &str {
        use Instruction::*;
        match self {
            NOP => "nop",
            HLT => "hlt",
            MOV { .. } | IMOV { .. } => "mov",
            LDR { .. } | LDRREG { .. } => "ldr",
            STR { .. } | STRREG { .. } => "str",
            PUSH { .. } | IPUSH { .. } => "push",
            POP { .. } => "pop",
            ADD { .. } | IADD { .. } => "add",
            SUB { .. } | ISUB { .. } => "sub",
            MUL { .. } | IMUL { .. } => "mul",
            DIV { .. } | IDIV { .. } => "div",
            MOD { .. } | IMOD { .. } => "mod",
            NEG { .. } => "neg",
            DNOT { .. } => "dnot",
            DMIN { .. } | IDMIN { .. } => "dmin",
            DMAX { .. } | IDMAX { .. } => "dmax",
            JIZ { .. } => "jiz",
            JNZ { .. } => "jnz",
            JEQ { .. } => "jeq",
            JNE { .. } => "jne",
            JLT { .. } => "jlt",
            JGT { .. } => "jgt",
            JLTI { .. } => "jlti",
            JGTI { .. } => "jgti",
            JMP { .. } => "jmp",
            CALL { .. } => "call",
            RET => "ret",
            OUT17 { .. } => "out17",
            OUT10 { .. } => "out10",
            OUTC { .. } | IOUTC { .. } => "outc",
            OUTS { .. } | IOUTS { .. } => "outs",
        }
    }

    fn operands(&self) -> Vec<Operand> {
        use Operand::*;
        match self {
            Self::NOP => vec![],
            Self::HLT => vec![],
            Self::MOV { dst, src } => vec![R(*dst), R(*src)],
            Self::IMOV { dst, imm } => vec![R(*dst), I(*imm)],
            Self::LDR { dst, addr } => vec![R(*dst), I(*addr)],
            Self::LDRREG { dst, reg } => vec![R(*dst), R(*reg)],
            Self::STR { addr, src } => vec![I(*addr), R(*src)],
            Self::STRREG { reg, src } => vec![R(*reg), R(*src)],
            Self::PUSH { src } => vec![R(*src)],
            Self::IPUSH { src } => vec![I(*src)],
            Self::POP { dst } => vec![R(*dst)],
            Self::ADD { dst, src } => vec![R(*dst), R(*src)],
            Self::SUB { dst, src } => vec![R(*dst), R(*src)],
            Self::MUL { dst, src } => vec![R(*dst), R(*src)],
            Self::DIV { dst, src } => vec![R(*dst), R(*src)],
            Self::MOD { dst, src } => vec![R(*dst), R(*src)],
            Self::IADD { dst, imm } => vec![R(*dst), I(*imm)],
            Self::ISUB { dst, imm } => vec![R(*dst), I(*imm)],
            Self::IMUL { dst, imm } => vec![R(*dst), I(*imm)],
            Self::IDIV { dst, imm } => vec![R(*dst), I(*imm)],
            Self::IMOD { dst, imm } => vec![R(*dst), I(*imm)],
            Self::NEG { dst } => vec![R(*dst)],
            Self::DNOT { dst } => vec![R(*dst)],
            Self::DMIN { dst, src } => vec![R(*dst), R(*src)],
            Self::DMAX { dst, src } => vec![R(*dst), R(*src)],
            Self::IDMIN { dst, imm } => vec![R(*dst), I(*imm)],
            Self::IDMAX { dst, imm } => vec![R(*dst), I(*imm)],
            Self::JIZ { reg, then } => vec![R(*reg), I(*then)],
            Self::JNZ { reg, then } => vec![R(*reg), I(*then)],
            Self::JEQ { a, b, then } => vec![R(*a), R(*b), I(*then)],
            Self::JNE { a, b, then } => vec![R(*a), R(*b), I(*then)],
            Self::JLT { a, b, then } => vec![R(*a), R(*b), I(*then)],
            Self::JGT { a, b, then } => vec![R(*a), R(*b), I(*then)],
            Self::JLTI { a, b, then } => vec![R(*a), R(*b), I(*then)],
            Self::JGTI { a, b, then } => vec![R(*a), R(*b), I(*then)],
            Self::JMP { addr } => vec![I(*addr)],
            Self::CALL { addr } => vec![I(*addr)],
            Self::RET => vec![],
            Self::OUT17 { reg } => vec![R(*reg)],
            Self::OUT10 { reg } => vec![R(*reg)],
            Self::OUTC { reg } => vec![R(*reg)],
            Self::IOUTC { char } => vec![I(*char)],
            Self::OUTS { ptr, len } => vec![R(*ptr), R(*len)],
            Self::IOUTS { ptr, len } => vec![R(*ptr), I(*len)],
        }
    }
}

enum Operand {
    R(Reg),
    I(Septendigyte),
}

impl std::fmt::Debug for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let operands = self.operands();
        if operands.is_empty() {
            write!(f, "{}", self.name())
        } else {
            write!(
                f,
                "{} {}",
                self.name(),
                operands
                    .into_iter()
                    .map(|o| {
                        match o {
                            Operand::R(r) => format!("{r:?}"),
                            Operand::I(i) => {
                                format!("{} ({:?})", DebugSeptendigyte(i), DebugSeptendigyte(i))
                            }
                        }
                    })
                    .fold(String::new(), |acc, s| {
                        if acc.is_empty() {
                            format!("{s}")
                        } else {
                            format!("{acc}, {s}")
                        }
                    })
            )
        }
    }
}

struct N2M289b17 {
    mem: Vec<Septendigyte>,
    regs: Regs,
}

impl N2M289b17 {
    fn fail(&self, ip: usize, msg: &str) -> ! {
        eprintln!("IP {}: {msg}", self.format_ip(ip));
        std::process::exit(1);
    }
}

impl Machine for N2M289b17 {
    type Inst = Instruction;

    fn decode(&self, ip: &mut usize) -> Result<Self::Inst, String> {
        Instruction::decode(&self.mem, ip)
    }

    fn execute(&mut self, inst: Self::Inst, ip: &mut usize) -> bool {
        let r = &mut self.regs;
        let m = &mut self.mem;
        match inst {
            Instruction::NOP => {}
            Instruction::HLT => return true,
            Instruction::MOV { dst, src } => r[dst] = r[src],
            Instruction::IMOV { dst, imm } => r[dst] = imm,
            Instruction::LDR { dst, addr } => r[dst] = m[addr.get() as usize],
            Instruction::LDRREG { dst, reg } => r[dst] = m[r[reg].get() as usize],
            Instruction::STR { addr, src } => m[addr.get() as usize] = r[src],
            Instruction::STRREG { reg, src } => m[r[reg].get() as usize] = r[src],
            Instruction::PUSH { src } => {
                m[r.sp.get() as usize] = r[src];
                r.sp = r.sp.wrapping_sub(Septendigyte::new(1));
            }
            Instruction::IPUSH { src } => {
                m[r.sp.get() as usize] = src;
                r.sp = r.sp.wrapping_sub(Septendigyte::new(1));
            }
            Instruction::POP { dst } => {
                if r.sp == Septendigyte::MAX {
                    self.fail(*ip, "Stack underflow");
                }
                r.sp = r.sp.wrapping_add(Septendigyte::new(1));
                r[dst] = m[r.sp.get() as usize];
            }
            Instruction::ADD { dst, src } => r[dst] = r[dst].wrapping_add(r[src]),
            Instruction::SUB { dst, src } => r[dst] = r[dst].wrapping_sub(r[src]),
            Instruction::MUL { dst, src } => r[dst] = r[dst].wrapping_mul(r[src]),
            Instruction::DIV { dst, src } => {
                r[dst] = match r[dst].checked_div(r[src]) {
                    Some(v) => v,
                    None => self.fail(*ip, "Division by zero"),
                }
            }
            Instruction::MOD { dst, src } => {
                r[dst] = match r[dst].checked_mod(r[src]) {
                    Some(v) => v,
                    None => self.fail(*ip, "Division by zero"),
                }
            }
            Instruction::IADD { dst, imm } => r[dst] = r[dst].wrapping_add(imm),
            Instruction::ISUB { dst, imm } => r[dst] = r[dst].wrapping_sub(imm),
            Instruction::IMUL { dst, imm } => r[dst] = r[dst].wrapping_mul(imm),
            Instruction::IDIV { dst, imm } => {
                r[dst] = match r[dst].checked_div(imm) {
                    Some(v) => v,
                    None => self.fail(*ip, "Division by zero"),
                }
            }
            Instruction::IMOD { dst, imm } => {
                r[dst] = match r[dst].checked_mod(imm) {
                    Some(v) => v,
                    None => self.fail(*ip, "Division by zero"),
                }
            }
            Instruction::NEG { dst } => r[dst] = r[dst].wrapping_neg(),
            Instruction::DNOT { dst } => r[dst] = r[dst].dnot(),
            Instruction::DMIN { dst, src } => r[dst] = r[dst].dmin(r[src]),
            Instruction::DMAX { dst, src } => r[dst] = r[dst].dmax(r[src]),
            Instruction::IDMIN { dst, imm } => r[dst] = r[dst].dmin(imm),
            Instruction::IDMAX { dst, imm } => r[dst] = r[dst].dmax(imm),
            Instruction::JIZ { reg, then } => {
                if r[reg] == Septendigyte::MIN {
                    *ip = then.get() as usize;
                }
            }
            Instruction::JNZ { reg, then } => {
                if r[reg] != Septendigyte::MIN {
                    *ip = then.get() as usize;
                }
            }
            Instruction::JEQ { a, b, then } => {
                if r[a] == r[b] {
                    *ip = then.get() as usize;
                }
            }
            Instruction::JNE { a, b, then } => {
                if r[a] != r[b] {
                    *ip = then.get() as usize;
                }
            }
            Instruction::JLT { a, b, then } => {
                if r[a] < r[b] {
                    *ip = then.get() as usize;
                }
            }
            Instruction::JGT { a, b, then } => {
                if r[a] > r[b] {
                    *ip = then.get() as usize;
                }
            }
            Instruction::JLTI { a, b, then } => {
                if r[a].as_signed() < r[b].as_signed() {
                    *ip = then.get() as usize;
                }
            }
            Instruction::JGTI { a, b, then } => {
                if r[a].as_signed() > r[b].as_signed() {
                    *ip = then.get() as usize;
                }
            }
            Instruction::JMP { addr } => *ip = addr.get() as usize,
            Instruction::CALL { addr } => {
                m[r.sp.get() as usize] = addr;
                r.sp = r.sp.wrapping_sub(Septendigyte::new(1));
                *ip = addr.get() as usize;
            }
            Instruction::RET => {
                if r.sp == Septendigyte::MAX {
                    self.fail(*ip, "Stack underflow");
                }
                r.sp = r.sp.wrapping_add(Septendigyte::new(1));
                *ip = m[r.sp.get() as usize].get() as usize;
            }
            Instruction::OUT17 { reg } => print!("{:?}", DebugSeptendigyte(r[reg])),
            Instruction::OUT10 { reg } => print!("{}", DebugSeptendigyte(r[reg])),
            Instruction::OUTC { reg } => print!("{}", septendigyte2char(r[reg])),
            Instruction::IOUTC { char } => print!("{}", septendigyte2char(char)),
            Instruction::OUTS { ptr, len } => {
                let ptr = r[ptr].get() as usize;
                let len = r[len].get() as usize;
                for i in 0..len {
                    print!("{}", septendigyte2char(m[(ptr + i) % 289]));
                }
            }
            Instruction::IOUTS { ptr, len } => {
                let ptr = r[ptr].get() as usize;
                let len = len.get() as usize;
                for i in 0..len {
                    print!("{}", septendigyte2char(m[(ptr + i) % 289]));
                }
            }
        }
        false
    }

    fn debug_dump(&self, _old_ip: usize) -> String {
        format!("{:?}", self.regs)
    }

    fn format_ip(&self, ip: usize) -> String {
        format!("{:?}", DebugSeptendigyte(Septendigyte::new(ip as u16)))
    }
}

fn parse_program(source: &str) -> Result<Vec<Septendigyte>, String> {
    let mut mem = Vec::new();
    for (line_no, raw_line) in source.lines().enumerate() {
        let line = raw_line.split(';').next().unwrap_or("").trim();
        for tok in line.split_whitespace() {
            let n = parse_septendigyte(tok).map_err(|e| format!("line {}: {e}", line_no + 1))?;
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
    mem.resize(121, Septendigyte::new(0));

    let machine = N2M289b17 {
        mem,
        regs: Regs::default(),
    };

    machine.run(debug);
}
