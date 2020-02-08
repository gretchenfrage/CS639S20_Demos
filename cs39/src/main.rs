extern crate regex;
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

fn run_demo(lookup: &DemoLookup, major: u32, minor: u32) -> Result<(), ()> {
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
    
    println!("[INFO] compiling");
    println!();
    let status = Command::new("clang++")
        .args("-std=c++11 -stdlib=libc++ -w -O3".split_whitespace())
        .args(cpp_files(&path))
        .current_dir(&path)
        .status().unwrap();
    if !status.success() {
        eprintln!();
        eprintln!("[ERROR] compile failure {}", status.code().unwrap());
        return Err(());
    }
    
    println!("[INFO] running");
    println!();
    let status = Command::new(path.join("a.out"))
        .current_dir(&path)
        .status().unwrap();
    println!();
    println!("[INFO] exit {}", status.code().unwrap());
    
    Ok(())
}

fn main() {
    let args: Vec<String> = args().collect();
    
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .canonicalize().unwrap()
        .parent().map(PathBuf::from).unwrap();
        
    let lookup = demo_lookup(&repo);
    
    if let Some("--help") = args.get(1).map(String::as_str) {
        println!(include_str!("../manual.txt"));
    } else if let Some("--list") = args.get(1).map(String::as_str) {
        println!("[INFO] listing demos");
        println!("{:#?}", lookup);
    } else {
        assert_eq!(args.len(), 3, "unexpected num of args");

        let major: u32 = args[1].parse().unwrap();
        let minor: u32 = args[2].parse().unwrap();
    
        let _ = run_demo(&lookup, major, minor);
    }
}
