use std::path::Path;
use std::process::Command;

pub fn assemble(asm_path: &Path, exe_path: &Path) {
    let output = if cfg!(target_os = "windows") {
        panic!("Cannot compile the assembly on windos")
    } else {
        Command::new("gcc")
            .arg(asm_path.to_str().unwrap())
            .arg("-o")
            .arg(exe_path.to_str().unwrap())
            .output()
            .expect("failed to compile")
    };

    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        println!("Error:");
        println!("{}", stderr);
        panic!();
    }
}