use std::fmt;

#[derive(PartialEq, Eq)]
pub enum Opcode {
    MovReg,
    AddReg,
    Sub,
    CmpReg,
    ShlReg,
    ShrReg,
    Jmp,
    SarReg,
    Mul,
    Div,
    MulU,
    DivU,
    Or,
    And,
    Xor,
    Not,
    MovImm,
    AddImm5,
    Setf,
    CmpImm,
    ShlImm,
    ShrImm,
    Cli,
    SarImm,
    Reti,
    Ldsr,
    Stsr,
    Sei,
    Bv,
    Bc,
    Bz,
    Bnh,
    Bn,
    Br,
    Blt,
    Ble,
    Bnv,
    Bnc,
    Bnz,
    Bh,
    Bp,
    Nop,
    Bge,
    Bgt,
    Movea,
    AddImm16,
    Jr,
    Jal,
    OrI,
    AndI,
    XorI,
    Movhi,
    Ldb,
    Ldh,
    Ldw,
    Stb,
    Sth,
    Stw,
    Inb,
    Inh,
    Inw,
    Outb,
    Outh,
    Extended,
    Outw,
}

impl Opcode {
    pub fn from_halfword(halfword: u16) -> Opcode {
        if halfword >> 13 == 0b100 {
            let cond_bits = (halfword >> 9) & 0x0f;
            match cond_bits {
                0b0000 => Opcode::Bv,
                0b0001 => Opcode::Bc,
                0b0010 => Opcode::Bz,
                0b0011 => Opcode::Bnh,
                0b0100 => Opcode::Bn,
                0b0101 => Opcode::Br,
                0b0110 => Opcode::Blt,
                0b0111 => Opcode::Ble,
                0b1000 => Opcode::Bnv,
                0b1001 => Opcode::Bnc,
                0b1010 => Opcode::Bnz,
                0b1011 => Opcode::Bh,
                0b1100 => Opcode::Bp,
                0b1101 => Opcode::Nop,
                0b1110 => Opcode::Bge,
                0b1111 => Opcode::Bgt,
                _ => panic!("Unrecognized cond bits: {:04b} (halfword: 0b{:016b})", cond_bits, halfword)
            }
        } else {
            let opcode_bits = halfword >> 10;
            match opcode_bits {
                0b000000 => Opcode::MovReg,
                0b000001 => Opcode::AddReg,
                0b000010 => Opcode::Sub,
                0b000011 => Opcode::CmpReg,
                0b000100 => Opcode::ShlReg,
                0b000101 => Opcode::ShrReg,
                0b000110 => Opcode::Jmp,
                0b000111 => Opcode::SarReg,
                0b001000 => Opcode::Mul,
                0b001001 => Opcode::Div,
                0b001010 => Opcode::MulU,
                0b001011 => Opcode::DivU,
                0b001100 => Opcode::Or,
                0b001101 => Opcode::And,
                0b001110 => Opcode::Xor,
                0b001111 => Opcode::Not,
                0b010000 => Opcode::MovImm,
                0b010001 => Opcode::AddImm5,
                0b010010 => Opcode::Setf,
                0b010011 => Opcode::CmpImm,
                0b010100 => Opcode::ShlImm,
                0b010101 => Opcode::ShrImm,
                0b010110 => Opcode::Cli,
                0b010111 => Opcode::SarImm,
                0b011001 => Opcode::Reti,
                0b011100 => Opcode::Ldsr,
                0b011101 => Opcode::Stsr,
                0b011110 => Opcode::Sei,
                0b101000 => Opcode::Movea,
                0b101001 => Opcode::AddImm16,
                0b101010 => Opcode::Jr,
                0b101011 => Opcode::Jal,
                0b101100 => Opcode::OrI,
                0b101101 => Opcode::AndI,
                0b101110 => Opcode::XorI,
                0b101111 => Opcode::Movhi,
                0b110000 => Opcode::Ldb,
                0b110001 => Opcode::Ldh,
                0b110011 => Opcode::Ldw,
                0b110100 => Opcode::Stb,
                0b110101 => Opcode::Sth,
                0b110111 => Opcode::Stw,
                0b111000 => Opcode::Inb,
                0b111001 => Opcode::Inh,
                0b111011 => Opcode::Inw,
                0b111100 => Opcode::Outb,
                0b111101 => Opcode::Outh,
                0b111110 => Opcode::Extended,
                0b111111 => Opcode::Outw,
                _ => panic!("Unrecognized opcode bits: {:06b} (halfword: 0b{:016b})", opcode_bits, halfword),
            }
        }
    }

    pub fn instruction_format(&self) -> InstructionFormat {
        match self {
            &Opcode::MovReg => InstructionFormat::I,
            &Opcode::AddReg => InstructionFormat::I,
            &Opcode::Sub => InstructionFormat::I,
            &Opcode::CmpReg => InstructionFormat::I,
            &Opcode::ShlReg => InstructionFormat::I,
            &Opcode::ShrReg => InstructionFormat::I,
            &Opcode::Jmp => InstructionFormat::I,
            &Opcode::SarReg => InstructionFormat::I,
            &Opcode::Mul => InstructionFormat::I,
            &Opcode::Div => InstructionFormat::I,
            &Opcode::MulU => InstructionFormat::I,
            &Opcode::DivU => InstructionFormat::I,
            &Opcode::Or => InstructionFormat::I,
            &Opcode::And => InstructionFormat::I,
            &Opcode::Xor => InstructionFormat::I,
            &Opcode::Not => InstructionFormat::I,
            &Opcode::MovImm => InstructionFormat::II,
            &Opcode::AddImm5 => InstructionFormat::II,
            &Opcode::Setf => InstructionFormat::II,
            &Opcode::CmpImm => InstructionFormat::II,
            &Opcode::ShlImm => InstructionFormat::II,
            &Opcode::ShrImm => InstructionFormat::II,
            &Opcode::Cli => InstructionFormat::II,
            &Opcode::SarImm => InstructionFormat::II,
            &Opcode::Reti => InstructionFormat::II,
            &Opcode::Ldsr => InstructionFormat::II,
            &Opcode::Stsr => InstructionFormat::II,
            &Opcode::Sei => InstructionFormat::II,
            &Opcode::Bv => InstructionFormat::III,
            &Opcode::Bc => InstructionFormat::III,
            &Opcode::Bz => InstructionFormat::III,
            &Opcode::Bnh => InstructionFormat::III,
            &Opcode::Bn => InstructionFormat::III,
            &Opcode::Br => InstructionFormat::III,
            &Opcode::Blt => InstructionFormat::III,
            &Opcode::Ble => InstructionFormat::III,
            &Opcode::Bnv => InstructionFormat::III,
            &Opcode::Bnc => InstructionFormat::III,
            &Opcode::Bnz => InstructionFormat::III,
            &Opcode::Bh => InstructionFormat::III,
            &Opcode::Bp => InstructionFormat::III,
            &Opcode::Nop => InstructionFormat::III,
            &Opcode::Bge => InstructionFormat::III,
            &Opcode::Bgt => InstructionFormat::III,
            &Opcode::Movea => InstructionFormat::V,
            &Opcode::AddImm16 => InstructionFormat::V,
            &Opcode::Jr => InstructionFormat::IV,
            &Opcode::OrI => InstructionFormat::V,
            &Opcode::AndI => InstructionFormat::V,
            &Opcode::XorI => InstructionFormat::V,
            &Opcode::Jal => InstructionFormat::IV,
            &Opcode::Movhi => InstructionFormat::V,
            &Opcode::Ldb => InstructionFormat::VI,
            &Opcode::Ldh => InstructionFormat::VI,
            &Opcode::Ldw => InstructionFormat::VI,
            &Opcode::Stb => InstructionFormat::VI,
            &Opcode::Sth => InstructionFormat::VI,
            &Opcode::Stw => InstructionFormat::VI,
            &Opcode::Inb => InstructionFormat::VI,
            &Opcode::Inh => InstructionFormat::VI,
            &Opcode::Inw => InstructionFormat::VI,
            &Opcode::Outb => InstructionFormat::VI,
            &Opcode::Outh => InstructionFormat::VI,
            &Opcode::Extended => InstructionFormat::VII,
            &Opcode::Outw => InstructionFormat::VI,
        }
    }

    pub fn subop(&self, subop: usize) -> SubOp {
        match subop {
            0b000000 => SubOp::CmpfS,
            0b000010 => SubOp::CvtWs,
            0b000011 => SubOp::CvtSw,
            0b000100 => SubOp::AddfS,
            0b000101 => SubOp::SubfS,
            0b000110 => SubOp::MulfS,
            0b000111 => SubOp::DivfS,
            0b001000 => SubOp::Xb,
            0b001001 => SubOp::Xh,
            0b001011 => SubOp::TrncSw,
            0b001100 => SubOp::Mpyhw,
            _ => panic!("Unrecognized subop bits: {:06b}", subop),
        }
    }

    pub fn system_register(&self, imm5: usize) -> SystemRegister {
        match imm5 {
            0 => SystemRegister::Eipc,
            1 => SystemRegister::Eipsw,
            2 => SystemRegister::Fepc,
            3 => SystemRegister::Fepsw,
            4 => SystemRegister::Ecr,
            5 => SystemRegister::Psw,
            24 => SystemRegister::Chcw,
            _ => panic!("Unrecognized system register: {}", imm5),
        }
    }

    pub fn condition(&self, imm5: usize) -> Condition {
        match imm5 {
            0x00 => Condition::V,
            0x01 => Condition::C,
            0x02 => Condition::Z,
            0x03 => Condition::Nh,
            0x04 => Condition::N,
            0x05 => Condition::T,
            0x06 => Condition::Lt,
            0x07 => Condition::Le,
            0x08 => Condition::Nv,
            0x09 => Condition::Nc,
            0x0a => Condition::Nz,
            0x0b => Condition::H,
            0x0c => Condition::P,
            0x0d => Condition::F,
            0x0e => Condition::Ge,
            0x0f => Condition::Gt,
            _ => panic!("Unrecognized condition: {}", imm5),
        }
    }
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mnemonic = match self {
            &Opcode::MovReg | &Opcode::MovImm => "mov",
            &Opcode::AddReg | &Opcode::AddImm5 => "add",
            &Opcode::Sub => "sub",
            &Opcode::CmpReg | &Opcode::CmpImm => "cmp",
            &Opcode::ShlReg | &Opcode::ShlImm => "shl",
            &Opcode::ShrReg | &Opcode::ShrImm => "shr",
            &Opcode::Jmp => "jmp",
            &Opcode::SarReg | &Opcode::SarImm => "sar",
            &Opcode::Mul => "mul",
            &Opcode::Div => "div",
            &Opcode::MulU => "mulu",
            &Opcode::DivU => "divu",
            &Opcode::Or => "or",
            &Opcode::And => "and",
            &Opcode::Xor => "xor",
            &Opcode::Not => "not",
            &Opcode::Setf => "setf",
            &Opcode::Cli => "cli",
            &Opcode::Reti => "reti",
            &Opcode::Ldsr => "ldsr",
            &Opcode::Stsr => "stsr",
            &Opcode::Sei => "sei",
            &Opcode::Bv => "bv",
            &Opcode::Bc => "bc",
            &Opcode::Bz => "bz",
            &Opcode::Bnh => "bnh",
            &Opcode::Bn => "bn",
            &Opcode::Br => "br",
            &Opcode::Blt => "blt",
            &Opcode::Ble => "ble",
            &Opcode::Bnv => "bnv",
            &Opcode::Bnc => "bnc",
            &Opcode::Bnz => "bnz",
            &Opcode::Bh => "bh",
            &Opcode::Bp => "bp",
            &Opcode::Nop => "nop",
            &Opcode::Bge => "bge",
            &Opcode::Bgt => "bgt",
            &Opcode::Movea => "movea",
            &Opcode::AddImm16 => "addi",
            &Opcode::Jr => "jr",
            &Opcode::Jal => "jal",
            &Opcode::OrI => "ori",
            &Opcode::AndI => "andi",
            &Opcode::XorI => "xori",
            &Opcode::Movhi => "movhi",
            &Opcode::Ldb => "ld.b",
            &Opcode::Ldh => "ld.h",
            &Opcode::Ldw => "ld.w",
            &Opcode::Stb => "st.b",
            &Opcode::Sth => "st.h",
            &Opcode::Stw => "st.w",
            &Opcode::Inb => "in.b",
            &Opcode::Inh => "in.h",
            &Opcode::Inw => "in.w",
            &Opcode::Outb => "out.b",
            &Opcode::Outh => "out.h",
            &Opcode::Extended => unreachable!(), // TODO: Better pattern
            &Opcode::Outw => "out.w",
        };
        write!(f, "{}", mnemonic)
    }
}

pub enum InstructionFormat {
    I,
    II,
    III,
    IV,
    V,
    VI,
    VII,
}

impl InstructionFormat {
    pub fn has_second_halfword(&self) -> bool {
        match self {
            &InstructionFormat::I => false,
            &InstructionFormat::II => false,
            &InstructionFormat::III => false,
            &InstructionFormat::IV => true,
            &InstructionFormat::V => true,
            &InstructionFormat::VI => true,
            &InstructionFormat::VII => true,
        }
    }
}

pub enum SubOp {
    CmpfS,
    CvtWs,
    CvtSw,
    AddfS,
    SubfS,
    MulfS,
    DivfS,
    Xb,
    Xh,
    TrncSw,
    Mpyhw,
}

impl fmt::Display for SubOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mnemonic = match self {
            &SubOp::CmpfS => "cmpf.s",
            &SubOp::CvtWs => "cvt.ws",
            &SubOp::CvtSw => "cvt.sw",
            &SubOp::AddfS => "addf.s",
            &SubOp::SubfS => "subf.s",
            &SubOp::MulfS => "mulf.s",
            &SubOp::DivfS => "divf.s",
            &SubOp::Xb => "xb",
            &SubOp::Xh => "xh",
            &SubOp::TrncSw => "trnc.sw",
            &SubOp::Mpyhw => "mpyhw",
        };
        write!(f, "{}", mnemonic)
    }
}

pub enum SystemRegister {
    Eipc,
    Eipsw,
    Fepc,
    Fepsw,
    Ecr,
    Psw,
    Chcw
}

impl fmt::Display for SystemRegister {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mnemonic = match self {
            &SystemRegister::Eipc => "eipc",
            &SystemRegister::Eipsw => "eipsw",
            &SystemRegister::Fepc => "fepc",
            &SystemRegister::Fepsw => "fepsw",
            &SystemRegister::Ecr => "ecr",
            &SystemRegister::Psw => "psw",
            &SystemRegister::Chcw => "chcw",
        };
        write!(f, "{}", mnemonic)
    }
}

pub enum Condition {
    V,
    C,
    Z,
    Nh,
    N,
    T,
    Lt,
    Le,
    Nv,
    Nc,
    Nz,
    H,
    P,
    F,
    Ge,
    Gt,
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mnemonic = match self {
            &Condition::V => "v",
            &Condition::C => "c",
            &Condition::Z => "z",
            &Condition::Nh => "nh",
            &Condition::N => "n",
            &Condition::T => "t",
            &Condition::Lt => "lt",
            &Condition::Le => "le",
            &Condition::Nv => "nv",
            &Condition::Nc => "nc",
            &Condition::Nz => "nz",
            &Condition::H => "h",
            &Condition::P => "p",
            &Condition::F => "f",
            &Condition::Ge => "ge",
            &Condition::Gt => "gt",
        };
        write!(f, "{}", mnemonic)
    }
}