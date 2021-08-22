use std::fmt::Display;
use std::process::Command;
use std::io;
use std::io::Write;

pub fn wasm_file(s:impl AsRef<str>+Display) ->String{
    format!("../target/wasm32-unknown-unknown/release/{}.wasm",s)
}

pub fn near_command(near_cmd:impl AsRef<str>+Display, near_arg:impl AsRef<str>+Display){
    println!("running near {} {}", near_cmd, near_arg);
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(&["/C",format!("near {} {}", near_cmd, near_arg).as_str()])
            .output()
            .expect("ls command failed to start")
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(format!("near {} {}", near_cmd, near_arg))
            .output()
            .expect("ls command failed to start")
    };
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();
}

