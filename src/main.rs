use arch::vm::VM;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;

mod arch;
pub mod test;

pub static mut VM_MODE: bool = false;
pub static mut DASM_MODE: bool = false;
pub static mut INPUT_FILE: &str = "";
pub static mut FLAG: usize = 0;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let mode = &args[1];
    let mut filename = "";

    match mode.as_str() {
        "-m" => {
            unsafe {
                VM_MODE = true;
            };
            filename = &args[2];
        }
        "-d" => {
            unsafe {
                DASM_MODE = true;
            };
            filename = &args[2];
        }
        _ => {
            filename = &args[1];
        }
    }

    let content = read_file_content(&filename)?;

    if mode == "-d" {
        let mut vm = VM::new(content, &args);
        let asm = vm.disassemble();

        for a in asm {
            println!("{:?}", a);
        }

        return Ok(());
    }

    let bin_args = if unsafe { VM_MODE } {
        &args[2..]
    } else {
        &args[1..]
    };

    let mut vm = VM::new(content, bin_args);
    vm.run();

    Ok(())
}

fn read_file_content(path: &str) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;

    let mut buf = Vec::new();
    let x = BufReader::new(file).read_to_end(&mut buf)?;

    Ok(buf)
}

fn get_filename_from_path(path: &str) -> Option<&str> {
    Path::new(path).file_name()?.to_str()
}
