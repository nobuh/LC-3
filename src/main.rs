type Word = u16;
const NUM_REGISTER: usize = 8;
// 16ビットのアドレス空間（0x0000 - 0xFFFF）なのでサイズは 0x10000
const MEMORY_SIZE: usize = 0x10000; 

// instruction mask
const OPCODE_MASK: u16 = 0xF000;
const IMMEDIATE: u16 = 0b0000_000_000_1_00000;
const IMM5: u16 = 0b0000_000_000_0_11111;
const REG3: u16 = 0b0000_0000_0000_0111;

// Opcodes
const OP_ADD: u16  = 0x1000;
const OP_TRAP: u16 = 0xF000;

struct CPU {
    r: [Word; NUM_REGISTER],    // 汎用レジスタ (R0 - R7)
    m: [Word; MEMORY_SIZE],     // メモリ
    pc: Word,                   // PCも16ビットで管理
    psr: Word,                  // プロセッサステータスレジスタ
}

impl CPU {
    fn new() -> Self {
        Self {
            r: [0; NUM_REGISTER],
            m: [0; MEMORY_SIZE], // 初期値は0（または任意の実装）
            pc: 0x3000,          // LC-3のプログラムは通常0x3000から開始されます
            psr: 0x0002,         // 初期状態はZ（Zero）フラグが立っていることが多いです
        }
    }

    // Fetch - Execute の1サイクルを実行
    fn step(&mut self) -> bool {
        // 1. Fetch
        let instruction = self.m[self.pc as usize];
        
        // LC-3のPCは16ビットで自然にオーバーフロー（ラップアラウンド）します
        self.pc = self.pc.wrapping_add(1);

        // 2. Decode & Execute
        let opcode = instruction & OPCODE_MASK;
        match opcode {
            OP_ADD => { // ADD
                if (instruction & IMMEDIATE) > 0 { // add SR1 + immediate signed 5 bit to DR
                    let imm5 = instruction & IMM5;
                    let sr1 = (instruction >> 6) & REG3;
                    let dr = (instruction >> 9) & REG3;
                    self.r[dr as usize] = self.r[sr1 as usize] + sext(imm5, 5);
                } else {                            // add SR1 + SR2 to DR
                    let sr2 = instruction & IMM5;
                    let sr1 = (instruction >> 6) & REG3;
                    let dr = (instruction >> 9) & REG3;
                    self.r[dr as usize] = self.r[sr1 as usize] + self.r[sr2 as usize];
                }
                true
            }
            OP_TRAP => {
                let trap_vector = instruction & 0x00FF; // 下位8ビットがトラップベクトル
                self.trap(trap_vector)
            }
            _ => {
                // 未実装の命令に当たったらループを抜けるためにfalseを返す
                false 
            }
        }
    }

    fn trap(&mut self, vector: Word) -> bool {
        match vector {
            0x21 => { // OUT
                let lowbyte = (self.r[0] & 0xFF) as u8;
                print!("{}", lowbyte as char);
                true
            }
            0x25 => { // HALT
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

fn sext(value: Word, msb_index: u32) -> Word {
    if msb_index >= 15 {
        return value;
    }

    let shift = 16 - 1 - msb_index;
    let extended = ((value as i16) << shift) >> shift;
    extended as Word
}

fn main() {
    let mut cpu = CPU::new();

    // テストプログラムのロード
    cpu.pc = 0x3000;
    cpu.m[0x3000] = 0b0001_000_000_1_01111; // ADD R0,R0 + 15
    cpu.m[0x3001] = 0b0001_000_000_000_000; // ADD R0,R0 + R0
    cpu.m[0x3002] = 0b0001_000_000_000_000; // ADD R0,R0 + R0
    cpu.m[0x3003] = 0b0001_000_000_1_00101; // ADD R0,R0 + 5
    cpu.m[0x3004] = 0xF021;     // TRAP x21 (OUT)
    cpu.m[0x3005] = 0xF025;     // TRAP x25 (HALT)

    // クロックループ
    while cpu.step() {}

    println!("Stopped. pc: {:#X}, r: {:?}, psr: {:#b}", cpu.pc, cpu.r, cpu.psr);
}
