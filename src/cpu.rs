pub struct Cpu {
    ram: [u8; 0x1000],
    v: [u8; 0x10],
    dt: u8,
    i: u16,
    pc: u16,
    sp: u16,
    st: u8,
}


impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            ram: [0; 0x1000],
            v: [0; 0x10],
            dt: 0,
            i: 0,
            pc: 0x200,
            sp: 0xFFF,
            st: 0,
        }
    }

    fn read_memory(&self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }
    fn write_memory(&mut self, addr: u16, val: u8) {
        self.ram[addr as usize] = val;
    }
    fn read_register(&self, x: u8) -> u8 {
        self.v[x as usize]
    }
    fn write_register(&mut self, x: u8, val: u8) {
        self.v[x as usize] = val;
    }
    fn push_stack(&mut self, addr: u16) {
        self.sp -= 2;
        let sp = self.sp;
        self.write_memory(sp, (addr >> 8) as u8);
        self.write_memory(sp + 1, addr as u8);
    }
    fn pop_stack(&mut self) -> u16 {
        let n1 = self.read_memory(self.sp);
        let n2 = self.read_memory(self.sp + 1);
        self.sp += 2;
        (n1 as u16) << 8 | n2 as u16
    }
    pub fn load_rom(&mut self, buf: &[u8], size: usize) {
        for i in 0..size {
            self.ram[i + 0x200] = buf[i];
        }
    }
    pub fn read_word(&mut self) {
        let opcode = (self.read_memory(self.pc) as u16) << 8 | self.read_memory(self.pc + 1) as u16;
        println!("{:X}\t{}", self.pc, format_opcode(opcode));
        self.call_opcode(opcode);
        self.pc += 2
    }
    pub fn decrement_timers(&mut self) {
        self.dt -= if self.dt > 0 { 1 } else { 0 };
        self.st -= if self.st > 0 { 1 } else { 0 };
    }

    pub fn call_opcode(&mut self, word: u16) {
        let (c, Vx, Vy, nibble) = word_to_nibbles(word);
        let addr = word & 0x0FFF;
        let byte = (word & 0x00FF)  as u8;
        match (c, Vx, Vy, nibble) {
            (0x0, 0x0, 0xE, 0x0) => self.CLS(),
            (0x0, 0x0, 0xE, 0xE) => self.RET(),
            (0x0,   _,   _,   _) => self.SYS(addr),
            (0x1,   _,   _,   _) => self.JP(addr),
            (0x2,   _,   _,   _) => self.CALL(addr),
            (0x3,   _,   _,   _) => self.SE(Vx, byte),
            (0x4,   _,   _,   _) => self.SNE(Vx, byte),
            (0x5,   _,   _, 0x0) => self.SE2(Vx, Vy),
            (0x6,   _,   _,   _) => self.LD(Vx, byte),
            (0x7,   _,   _,   _) => self.ADD(Vx, byte),
            (0x8,   _,   _, 0x0) => self.LD2(Vx, Vy),
            (0x8,   _,   _, 0x1) => self.OR(Vx, Vy),
            (0x8,   _,   _, 0x2) => self.AND(Vx, Vy),
            (0x8,   _,   _, 0x3) => self.XOR(Vx, Vy),
            (0x8,   _,   _, 0x4) => self.ADD2(Vx, Vy),
            (0x8,   _,   _, 0x5) => self.SUB(Vx, Vy),
            (0x8,   _,   _, 0x6) => self.SHR(Vx, Vy),
            (0x8,   _,   _, 0x7) => self.SUBN(Vx, Vy),
            (0x8,   _,   _, 0xE) => self.SHL(Vx, Vy),
            (0x9,   _,   _, 0x0) => self.SNE2(Vx, Vy),
            (0xA,   _,   _,   _) => self.LDI(addr),
            (0xB,   _,   _,   _) => self.JPV0(addr),
            (0xC,   _,   _,   _) => self.RND(Vx, byte),
            (0xD,   _,   _,   _) => self.DRW(Vx, Vy, nibble),
            (0xE,   _, 0x9, 0xE) => self.SKP(Vx),
            (0xE,   _, 0xA, 0x1) => self.SKNP(Vx),
            (0xF,   _, 0x0, 0x7) => self.LDT(Vx),
            (0xF,   _, 0x0, 0xA) => self.LDK(Vx),
            (0xF,   _, 0x1, 0x5) => self.LDT2(Vx),
            (0xF,   _, 0x1, 0x8) => self.LDS(Vx),
            (0xF,   _, 0x1, 0xE) => self.ADDI(Vx),
            (0xF,   _, 0x2, 0x9) => self.LDD(Vx),
            (0xF,   _, 0x3, 0x3) => self.LDB(Vx),
            (0xF,   _, 0x5, 0x5) => self.STR(Vx),
            (0xF,   _, 0x6, 0x5) => self.LDR(Vx),
            (  _,   _,   _,   _) => self.SYS(addr),
        };
    }

    fn SYS(&self, addr: u16) {}
    fn CLS(&self) {}
    fn RET(&mut self) {
        self.pc = self.pop_stack();
    }
    fn JP(&mut self, addr: u16) {
        self.pc = addr - 2;
    }
    fn CALL(&mut self, addr: u16) {
        let pc = self.pc;
        self.push_stack(pc);
        self.pc = addr - 2;
    }
    fn SE(&mut self, x: u8, byte: u8) {
        let xval = self.read_register(x);
        if xval == byte {
            self.pc += 2;
        }
    }
    fn SNE(&mut self, x: u8, byte: u8) {
        let xval = self.read_register(x);
        if xval != byte {
            self.pc += 2;
        }
    }
    fn SE2(&mut self, x: u8, y: u8) {
        let xval = self.read_register(x);
        let yval = self.read_register(y);
        if xval == yval {
            self.pc += 2;
        }
    }
    fn LD(&mut self, x: u8, byte: u8) {
        self.write_register(x, byte);
    }
    fn ADD(&mut self, x: u8, byte: u8) {
        let xval = self.read_register(x);
        self.write_register(x, xval + byte);
    }
    fn LD2(&mut self, x: u8, y: u8) {
        let yval = self.read_register(y);
        self.write_register(x, yval);
    }
    fn OR(&mut self, x: u8, y: u8) {
        let xval = self.read_register(x);
        let yval = self.read_register(y);
        self.write_register(x, xval | yval);
    }
    fn AND(&mut self, x: u8, y: u8) {
        let xval = self.read_register(x);
        let yval = self.read_register(y);
        self.write_register(x, xval & yval);
    }
    fn XOR(&mut self, x: u8, y: u8) {
        let xval = self.read_register(x);
        let yval = self.read_register(y);
        self.write_register(x, xval ^ yval);
    }
    fn ADD2(&mut self, x: u8, y: u8) {
        let xval = self.read_register(x);
        let yval = self.read_register(y);
        let res: u16 = (xval as u16) + (yval as u16);
        let carry = (res & 0xFF00) >> 8;
        self.write_register(x, res as u8);
        self.write_register(0xF, (carry != 0) as u8);
    }
    fn SUB(&mut self, x: u8, y: u8) {
        let xval = self.read_register(x);
        let yval = self.read_register(y);
        let res = xval - yval;
        let borrow = (xval > yval) as  u8;
        self.write_register(x, res);
        self.write_register(0xF, borrow);
    }
    fn SHR(&mut self, x: u8, y: u8) {
        let yval = self.read_register(y);
        self.write_register(x, yval >> 1);
        self.write_register(0xF, yval & 0x01);
    }
    fn SUBN(&mut self, x: u8, y: u8) {
        let xval = self.read_register(x);
        let yval = self.read_register(y);
        let res = yval - xval;
        let borrow = (yval > xval) as  u8;
        self.write_register(x, res);
        self.write_register(0xF, borrow);
    }
    fn SHL(&mut self, x: u8, y: u8) {
        let yval = self.read_register(y);
        self.write_register(x, yval << 1);
        self.write_register(0xF, yval >> 7);
    }
    fn SNE2(&mut self, x: u8, y: u8) {
        let xval = self.read_register(x);
        let yval = self.read_register(y);
        if xval != yval {
            self.pc += 2;
        }
    }
    fn LDI(&mut self, addr: u16) {
        self.i = addr;
    }
    fn JPV0(&mut self, addr: u16) {
        self.pc = self.read_register(0x0) as u16 + addr - 2;
    }
    fn RND(&mut self, x: u8, byte: u8) {
        let rnd = 0;
        self.write_register(x, rnd & byte);
    }
    fn DRW(&mut self, x: u8, y: u8, nibble: u8) {
        // todo
    }
    fn SKP(&mut self, x: u8) {
        // todo
    }
    fn SKNP(&mut self, x: u8) {
        // todo
    }
    fn LDT(&mut self, x: u8) {
        let dt = self.dt;
        self.write_register(x, dt);
    }
    fn LDK(&mut self, x: u8) {
        // toddo
    }
    fn LDT2(&mut self, x: u8) {
        self.dt = self.read_register(x);
    }
    fn LDS(&mut self, x: u8) {
        self.st = self.read_register(x);
    }
    fn ADDI(&mut self, x: u8) {
        self.i += self.read_register(x) as u16;
    }
    fn LDD(&mut self, x: u8) {
        // todo
    }
    fn LDB(&mut self, x: u8) {
        let xval = self.read_register(x);
        let c = xval / 100;
        let d = (xval - c * 100) / 10;
        let u = xval - c * 100 - d * 10;
        let addr = self.i;
        self.write_memory(addr, c);
        self.write_memory(addr + 1, c);
        self.write_memory(addr + 2, u);
        self.i += 2;
    }
    fn STR(&mut self, x: u8) {
        let addr = self.i;
        for reg in 0..x {
            let val = self.read_register(reg);
            self.write_memory(addr + reg as u16, val);
        }
        self.i += x as u16 + 1;
    }
    fn LDR(&mut self, x: u8) {
        for reg in 0..x {
            let val = self.read_memory(self.i + x as u16);
            self.write_register(reg, val);
        }
        self.i += x as u16;
    }
}


fn word_to_nibbles(word: u16) -> (u8, u8, u8, u8) {
    (
        ((word & 0xF000) >> 12) as u8,
        ((word & 0x0F00) >>  8) as u8,
        ((word & 0x00F0) >>  4) as u8,
        ((word & 0x000F) >>  0) as u8,
    )
}



pub fn format_opcode(word: u16) -> String {
    let (c, Vx, Vy, nibble) = word_to_nibbles(word);
    let addr = word & 0x0FFF;
    let byte = (word & 0x00FF)  as u8;
    let fmt = match (c, Vx, Vy, nibble) {
        (0x0, 0x0, 0xE, 0x0) => "CLS".to_string(),
        (0x0, 0x0, 0xE, 0xE) => "RET".to_string(),
        (0x0,   _,   _,   _) => format!("SYS\t{:X}", addr),
        (0x1,   _,   _,   _) => format!("JP\t{:X}", addr),
        (0x2,   _,   _,   _) => format!("CALL\t{:X}", addr),
        (0x3,   _,   _,   _) => format!("SE\t[{:X}] {:X}", Vx, byte),
        (0x4,   _,   _,   _) => format!("SNE\t[{:X}] {:X}", Vx, byte),
        (0x5,   _,   _, 0x0) => format!("SE2\t[{:X}] [{:X}]", Vx, Vy),
        (0x6,   _,   _,   _) => format!("LD\t[{:X}] {:X}", Vx, byte),
        (0x7,   _,   _,   _) => format!("ADD\t[{:X}] {:X}", Vx, byte),
        (0x8,   _,   _, 0x0) => format!("LD2\t[{:X}] [{:X}]", Vx, Vy),
        (0x8,   _,   _, 0x1) => format!("OR\t[{:X}] [{:X}]", Vx, Vy),
        (0x8,   _,   _, 0x2) => format!("AND\t[{:X}] [{:X}]", Vx, Vy),
        (0x8,   _,   _, 0x3) => format!("XOR\t[{:X}] [{:X}]", Vx, Vy),
        (0x8,   _,   _, 0x4) => format!("ADD2\t[{:X}] [{:X}]", Vx, Vy),
        (0x8,   _,   _, 0x5) => format!("SUB\t[{:X}] [{:X}]", Vx, Vy),
        (0x8,   _,   _, 0x6) => format!("SHR\t[{:X}] [{:X}]", Vx, Vy),
        (0x8,   _,   _, 0x7) => format!("SUBN\t[{:X}] [{:X}]", Vx, Vy),
        (0x8,   _,   _, 0xE) => format!("SHL\t[{:X}] [{:X}]", Vx, Vy),
        (0x9,   _,   _, 0x0) => format!("SNE2\t[{:X}] [{:X}]", Vx, Vy),
        (0xA,   _,   _,   _) => format!("LDI\t{:X}", addr),
        (0xB,   _,   _,   _) => format!("JPV0\t{:X}", addr),
        (0xC,   _,   _,   _) => format!("RND\t[{:X}] {:X}", Vx, byte),
        (0xD,   _,   _,   _) => format!("DRW\t[{:X}] [{:X}] {:X}", Vx, Vy, nibble),
        (0xE,   _, 0x9, 0xE) => format!("SKP\t[{:X}]", Vx),
        (0xE,   _, 0xA, 0x1) => format!("SKNP\t[{:X}]", Vx),
        (0xF,   _, 0x0, 0x7) => format!("LDT\t[{:X}]", Vx),
        (0xF,   _, 0x0, 0xA) => format!("LDK\t[{:X}]", Vx),
        (0xF,   _, 0x1, 0x5) => format!("LDT2\t[{:X}]", Vx),
        (0xF,   _, 0x1, 0x8) => format!("LDS\t[{:X}]", Vx),
        (0xF,   _, 0x1, 0xE) => format!("ADDI\t[{:X}]", Vx),
        (0xF,   _, 0x2, 0x9) => format!("LDD\t[{:X}]", Vx),
        (0xF,   _, 0x3, 0x3) => format!("LDB\t[{:X}]", Vx),
        (0xF,   _, 0x5, 0x5) => format!("STR\t[{:X}]", Vx),
        (0xF,   _, 0x6, 0x5) => format!("LDR\t[{:X}]", Vx),
        (  _,   _,   _,   _) => format!("SYS\t{:X}", addr),
    };
    fmt
}
