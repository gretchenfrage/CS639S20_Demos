
use crate::cap_parse;
use std::{
    path::{Path, PathBuf},
    collections::BTreeMap,
    fs::{
        read_dir,
        FileType,
    },
    ffi::OsStr,
};
use regex::Regex;

#[derive(Debug)] 
pub struct Subdir {
    pub subdir_path: PathBuf,
    pub demos: BTreeMap<u32, PathBuf>,
}

pub type DemoLookup = BTreeMap::<u32, Subdir>;

/// Read the demo directory structure.
pub fn demo_lookup<P: AsRef<Path>>(repo: P) -> DemoLookup {
    let pat = r#"^[_[[:alnum:]]]+_(?P<major>\d+)_(?P<minor>\d+)$"#;
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

/// Find a path for a demo by number.
pub fn find_demo(
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


/// List direct sub-**directories** of a directory.
///
/// Filters to not being with '.'.
pub fn subdirs<P: AsRef<Path>>(path: P) -> impl Iterator<Item=PathBuf> {
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
