pub struct Cpu {
    pub ram: [u8; 0x1000],
    pub v: [u8; 0x10],
    pub dt: u8,
    pub i: u16,
    pub pc: u16,
    pub sp: u16,
    pub st: u8,
}
