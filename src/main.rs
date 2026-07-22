use std::cmp::Ordering;

type Word = u16;
const NUM_REGISTER: usize = 8;
const MEMORY_SIZE: usize = 0x10000;

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
        let inst = self.m[self.pc as usize];
        self.pc = self.pc.wrapping_add(1);

        let dr = ((inst >> 9) & 0x7) as usize;
        let sr1 = ((inst >> 6) & 0x7) as usize;

        match inst >> 12 {
            0x1 => { // ADD
                let val2 = if (inst & 0x0020) != 0 {
                    sext(inst & 0x001F, 5) as u16 // Immediate (imm5)
                } else {
                    self.r[(inst & 0x7) as usize] // Register (sr2)
                };
                self.r[dr] = self.r[sr1].wrapping_add(val2);
                self.update_flags(self.r[dr]); 
                true
            }
            0x5 => { // AND
                let val2 = if (inst & 0x0020) != 0 {
                    sext(inst & 0x001F, 5) as u16 // Immediate (imm5)
                } else {
                    self.r[(inst & 0x7) as usize] // Register (sr2)
                };
                self.r[dr] = self.r[sr1] & val2;
                self.update_flags(self.r[dr]); 
                true
            }
            0xE => { // LEA
                let offset = sext(inst & 0x01FF, 9);
                self.r[dr] = self.pc.wrapping_add(offset as u16);
                true
            }
            0xF => self.trap(inst & 0x00FF), // TRAP
            _ => false,
        }
    }

    fn update_flags(&mut self, val: Word) {
        self.psr &= !0x7; // clear NZP flag 
        self.psr |= match (val as i16).cmp(&0) {
            Ordering::Less => 0b100,    // N
            Ordering::Equal => 0b010,   // Z
            Ordering::Greater => 0b001, // P
        };
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

fn sext(v: u16, bit_count: u32) -> i16 {
    let shift = 16 - bit_count;
    ((v as i16) << shift) >> shift
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

    cpu.m[0x3007] = 0b0001_001_001_1_11110; // ADD R1, R1 + 0b11110 (-2)
    cpu.m[0x3008] = 0b0001_010_010_1_11011; // ADD R2, R2 + 0b11011 (-5)
    cpu.m[0x3009] = 0b0101_001_001_1_11111; // AND R1, R1 & 0b11111 (-1) => 0b11110 (-2)
    cpu.m[0x300A] = 0b0101_011_001_000_010; // AND R3, R1 & R2 => 0b11010 (-6)

    cpu.m[0x300B] = 0xF025; // TRAP x25 (HALT)

    while cpu.step() {}

    println!("\npc: {:#X}, r: {:#X?}, psr: {:#b}", cpu.pc, cpu.r, cpu.psr);
}


