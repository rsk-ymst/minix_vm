use std::{
    fs::File,
    io::{self, BufRead, BufReader, Read},
    process::Command,
};

#[test]
fn dasm_1() {
    exec_case("1c");
}

#[test]
fn dasm_2() {
    exec_case("2c");
}

#[test]
fn dasm_3() {
    exec_case("3c");
}

#[test]
fn dasm_4() {
    exec_case("4c");
}

#[test]
fn dasm_5() {
    exec_case("5c");
}

#[test]
fn dasm_6() {
    exec_case("6c");
}

#[test]
fn dasm_7() {
    exec_case("7c");
}

#[test]
fn dasm_nm() {
    exec_case("nm");
}

// #[test]
// fn dasm_08() {
//     exec_case(8);
// }

// #[test]
// fn dasm_09() {
//     exec_case(9);
// }

#[test]
fn vm_1() {
    exec_vm_case("1c");
}

#[test]
fn vm_2() {
    exec_vm_case("2c");
}

#[test]
fn vm_3() {
    exec_vm_case("3c");
}

#[test]
fn vm_4() {
    exec_vm_case("4c");
}

#[test]
fn vm_5() {
    exec_vm_case("5c");
}

#[test]
fn vm_6() {
    exec_vm_case("6c");
}

#[test]
fn vm_7() {
    exec_vm_case("7c");
}

#[test]
fn vm_nm() {
    exec_vm_case("nm");
}

#[test]
fn default_1() {
    exec_default_case("1c");
}

#[test]
fn default_2() {
    exec_default_case("2c");
}

#[test]
fn default_3() {
    exec_default_case("3c");
}

#[test]
fn default_4() {
    exec_default_case("4c");
}

#[test]
fn default_5() {
    exec_default_5c_case();
}

#[test]
fn default_6() {
    exec_default_case("6c");
}

#[test]
fn default_7() {
    exec_default_case("7c");
}

// fn default_7() {
//     exec_default_case("7c");
// }

#[test]
fn default_nm() {
    exec_default_case("nm");
}

// #[test]
// fn vm_8() {
//     exec_vm_case(8);
// }

fn exec_case(filename: &str) {
    exec_disassembler(filename);

    let origin_asm = format!("./origin/asm/{}.txt", filename);
    let output_asm = format!("./out/asm/{}.txt", filename);

    compare_lines(&origin_asm, &output_asm);
}

fn exec_vm_case(filename: &str) {
    exec_vm(filename);

    let origin_vm = format!("./origin/vm/{}.txt", filename);
    let output_vm = format!("./out/vm/{}.txt", filename);

    compare_lines(&origin_vm, &output_vm);
}

fn exec_default_case(filename: &str) {
    exec_default(filename);

    let origin_vm = format!("./origin/default/{}.txt", filename);
    let output_vm = format!("./out/default/{}.txt", filename);

    compare_lines(&origin_vm, &output_vm);
}

fn exec_default_5c_case() {
    let filename = "5c";
    exec_default_5c();

    let origin_vm = format!("./origin/default/{}.txt", filename);
    let output_vm = format!("./out/default/{}.txt", filename);

    compare_lines(&origin_vm, &output_vm);
}

fn exec_disassembler(filename: &str) {
    Command::new("./src/test/shell/dasm.sh")
        .args([&filename])
        .output()
        .expect("Failed to execute cargo run");
}

fn exec_vm(filename: &str) {
    Command::new("./src/test/shell/vm.sh")
        .args([&filename])
        .output()
        .expect("Failed to execute cargo run");
}

fn exec_default(filename: &str) {
    Command::new("./src/test/shell/default.sh")
        .args([&filename])
        .output()
        .expect("Failed to execute cargo run");
}

fn exec_default_5c() {
    Command::new("./src/test/shell/default_5c.sh")
        .output()
        .expect("Failed to execute cargo run");
}

fn compare_lines(file1: &str, file2: &str) -> io::Result<()> {
    let file1 = File::open(file1).unwrap();
    let mut lines1 = BufReader::new(file1).lines();

    let file2 = File::open(file2).unwrap();
    let mut lines2 = BufReader::new(file2).lines();

    let mut i = 1;
    loop {
        let line1 = lines1.next();
        let line2 = lines2.next();

        if line1.is_none() && line2.is_none() {
            break;
        }

        assert_eq!(
            line1.unwrap_or(Ok("inconsistent".to_owned()))?,
            line2.unwrap_or(Ok("inconsistent".to_owned()))?,
            "line number: {i}"
        );
        i += 1;
    }

    Ok(())
}
