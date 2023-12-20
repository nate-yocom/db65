use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};
use std::os::raw::c_char;
extern "C" {
    pub fn ExecuteInsn() -> u32;

}
extern "C" {
    pub fn Reset();

}
extern "C" {
    pub fn ReadRegisters() -> *mut CPURegs;

}
#[no_mangle]
extern "C" fn MemWriteByte(_addr: u32, _val: u8) {
    println!("MemWriteByte {:04x} {:02x}", _addr, _val);
}

#[no_mangle]
extern "C" fn MemReadWord(_addr: u32) -> u32 {
    println!("MemReadWord {:04x}", _addr);
    return 0;
}
#[no_mangle]
extern "C" fn MemReadByte(_addr: u32) -> u8 {
    println!("MemReadByte {:04x}", _addr);
    return 0;
}
#[no_mangle]
extern "C" fn MemReadZPWord(_addr: u32) -> u32 {
    println!("MemReadZPWord");
    return 0;
}
#[no_mangle]
extern "C" fn Warning(_format: *const c_char, x: u32, y: u32) -> u32 {
    println!("Warning");
    return 0;
}
#[no_mangle]
extern "C" fn Error(_format: *const c_char, x: u32, y: u32) -> u32 {
    println!("Error");
    return 0;
}
#[repr(C)]
pub struct CPURegs {
    ac: u32, /* Accumulator */
    xr: u32, /* X register */
    yr: u32, /* Y register */
    zr: u32, /* Z register */
    sr: u32, /* Status register */
    sp: u32, /* Stackpointer */
    pc: u32, /* Program counter */
}
#[no_mangle]

extern "C" fn ParaVirtHooks(regs: *mut CPURegs) {
    println!("MemzpReadByte");
}
fn main() -> Result<()> {
    // `()` can be used when no completer is required
    let mut rl = DefaultEditor::new()?;
    #[cfg(feature = "with-file-history")]
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    unsafe {
        Reset();
        ExecuteInsn();
        let regs = ReadRegisters();
        println!("ac: {:02x}", (*regs).ac);
        println!("xr: {:02x}", (*regs).xr);
        println!("yr: {:02x}", (*regs).yr);
        println!("zr: {:02x}", (*regs).zr);
        println!("sr: {:02x}", (*regs).sr);
        println!("sp: {:02x}", (*regs).sp);
        println!("pc: {:02x}", (*regs).pc);
    }
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                println!("Line: {}", line);
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    #[cfg(feature = "with-file-history")]
    rl.save_history("history.txt");
    Ok(())
}
