use std::cmp::Ordering;
use std::io::{self, BufRead};

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
            0x0 => { // BR
                let nzp = (self.psr & 0x7) as u16;  // select psr's nzp flag
                if (dr as u16 & nzp) > 0 {          // BR nzp flag = dr
                    let offset = sext(inst & 0x01FF, 9);
                    self.pc = self.pc.wrapping_add(offset as u16);
                }
                true
            }
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
            0x4 => { // JSR or JSRR
                let temp = self.pc;
                let pcoffset11 = sext(inst & 0x07FF, 11);
                self.pc = if dr < 4 { // bit 11 is off
                    self.r[sr1] 
                } else { // JSR
                    self.pc.wrapping_add(pcoffset11 as u16)
                };
                self.r[7] = temp;
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
            0xC => { // JMP or RET (RET = JMP R7) 
                self.pc = self.r[sr1]; // BaseR is just same as SR1
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
                println!(" HALT");
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

    let _ = load_program(&mut cpu);
    println!("------------------");

    while cpu.step() {}

    println!("\npc: {:#X}, r: {:#X?}, psr: {:#b}", cpu.pc, cpu.r, cpu.psr);
}

fn load_program(cpu: &mut CPU) -> io::Result<()> {
    let mut address = 0x3000;

    let stdin = io::stdin();
    
    for line in stdin.lock().lines() {
        let line = line?;
        let trimmed = line.trim();
        
        if trimmed.is_empty() {
            continue;
        }
        
        let Some(first) = trimmed.split_whitespace().next() else { todo!() };

        let result = if first.starts_with("0x") || first.starts_with("0X") {
            u16::from_str_radix(&first[2..], 16)
        } else if first.starts_with("0b") || first.starts_with("0B") {
            u16::from_str_radix(&first[2..], 2)
        } else {
            Ok(0u16)
        };

        match result {
            Ok(value) => {
                cpu.m[address] = value;
                println!("m[0x{:04X}] = 0x{:04X}", address, value);
                address += 1; 
            }
            Err(_) => {
                eprintln!("error '{}'", first);
            }
        } 
    }

     Ok(())
}
