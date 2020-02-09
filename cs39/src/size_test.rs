
use crate::{
    cap_parse,
    navigate::{
        DemoLookup,
        find_demo,
    },
    compile::{
        cpp_files, 
        modify_compile,
        Compiled,
    },
    output::INFO_INDENT,
};
use std::{
    path::Path,
    process::Command,
    mem::replace,
    collections::HashMap,
    fs::read_to_string,
    ffi::OsString,
};
use regex::{self, Regex};
use byte_unit::Byte;

/// X or Y.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[repr(usize)]
pub enum Dim { X, Y }

/// Parse a `DIMX`/`DIMY` preprocessor directive.
pub fn parse_dim_line(dim: Dim, line: &str) -> Option<u128> {
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

/// Construct a `DIMX`/`DIMY` preprocessor directive.
pub fn format_dim_line(dim: Dim, val: u128) -> String {
    format!(
        r##"#define {}DIM {}"##,
        match dim {
            Dim::X => 'X',
            Dim::Y => 'Y',
        },
        val,
    )
}

/// Search for DIMX/DIMY values in a demo.
pub fn find_dims(
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

/// `size_test` task.
pub fn run<P>(
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
        
        let incr = 1;
        
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
    let mut dim_pretty = Vec::new();
    for &(x, y) in &dim_seq {
        let data_size = x * y * 4;
        let data_size_str = Byte::from_bytes(data_size)
            .get_appropriate_unit(true)
            .format(0);
        let dim_pretty_curr = format!("{}×{} = {}", x, y, data_size_str);
        println!("{} • {}", INFO_INDENT, dim_pretty_curr);
        dim_pretty.push(dim_pretty_curr);
    }
    
    for (i, &(x, y)) in dim_seq.iter().enumerate() {
        println!("[INFO] benchmarking dimension {}", &dim_pretty[i]);
        
        let Compiled { workdir, binary } = modify_compile(
            &repo, lookup, major, minor, 
            |code: &mut HashMap<OsString, String>| {
                for (file, content) in replace(code, HashMap::new()) {
                    let rewritten: String = content.lines()
                        .map(|line: &str| {
                            let mut line = line.to_owned();
                            if parse_dim_line(Dim::X, &line).is_some() {
                                line = format_dim_line(Dim::X, x);
                            } else if parse_dim_line(Dim::Y, &line).is_some() {
                                line = format_dim_line(Dim::Y, y);
                            }
                            line.push('\n');
                            line
                        })
                        .collect();
                    code.insert(file, rewritten);
                }
            })?;
            
        let status = Command::new(&binary)
            .current_dir(&workdir)
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
