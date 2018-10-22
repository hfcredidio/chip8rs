mod cpu;

use cpu::Cpu;

fn main() {
    let cpu = Cpu{
        ram: [0; 0x1000],
        v: [0; 0x10],
        dt: 0,
        i: 0,
        pc: 0,
        sp: 0,
        st: 0,
    };
}
