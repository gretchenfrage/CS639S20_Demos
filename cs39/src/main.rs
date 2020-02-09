extern crate regex;
extern crate num_cpus;
extern crate rand;
extern crate byte_unit;

use crate::{
    compile::{
        compile,
        Compiled,
    },
    navigate::{
        DemoLookup,
        demo_lookup,
    },
    output::{Indent, INFO_INDENT},
};
use std::{
    env::args,
    process::Command,
    path::PathBuf,
    str::FromStr,
};

/// C++ compilation.
pub mod compile;

/// Code demo navigation.
pub mod navigate;

/// Program output helpers.
pub mod output;

/// `size_test` task.
pub mod size_test;

/// Extract and parse a regex capture group.
pub fn cap_parse<T: FromStr>(cap: &regex::Captures, group: &str) -> Option<T> {
    cap.name(group).and_then(|m| m.as_str().parse().ok())
}

/// `run` task.
pub fn run_demo(lookup: &DemoLookup, major: u32, minor: u32) -> Result<(), ()> {
    let Compiled { workdir, binary } = compile(lookup, major, minor)?;
    
    println!("[INFO] running");
    println!();
    let status = Command::new(&binary)
        .current_dir(&workdir)
        .status().unwrap();
    println!();
    println!("[INFO] exit {}", status.code().unwrap());
    
    Ok(())
}

/// `stat` task/subtask.
fn cpu_stat() {
    println!("[INFO] cpu info:");
    println!("{}", Indent(INFO_INDENT, ""));
    println!("{}", Indent(INFO_INDENT, 
        format_args!("LOGICAL CPUS = {}", num_cpus::get())));
    println!("{}", Indent(INFO_INDENT,
        format_args!("PHYSICAL CPUS = {}", num_cpus::get_physical())));
    println!("{}", Indent(INFO_INDENT, ""));
}

/// `cpu_task` test.
fn cpu_test(lookup: &DemoLookup, major: u32, minor: u32) -> Result<(), ()> {
    let Compiled { workdir, binary } = compile(lookup, major, minor)?;
    
    cpu_stat();
    
    let min_cpu = 1;
    let max_cpu = num_cpus::get();
    
    for cpu in min_cpu..=max_cpu {
        println!("[INFO] benchmarking with {} thread", cpu);
        let status = Command::new(&binary)
            .current_dir(&workdir)
            .env("OMP_NUM_THREADS", cpu.to_string())
            .status().unwrap();
        println!();
        if !status.success() {
            println!("[ERROR] exit code {}", status.code().unwrap());
            return Err(());
        }
    }
    
    println!("[INFO] done");
    
    Ok(())
} 

/// `reinstall` subtask.
pub fn reinstall() {
    println!("[INFO] recompiling cs39 cli");
    let _ = Command::new("cargo")
        .arg("install")
        .arg("--path")
        .arg(env!("CARGO_MANIFEST_DIR"))
        .arg("--force")
        .status();
    println!();
}

/// CLI parsing helper.
pub fn get_version(args: &[String]) -> (u32, u32) {
    assert!(args.len() >= 4, "unexpected num of args");

    let major: u32 = args[2].parse().unwrap();
    let minor: u32 = args[3].parse().unwrap();
    
    (major, minor)
}

fn main() {
    let args: Vec<String> = args().collect();
    
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .canonicalize().unwrap()
        .parent().map(PathBuf::from).unwrap();
        
    let lookup = demo_lookup(&repo);
    
    match args[1].as_str() {
        "help" => {
            println!(include_str!("../manual.txt"));
        },
        "list" => {
            println!("[INFO] listing demos");
            println!("{:#?}", lookup);
        },
        "reinstall" => {
            reinstall();
        },
        "stat" => {
            cpu_stat();
        },
        "run" => {
            let (major, minor) = get_version(&args);
            let _ = run_demo(&lookup, major, minor);
        },
        "cpu_test" => {
            let (major, minor) = get_version(&args);
            let _ = cpu_test(&lookup, major, minor);
        },
        "size_test" => {
            let (major, minor) = get_version(&args);
            let _ = size_test::run(&repo, &lookup, major, minor);
        }
        _ => {
            println!(include_str!("../manual.txt"));
        },
    }
}
