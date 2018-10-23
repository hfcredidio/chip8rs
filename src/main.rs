mod cpu;

use cpu::Cpu;
use std::io::Read;

fn main() {
    let mut cpu = Cpu::new();
    let mut file = std::fs::File::open("games/TANK").unwrap(); 
    let mut data: [u8; 0x1000] = [0; 0x1000];
    let size = file.read(&mut data).unwrap();
    cpu.load_rom(&data, size);
    while true {
        cpu.decrement_timers();
        cpu.read_word();
        std::thread::sleep(std::time::Duration::from_millis(17));
    }
    println!("{}", size);
}
