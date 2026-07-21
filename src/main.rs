use std::cmp::Ordering;

type Word = u16;
const NUM_REGISTER: usize = 8;
const MEMORY_SIZE: usize = 0x10000;

// instruction mask
const OPCODE_MASK: u16 = 0xF000;
const IMMEDIATE: u16 = 0b0000_000_000_1_00000;
const IMM5: u16 = 0b0000_000_000_0_11111;
const REG3: u16 = 0b0000_0000_0000_0111;
const PCOFFSET9: u16 = 0b0000_0001_1111_1111;

// Opcodes
const OP_ADD:  u16 = 0x1000;
const OP_LEA:  u16 = 0xE000;
const OP_TRAP: u16 = 0xF000;

struct CPU {
    r: [Word; NUM_REGISTER], 
    m: [Word; MEMORY_SIZE],  
    pc: Word,                
    psr: Word,               
}

impl CPU {
    fn new() -> Self {
        Self {
            r: [0; NUM_REGISTER],
            m: [0; MEMORY_SIZE], 
            pc: 0x3000,          
            psr: 0x0002,         
        }
    }

    // exec 1 cycle
    fn step(&mut self) -> bool {
        // 1. Fetch
        let instruction = self.m[self.pc as usize];
        self.pc = self.pc.wrapping_add(1);

        // 2. Decode & Execute
        let opcode = instruction & OPCODE_MASK;
        match opcode {
            OP_ADD => {
                if (instruction & IMMEDIATE) > 0 {
                    let imm5 = instruction & IMM5;
                    let sr1 = (instruction >> 6) & REG3;
                    let dr = (instruction >> 9) & REG3;
                    // 5 bit signed integer MSB index is 4.
                    self.r[dr as usize] = (self.r[sr1 as usize] as i16 + sext(imm5, 4)) as Word;
                    self.psr_cmp(self.r[dr as usize] as i16);
                } else {
                    let sr2 = instruction & REG3;
                    let sr1 = (instruction >> 6) & REG3;
                    let dr = (instruction >> 9) & REG3;
                    self.r[dr as usize] = self.r[sr1 as usize] + self.r[sr2 as usize];
                    self.psr_cmp(self.r[dr as usize] as i16);
                }
                true
            }
            OP_LEA => {
                let pcoffset9 = instruction & PCOFFSET9;
                let dr = (instruction >> 9) & REG3;
                let address = self.pc as i16 + sext(pcoffset9, 8);
                self.r[dr as usize] = self.m[address as usize];
                true
            }
            OP_TRAP => {
                let trap_vector = instruction & 0x00FF;
                self.trap(trap_vector)
            }
            _ => false,
        }
    }

    fn psr_cmp(&mut self, result: i16) {
        self.psr = self.psr & !REG3; // condtion code clear
        match result.cmp(&0) {
            Ordering::Less => self.psr = self.psr | 0b100,
            Ordering::Equal => self.psr = self.psr | 0b010,
            Ordering::Greater => self.psr = self.psr | 0b001,
        }
    }

    fn trap(&mut self, vector: Word) -> bool {
        match vector {
            0x21 => {
                // OUT
                let lowbyte = (self.r[0] & 0xFF) as u8;
                print!("{}", lowbyte as char);
                true
            }
            0x25 => {
                // HALT
                println!("\n--- HALT ---");
                false
            }
            _ => {
                println!("\nUnknown trap vector: {:#X}", vector);
                false
            }
        }
    }
}

fn sext(value: Word, msb_index: u32) -> i16 {
    if msb_index >= 15 {
        return value as i16;
    }

    let shift = 16 - 1 - msb_index;
    let extended = ((value as i16) << shift) >> shift;
    extended
}

fn main() {
    let mut cpu = CPU::new();

    // Load Program
    cpu.pc = 0x3000;
    cpu.m[0x3000] = 0b0001_000_000_1_01111; // ADD R0,R0 + 15
    cpu.m[0x3001] = 0b0001_000_000_000_000; // ADD R0,R0 + R0 
    cpu.m[0x3002] = 0b0001_000_000_000_000; // ADD R0,R0 + R0
    cpu.m[0x3003] = 0b0001_000_000_1_01010; // ADD R0,R0 + 10
    cpu.m[0x3004] = 0b0001_000_000_1_11011; // ADD R0,ZZ + -5
    cpu.m[0x3005] = 0xF021; // TRAP x21 (OUT)
    cpu.m[0x3006] = 0b1110_111_1_1111_1110; // LEA R7, -2 (0x3007 -2 = 0x3005)
    cpu.m[0x3007] = 0xF025; // TRAP x25 (HALT)

    while cpu.step() {}

    println!("\npc: {:#X}, r: {:#X?}, psr: {:#b}", cpu.pc, cpu.r, cpu.psr);
}


