/*
The debugger core. It sits on top of the Cpu wrapper that has all the unsafe code
It does not do any ui. This allows for maybe a future gui to provide
the same functionality as the cli shell.

*/

use anyhow::{bail, Result};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use crate::{cpu::Cpu, execute::StopReason, loader};
pub struct Debugger {
    symbols: HashMap<String, u16>,
    pub break_points: HashMap<u16, BreakPoint>,
    //pub break_points: HashMap<u16, BreakPoint>,
    pub(crate) next_bp: Option<u16>,
    _loader_sp: u8,
    loader_start: u16,
    pub(crate) dis_line: String,
    pub(crate) ticks: usize,
    pub(crate) stack_frames: Vec<StackFrame>,
    pub(crate) enable_stack_check: bool,
    pub(crate) enable_mem_check: bool,
    load_name: String,
}
#[derive(Debug)]
pub(crate) enum FrameType {
    Jsr((u16, u16, u8, u16)), // addr, return addr,sp,sp65
    Pha(u8),
    Php(u8),
}
#[derive(Debug)]
pub struct StackFrame {
    pub(crate) frame_type: FrameType,
}
#[derive(Debug, Clone)]
pub struct BreakPoint {
    pub(crate) addr: u16,
    pub(crate) symbol: String,
    pub(crate) number: usize,
    pub(crate) temp: bool,
}
impl Debugger {
    pub fn new() -> Self {
        Cpu::reset();
        Self {
            symbols: HashMap::new(),
            break_points: HashMap::new(),
            _loader_sp: 0,
            loader_start: 0,
            dis_line: String::new(),
            ticks: 0,
            stack_frames: Vec::new(),
            enable_stack_check: false,
            enable_mem_check: false,
            next_bp: None,
            load_name: String::new(),
        }
    }
    pub fn delete_breakpoint(&mut self, id_opt: Option<&String>) -> Result<()> {
        if let Some(id) = id_opt {
            if let Ok(num) = usize::from_str_radix(&id, 10) {
                if let Some(find) = self.break_points.iter().find_map(|bp| {
                    if bp.1.number == num {
                        Some(*bp.0)
                    } else {
                        None
                    }
                }) {
                    self.break_points.remove(&find);
                }
            }
            // else lookup symbol?
        } else {
            self.break_points.clear();
        };
        Ok(())
    }
    pub fn set_break(&mut self, addr_str: &str, temp: bool) -> Result<()> {
        let bp_addr;
        let first_char = addr_str.chars().next().unwrap();
        let mut save_sym = String::new();
        if first_char == '.' {
            if let Some(sym) = self.symbols.get(addr_str) {
                save_sym = addr_str.to_string();
                bp_addr = *sym;
            } else {
                bail!("unknown symbol")
            }
        } else {
            if addr_str.chars().next().unwrap() == '$' {
                let rest = addr_str[1..].to_string();
                bp_addr = u16::from_str_radix(&rest, 16)?;
            } else {
                bp_addr = u16::from_str_radix(addr_str, 10)?;
            }
        }
        self.break_points.insert(
            bp_addr,
            BreakPoint {
                addr: bp_addr,
                symbol: save_sym,
                number: self.break_points.len() + 1,
                temp,
            },
        );
        Ok(())
    }
    pub fn load_ll(&mut self, file: &Path) -> Result<()> {
        let fd = File::open(file)?;
        let mut reader = BufReader::new(fd);
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line)? {
                0 => break,
                _len => {
                    //al 000000 .sp
                    let mut spl = line.split(" ");
                    let _al = spl.next();
                    let addr_str = spl.next().unwrap().trim_end();
                    let name = spl.next().unwrap().trim_end();
                    let addr = u16::from_str_radix(addr_str, 16).unwrap();
                    self.symbols.insert(name.to_string(), addr);
                }
            }
        }
        Ok(())
    }
    pub fn load_code(&mut self, file: &Path) -> Result<(u16, u16)> {
        let (sp65_addr, run, _cpu, size) = loader::load_code(file)?;
        // println!("size={:x}, entry={:x}, cpu={}", size, run, cpu);
        Cpu::sp65_addr(sp65_addr);
        let arg0 = file.file_name().unwrap().to_str().unwrap().to_string();
        self.load_name = arg0;
        self.loader_start = run;

        Ok((size, run))
    }
    pub fn get_breaks(&self) -> Vec<u16> {
        self.break_points.iter().map(|bp| bp.1.addr).collect()
    }
    pub fn go(&mut self) -> Result<StopReason> {
        self.core_run()
    }
    fn state(&self) {
        println!("pc={:04x}", Cpu::read_pc());
    }
    pub fn next(&mut self) -> Result<StopReason> {
        let next_inst = Cpu::read_byte(Cpu::read_pc());
        let reason = if next_inst == 0x20 {
            // if the next instruction is a jsr then]
            // set a temp bp on the folloing inst and run
            let inst = Cpu::read_pc() + 3;
            self.next_bp = Some(inst);
            self.execute(0)
        } else {
            // otherwise execute one instruction
            self.execute(1)
        };
        reason
    }
    pub fn get_bp(&self, addr: u16) -> Option<&BreakPoint> {
        return self.break_points.get(&addr);
    }
    pub fn step(&mut self) -> Result<StopReason> {
        self.execute(1)
    }
    fn core_run(&mut self) -> Result<StopReason> {
        self.execute(0) // 0 = forever
    }
    pub fn run(&mut self, cmd_args: Vec<&String>) -> Result<StopReason> {
        Cpu::write_word(0xFFFC, self.loader_start);
        Cpu::reset();
        Cpu::push_arg(&self.load_name);
        for i in 0..cmd_args.len() {
            Cpu::push_arg(&cmd_args[i])
        }
        self.stack_frames.clear();
        self.core_run()
    }
    pub fn get_chunk(&self, addr: u16, mut len: u16) -> Result<Vec<u8>> {
        let mut v = Vec::new();
        let max_add = addr.saturating_add(len);
        len = max_add - addr;
        for i in 0..len {
            v.push(Cpu::read_byte(addr + i));
        }
        Ok(v)
    }
    pub fn convert_addr(&self, addr_str: &str) -> Result<u16> {
        if let Some(sym) = self.symbols.get(addr_str) {
            return Ok(*sym);
        }

        if addr_str.chars().next().unwrap() == '$' {
            let rest = addr_str[1..].to_string();
            return Ok(u16::from_str_radix(&rest, 16)?);
        }
        Ok(u16::from_str_radix(addr_str, 10)?)
    }
    pub fn symbol_lookup(&self, addr: u16) -> String {
        for (name, sym_addr) in &self.symbols {
            if *sym_addr == addr {
                return name.to_string();
            }
        }
        format!("x{:04x}", addr)
    }
    pub fn zp_symbol_lookup(&self, addr: u8) -> String {
        for (name, sym_addr) in &self.symbols {
            if *sym_addr == addr as u16 {
                return name.to_string();
            }
        }
        format!("{:02x}", addr)
    }
    pub fn read_pc(&self) -> u16 {
        Cpu::read_pc()
    }
    pub fn read_sp(&self) -> u8 {
        Cpu::read_sp()
    }
    pub fn read_ac(&self) -> u8 {
        Cpu::read_ac()
    }
    pub fn read_xr(&self) -> u8 {
        Cpu::read_xr()
    }
    pub fn read_yr(&self) -> u8 {
        Cpu::read_yr()
    }
    pub fn read_zr(&self) -> u8 {
        Cpu::read_zr()
    }
    pub fn read_sr(&self) -> u8 {
        Cpu::read_sr()
    }
    pub fn write_ac(&self, v: u8) {
        Cpu::write_ac(v);
    }
    pub fn write_xr(&self, v: u8) {
        Cpu::write_xr(v);
    }
    pub fn write_yr(&self, v: u8) {
        Cpu::write_yr(v);
    }
    pub fn write_zr(&self, v: u8) {
        Cpu::write_zr(v);
    }
    pub fn write_sr(&self, v: u8) {
        Cpu::write_sr(v);
    }
    pub fn write_sp(&self, v: u8) {
        Cpu::write_sp(v);
    }
    pub fn write_pc(&self, v: u16) {
        Cpu::write_pc(v);
    }
    pub fn read_stack(&self) -> &Vec<StackFrame> {
        &self.stack_frames
    }
}
