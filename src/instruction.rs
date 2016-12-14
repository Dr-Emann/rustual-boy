use std::fmt;

#[derive(PartialEq, Eq)]
pub enum Opcode {
    MovReg,
    AddReg,
    Sub,
    CmpReg,
    Jmp,
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
    CmpImm,
    Cli,
    Ldsr,
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
                0b000110 => Opcode::Jmp,
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
                0b010011 => Opcode::CmpImm,
                0b010110 => Opcode::Cli,
                0b011100 => Opcode::Ldsr,
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
            &Opcode::Jmp => InstructionFormat::I,
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
            &Opcode::CmpImm => InstructionFormat::II,
            &Opcode::Cli => InstructionFormat::II,
            &Opcode::Ldsr => InstructionFormat::II,
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
            &Opcode::Outw => InstructionFormat::VI,
        }
    }

    pub fn system_register(&self, imm5: usize) -> SystemRegister {
        match imm5 {
            5 => SystemRegister::Psw,
            24 => SystemRegister::Chcw,
            _ => panic!("Unrecognized system register: {}", imm5),
        }
    }

    pub fn num_cycles(&self, branch_taken: bool) -> usize {
        match self {
            &Opcode::MovReg => 1,
            &Opcode::AddReg => 1,
            &Opcode::Sub => 1,
            &Opcode::CmpReg => 1,
            &Opcode::Jmp => 3,
            &Opcode::Mul => 13,
            &Opcode::Div => 38,
            &Opcode::MulU => 13,
            &Opcode::DivU => 36,
            &Opcode::Or => 1,
            &Opcode::And => 1,
            &Opcode::Xor => 1,
            &Opcode::Not => 1,
            &Opcode::MovImm => 1,
            &Opcode::AddImm5 => 1,
            &Opcode::CmpImm => 1,
            &Opcode::Cli => 1,
            &Opcode::Ldsr => 1,
            &Opcode::Sei => 1,
            &Opcode::Bv |
            &Opcode::Bc |
            &Opcode::Bz |
            &Opcode::Bnh |
            &Opcode::Bn |
            &Opcode::Blt |
            &Opcode::Ble |
            &Opcode::Bnv |
            &Opcode::Bnc |
            &Opcode::Bnz |
            &Opcode::Bh |
            &Opcode::Bp |
            &Opcode::Bge |
            &Opcode::Bgt => if branch_taken { 3 } else { 1 },
            &Opcode::Br => 3,
            &Opcode::Nop => 1,
            &Opcode::Movea => 1,
            &Opcode::AddImm16 => 1,
            &Opcode::Jr => 3,
            &Opcode::Jal => 3,
            &Opcode::OrI => 1,
            &Opcode::AndI => 1,
            &Opcode::XorI => 1,
            &Opcode::Movhi => 1,
            &Opcode::Ldb => 4,
            &Opcode::Ldh => 4,
            &Opcode::Ldw => 4,
            &Opcode::Stb => 4,
            &Opcode::Sth => 4,
            &Opcode::Stw => 4,
            &Opcode::Inb => 4,
            &Opcode::Inh => 4,
            &Opcode::Inw => 4,
            &Opcode::Outb => 4,
            &Opcode::Outh => 4,
            &Opcode::Outw => 4,
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
            &Opcode::Jmp => "jmp",
            &Opcode::Mul => "mul",
            &Opcode::Div => "div",
            &Opcode::MulU => "mulu",
            &Opcode::DivU => "divu",
            &Opcode::Or => "or",
            &Opcode::And => "and",
            &Opcode::Xor => "xor",
            &Opcode::Not => "not",
            &Opcode::Cli => "cli",
            &Opcode::Ldsr => "ldsr",
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
        }
    }
}

pub enum SystemRegister {
    Psw,
    Chcw
}

impl fmt::Display for SystemRegister {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mnemonic = match self {
            &SystemRegister::Psw => "psw",
            &SystemRegister::Chcw => "chcw",
        };
        write!(f, "{}", mnemonic)
    }
}