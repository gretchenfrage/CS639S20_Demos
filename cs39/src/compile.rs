
use crate::navigate::{find_demo, DemoLookup};
use std::{
    path::{Path, PathBuf},
    ffi::{OsStr, OsString},
    fs::{
        self,
        read_to_string,
        read_dir,
        create_dir_all,
        FileType
    },
    collections::HashMap,
    process::{
        Command,
        ExitStatus,
    }
};
use rand::prelude::*;

/// Result of code compilation.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Compiled {
    pub workdir: PathBuf,
    pub binary: PathBuf,
}

/// List names of C++ source code files directly in a directory.
pub fn cpp_files<P: AsRef<Path>>(path: P) -> impl Iterator<Item=OsString> {
    read_dir(path).unwrap()
        .filter_map(Result::ok)
        .filter_map(|f| f.file_type().ok()
            .filter(FileType::is_file)
            .map(|_| f.path()))
        .filter_map(|p| p.extension()
            .filter(|e| *e == "cpp" || *e == "h")
            .and_then(|_| p.file_name().map(OsStr::to_owned)))
}

/// Possible C++ compiler toolchain to invoke.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[allow(dead_code)]
pub enum Compiler {
    ClangPp,
    // https://stackoverflow.com/questions/3178342/compiling-a-c-program-with-gcc#3206195
    Gcc9,
    // this seems to be the default installed in my arch linux
    Gcc,
}

/// Try to compile code in a subdirectory.
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
            Compiler::Gcc => Command::new("gcc")
                .args("-x c++ -fopenmp -w -O3 ".split_whitespace())
                .args(cpp_files(&path))
                .arg("-lstdc++")
                .current_dir(&path)
                .status().unwrap(),
        }
    }
}

/// Compile code, get path to binary.
pub fn compile(lookup: &DemoLookup, major: u32, minor: u32) -> Result<Compiled, ()> {
    let path = find_demo(lookup, major, minor)?;
    
    let compiler = Compiler::Gcc;
    
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

/// Read code to memory, modify, write to temp dir, compile, get path
/// to binary.
pub fn modify_compile<P, F>(
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
