extern crate regex;
extern crate num_cpus;
use std::{
    env::args,
    str::FromStr,
    path::{PathBuf, Path},
    collections::BTreeMap,
    fs::{read_dir, FileType},
    ffi::{OsStr, OsString},
    process::Command,
};
use regex::Regex;

fn subdirs<P: AsRef<Path>>(path: P) -> impl Iterator<Item=PathBuf> {
    read_dir(path).unwrap()
        .filter_map(Result::ok)
        .filter_map(|f| f.file_type().ok()
            .filter(FileType::is_dir)
            .map(|_| f.path())
        .filter(|p| p.file_name()
            .and_then(OsStr::to_str)
            .and_then(|s| s.chars().next())
            .map(|c| c != '.')
            .unwrap_or(false)))
}

fn cpp_files<P: AsRef<Path>>(path: P) -> impl Iterator<Item=OsString> {
    read_dir(path).unwrap()
        .filter_map(Result::ok)
        .filter_map(|f| f.file_type().ok()
            .filter(FileType::is_file)
            .map(|_| f.path()))
        .filter_map(|p| p.extension()
            .filter(|e| *e == "cpp" || *e == "h")
            .and_then(|_| p.file_name().map(OsStr::to_owned)))
}

fn cap_parse<T: FromStr>(cap: &regex::Captures, group: &str) -> Option<T> {
    cap.name(group).and_then(|m| m.as_str().parse().ok())
}

#[derive(Debug)] 
struct Subdir {
    subdir_path: PathBuf,
    demos: BTreeMap<u32, PathBuf>,
}

type DemoLookup = BTreeMap::<u32, Subdir>;

fn demo_lookup<P: AsRef<Path>>(repo: P) -> DemoLookup {
    let pat = r#"^[[:alnum:]]+_(?P<major>\d+)_(?P<minor>\d+)$"#;
    let pat = Regex::new(pat).unwrap();
    
    let mut lookup = DemoLookup::new();
    
    for subdir in subdirs(&repo) {
        let mut demos: Vec<(PathBuf, (u32, u32))> = subdirs(&subdir)
            .filter_map(|p| p.file_stem()
                .and_then(OsStr::to_str)
                .and_then(|s| pat.captures(s))
                .and_then(|cap| {
                    cap_parse::<u32>(&cap, "major")
                        .and_then(move |major| 
                            cap_parse::<u32>(&cap, "minor")
                                .map(move |minor| (major, minor)))
                })
                .map(move |num| (p, num)))
            .collect();
        demos.sort_by_key(|&(_, num)| num);
        
        let mut majors: Vec<u32> = demos.iter()
            .map(|&(_, (n, _))| n)
            .collect();
        majors.dedup();

        if majors.len() == 0 { continue; }
        if majors.len() > 1 {
            eprintln!("[WARN] several major versions detected in {:?}", subdir);
            continue;
        }
        let major = majors[0];
        if let Some(conflict) = lookup.get(&major) {
            eprintln!(
                "[WARN] conflicting major version {} between {:?} and {:?}",
                major, subdir, conflict.subdir_path);
            if !(demos.len() > conflict.demos.len()) {
                continue;
            }
        }
        
        lookup.insert(major, Subdir {
            subdir_path: subdir,
            demos: demos.into_iter()
                .map(|(path, (_, minor))| (minor, path))
                .collect(),
        });
    }
    
    lookup
}

struct Compiled {
    workdir: PathBuf,
    binary: PathBuf,
}

/// Compile code, get path to binary.
fn compile(lookup: &DemoLookup, major: u32, minor: u32) -> Result<Compiled, ()> {
    let subdir = lookup.get(&major)
        .ok_or_else(|| {
            eprintln!("[ERROR] major version {} not found", major);
            eprintln!("        available: {:?}", 
                lookup.keys().copied().collect::<Vec<u32>>());
        })?;
    let path = subdir.demos.get(&minor)
        .ok_or_else(|| {
            eprintln!("[ERROR] minor version {} not found in {:?}", 
                minor, subdir.subdir_path);
            eprintln!("        available: {:?}",
                subdir.demos.keys().copied().collect::<Vec<u32>>());
        })?;
    
    #[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
    #[allow(dead_code)]
    enum Compiler {
        ClangPp,
        // https://stackoverflow.com/questions/3178342/compiling-a-c-program-with-gcc#3206195
        Gcc9,
    }
    
    let compiler = Compiler::Gcc9;
    
    println!("[INFO] compiling with {:?}", compiler);
    println!();
    
    let status = match compiler {
        Compiler::ClangPp => Command::new("clang++")
            .args("-std=c++11 -stdlib=libc++ -w -O3".split_whitespace())
            .args(cpp_files(&path))
            .current_dir(&path)
            .status().unwrap(),
        Compiler::Gcc9 => Command::new("gcc-9")
            .args("-x c++ -fopenmp -w -O3 ".split_whitespace())
            .args(cpp_files(&path))
            .arg("-lstdc++")
            .current_dir(&path)
            .status().unwrap(),
    };
    
    if !status.success() {
        eprintln!();
        eprintln!("[ERROR] compile failure {}", status.code().unwrap());
        return Err(());
    }
    
    Ok(Compiled {
        workdir: path.clone(),
        binary: path.join("a.out")
    })
}

fn run_demo(lookup: &DemoLookup, major: u32, minor: u32) -> Result<(), ()> {
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

fn cpu_stat() {
    use std::fmt::{self, Display, Formatter};
    struct Indent<'a, I: Display>(&'a str, I);
    impl<'a, I: Display> Display for Indent<'a, I> {
        fn fmt(&self, f: &mut Formatter) -> fmt::Result {
            let string = format!("{}", self.1);
            let mut first = true;
            for line in string.lines() {
                if first {
                    first = false;
                } else {
                    f.write_str("\n")?;
                }
                f.write_str(self.0)?;
                f.write_str(line)?;
            }
            Ok(())
        }
    }
    
    println!("[INFO] cpu info:");
    println!("{}", Indent("       ", ""));
    println!("{}", Indent("       ", 
        format_args!("LOGICAL CPUS = {}", num_cpus::get())));
    println!("{}", Indent("       ", 
        format_args!("PHYSICAL CPUS = {}", num_cpus::get_physical())));
    println!("{}", Indent("       ", ""));
}

fn hw1(lookup: &DemoLookup, major: u32, minor: u32) -> Result<(), ()> {
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
            println!("[FAIL] exit code {}", status.code().unwrap());
            return Err(());
        }
    }
    
    println!("[INFO] done");
    
    Ok(())
} 

fn get_version(args: &[String]) -> (u32, u32) {
    assert_eq!(args.len(), 4, "unexpected num of args");

    let major: u32 = args[2].parse().unwrap();
    let minor: u32 = args[3].parse().unwrap();
    
    (major, minor)
}

fn reinstall() {
    println!("[INFO] recompiling cs39 cli");
    let _ = Command::new("cargo")
        .arg("install")
        .arg("--path")
        .arg(env!("CARGO_MANIFEST_DIR"))
        .arg("--force")
        .status();
    println!();
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
        "hw1" => {
            let (major, minor) = get_version(&args);
            let _ = hw1(&lookup, major, minor);
        },
        _ => {
            println!(include_str!("../manual.txt"));
        },
    }
}
