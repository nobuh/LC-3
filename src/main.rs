type Word = u16;
const NUM_REGISTER: usize = 8;
const MEMORY_SIZE: usize = 0xffff;

struct CPU {
    r: [Word; NUM_REGISTER],  // registers
    m: [Word; MEMORY_SIZE],   // memory
}

fn main() {
    let mut p = CPU {
        r: [0; NUM_REGISTER],
        m: [0; MEMORY_SIZE],
    };

    // just test 
    p.r[0] = 0x41; // A
    let lowbyte = (p.r[0] & 0xff) as u8;
    println!("{}", lowbyte as char); 
}
