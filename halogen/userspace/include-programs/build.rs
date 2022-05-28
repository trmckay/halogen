use std::{fs, process::Command};

const PROGRAMS_DIR: &str = "../programs";
const LIBUSER_DIR: &str = "../libuser";

fn main() {
    println!("cargo:rerun-if-changed={}", PROGRAMS_DIR);
    println!("cargo:rerun-if-changed=build.rs");

    if !Command::new("make")
        .arg("-B")
        .current_dir(LIBUSER_DIR)
        .status()
        .expect("failed to exec make")
        .success()
    {
        panic!("build failed")
    }

    fs::read_dir(PROGRAMS_DIR)
        .expect("error reading programs directory")
        .filter_map(|entry| {
            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    if entry.path().is_dir() {
                        Some(path)
                    } else {
                        None
                    }
                }
                Err(_) => None,
            }
        })
        .for_each(|dir| {
            if !Command::new("make")
                .arg("-B")
                .arg({
                    let mut elf = dir.file_name().expect("invalid directory").to_os_string();
                    elf.push(".elf");
                    elf.as_os_str().to_owned()
                })
                .current_dir(dir)
                .status()
                .expect("failed to exec make")
                .success()
            {
                panic!("build failed")
            }
        });
}
