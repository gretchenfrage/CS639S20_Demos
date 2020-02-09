extern crate regex;
extern crate num_cpus;
extern crate rand;
extern crate byte_unit;
use std::{
    env::args,
    str::FromStr,
    path::{PathBuf, Path},
    collections::{BTreeMap, HashMap},
    fs::{
        self,
        read_dir, 
        read_to_string, 
        create_dir_all,
        FileType,
    },
    ffi::{OsStr, OsString},
    process::{Command, ExitStatus},
};
use regex::{Regex};
use rand::prelude::*;
use byte_unit::{Byte, ByteUnit};

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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[allow(dead_code)]
enum Compiler {
    ClangPp,
    // https://stackoverflow.com/questions/3178342/compiling-a-c-program-with-gcc#3206195
    Gcc9,
}

impl Compiler {
    fn compile<P>(self, path: P) -> ExitStatus 
    where
        P: AsRef<Path>,
    {
        match self {
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
        }
    }
}

fn find_demo(
    lookup: &DemoLookup, 
    major: u32, 
    minor: u32
) -> Result<PathBuf, ()> {
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
    Ok(path.clone())
}

/// Read code to memory, modify, write to temp dir, compile, get path
/// to binary.
fn modify_compile<P, F>(
    repo: P,
    lookup: &DemoLookup, 
    major: u32,
    minor: u32, 
    edit: F
) -> Result<Compiled, ()> 
where
    P: AsRef<Path>,
    F: FnOnce(&mut HashMap<OsString, String>),
{
    // find code
    let path = find_demo(lookup, major, minor)?;
    
    // read code
    let mut code: HashMap<OsString, String> = cpp_files(&path)
        .map(|file| (
            file.clone(), 
            read_to_string(path.join(file)).unwrap()
        ))
        .collect();
        
    // code modification callback
    edit(&mut code);
    
    // allocate temp directory
    let temp = repo.as_ref().join("tmp").join(format!("rng-{}", random::<u16>()));
    println!("[INFO] building code in {:?}", temp);
    create_dir_all(&temp).unwrap();
    
    // save code
    for (file, content) in code {
        let path = temp.join(file);
        fs::write(path, content).unwrap();
    }
    
    let compiler = Compiler::Gcc9;
    
    println!("[INFO] compiling with {:?}", compiler);
    println!();
    
    let status = compiler.compile(&temp);
    if !status.success() {
        eprintln!();
        eprintln!("[ERROR] compile failure {}", status.code().unwrap());
        return Err(());
    }
    
    // done
    Ok(Compiled {
        workdir: temp.clone(),
        binary: temp.join("a.out")
    })
}

/// Compile code, get path to binary.
fn compile(lookup: &DemoLookup, major: u32, minor: u32) -> Result<Compiled, ()> {
    let path = find_demo(lookup, major, minor)?;
    
    let compiler = Compiler::Gcc9;
    
    println!("[INFO] compiling with {:?}", compiler);
    println!();
    
    let status = compiler.compile(&path);

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
const INFO_INDENT: &'static str = "       ";

fn cpu_stat() {
    println!("[INFO] cpu info:");
    println!("{}", Indent(INFO_INDENT, ""));
    println!("{}", Indent(INFO_INDENT, 
        format_args!("LOGICAL CPUS = {}", num_cpus::get())));
    println!("{}", Indent(INFO_INDENT,
        format_args!("PHYSICAL CPUS = {}", num_cpus::get_physical())));
    println!("{}", Indent(INFO_INDENT, ""));
}

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

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[repr(usize)]
enum Dim { X, Y }

fn parse_dim_line(dim: Dim, line: &str) -> Option<u128> {
    let pat = format!(
        r#"^{}[[:space:]]+(?P<n>\d+)[[:space:]]*$"#,
        regex::escape(&format!(
            r##"#define {}DIM"##,
            match dim {
                Dim::X => 'X',
                Dim::Y => 'Y',
            },
        )),
    );
    let pat = Regex::new(&pat).unwrap();
    
    pat.captures(line)
        .map(|caps| cap_parse::<u128>(&caps, "n").unwrap())
}

fn format_dim_line(dim: Dim, val: u32) -> String {
    format!(
        r##"#define {}DIM {}"##,
        match dim {
            Dim::X => 'X',
            Dim::Y => 'Y',
        },
        val,
    )
}

fn find_dims(
    lookup: &DemoLookup, 
    major: u32,
    minor: u32
) -> Result<(u128, u128), ()> {
    let path = find_demo(lookup, major, minor)?;
    let mut found: [Option<u128>; 2] = [None, None];
    for file in cpp_files(&path) {
        let code = read_to_string(path.join(file)).unwrap();
        for line in code.lines() {
            for &dim in &[Dim::X, Dim::Y] {
                if let Some(val) = parse_dim_line(dim, line) {
                    if found[dim as usize].is_some() {
                        println!("[ERROR] dimension {:?} defined twice in code", dim);
                        return Err(());
                    } else {
                        found[dim as usize] = Some(val);
                    }
                }
            }
        }
    }
    for &dim in &[Dim::X, Dim::Y] {
        if found[dim as usize].is_none() {
            println!("[ERROR] dimension {:?} not found in code", dim);
            return Err(());
        }
    }
    Ok((found[0].unwrap(), found[1].unwrap()))
}

fn size_test<P>(
    repo: P, 
    lookup: &DemoLookup, 
    major: u32, 
    minor: u32
) -> Result<(), ()> 
where
    P: AsRef<Path> 
{
    // find the default dimensions
    let (base_x, base_y) = find_dims(lookup, major, minor)?;
    println!("[INFO] default dimensions are {:?}", (base_x, base_y));
    
    let dim_seq: Vec<(u128, u128)> = {
        let mut vec = Vec::new();
        
        let base = base_x * base_y;
        let mut curr = base;
        
        let incr = 2;
        
        for _ in 0..10 {
            if (curr >> incr) >= (1 << 12) {
                curr >>= incr;
            } else {
                break;
            }
        }
        
        loop {
            vec.push(curr);
            if (curr << incr) > (base << 12) {
                break;
            } else if (curr << incr) > ((1 << 30) * 4 / 4) {
                break;
            } else {
                curr <<= incr;
            }
        }
        
        fn u128sqrt(n: u128) -> u128 {
            (n as f64).sqrt() as u128
        }
        
        vec
            .into_iter()
            .map(|s| {
                let y = u128sqrt(base_y * s / base_x);
                let mut x = s / y;
                x += s % (x * y);
                (x, y)
            })
            .collect()
    };
    
    println!("[INFO] testing with dimensions:");
    for (x, y) in dim_seq {
        let data_size = x * y * 4;
        let data_size_str = Byte::from_bytes(data_size)
            .get_appropriate_unit(true)
            .format(0);
        println!("{} • {}×{} = {}", INFO_INDENT, x, y, data_size_str);
    }
    
    Ok(())
    
    /*
    // x and y size
    let (base_x, base_y) = {
        println!("[INFO] looking for default XDIM, YDIM");
        
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
            
        let header_file = path.join("Laplacian.h");
        let header_content = read_to_string(&header_file)
            .unwrap();
            
        let pat_x = format!(
            r#"^{}[[:space:]]+(?P<n>\d+)[[:space:]]*$"#,
            escape(r##"#define XDIM"##),
        );
        let pat_x = Regex::new(pat_x).unwrap();
        
        let pat_y = format!(
            r#"^{}[[:space:]]+(?P<n>\d+)[[:space:]]*$"#,
            escape(r##"#define YDIM"##),
        );
        let pat_y = Regex::new(pat_y).unwrap();
        
        let mut found_x: Option<u128> = None;
        let mut found_y: Option<u128> = None;
        
        for line in header_content.lines {
            if let Some(x) = cap_parse(&pat_x.capture(line), "n") {
                if found_x.is_some() {
                    println!("[]")
                }
            }
            if let Some(y) = cap_parse(&pat_y.capture(line), "n") {
                
            }
        }
        
    };
    */
}

fn get_version(args: &[String]) -> (u32, u32) {
    assert!(args.len() >= 4, "unexpected num of args");

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
        "cpu_test" => {
            let (major, minor) = get_version(&args);
            let _ = cpu_test(&lookup, major, minor);
        },
        "size_test" => {
            let (major, minor) = get_version(&args);
            let _ = size_test(&repo, &lookup, major, minor);
        }
        _ => {
            println!(include_str!("../manual.txt"));
        },
    }
}
