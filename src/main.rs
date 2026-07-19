type Word = u16;
const NUM_REGISTER: usize = 8;
// 16ビットのアドレス空間（0x0000 - 0xFFFF）なのでサイズは 0x10000
const MEMORY_SIZE: usize = 0x10000; 

// instruction mask
const OPCODE_MASK: u16 = 0xF000;

// Opcodes
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

fn main() {
    let mut cpu = CPU::new();

    // テストプログラムのロード
    cpu.pc = 0x3000;
    cpu.r[0] = 0x0041;          // 'A'
    cpu.m[0x3000] = 0xF021;     // TRAP x21 (OUT)
    cpu.m[0x3001] = 0xF025;     // TRAP x25 (HALT)

    // クロックループ
    while cpu.step() {}

    println!("Stopped. pc: {:#X}, r: {:?}, psr: {:#b}", cpu.pc, cpu.r, cpu.psr);
}
