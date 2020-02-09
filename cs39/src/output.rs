
use std::{
    fmt::{self, Display, Formatter},
    path::{Path, PathBuf},
    fs::{
        File,
        create_dir_all,
    },
    marker::PhantomData,
};
use csv::Writer as CsvWriter;
use serde::Serialize;

pub struct Indent<'a, I: Display>(pub &'a str, pub I);

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

pub const INFO_INDENT: &'static str = "       ";

/// Allocate a path for a CSV file.
pub fn csv_path<S>(name: S) -> PathBuf 
where
    S: AsRef<str> 
{
    // unpleasent redundancy, but whatevs.
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .canonicalize().unwrap()
        .parent().map(PathBuf::from).unwrap();
    
    let host = repo.join("output");
    create_dir_all(&host).unwrap();
    
    host.join(name.as_ref())
}

/// Output data table writer.
pub struct TableWriter<T: Serialize> {
    target: TableTarget,
    
    p: PhantomData<fn(T)>,
}

enum TableTarget {
    None,
    Csv(CsvWriter<File>),
}

impl<T: Serialize> TableWriter<T> {
    fn from(target: TableTarget) -> Self {
        TableWriter {
            target,
            p: PhantomData,
        }
    }
    
    pub fn none() -> Self {
        Self::from(TableTarget::None)
    }
    
    pub fn csv_file<P>(path: P) -> Self 
    where 
        P: AsRef<Path>
    {
        println!("[INFO] writing csv file to {:?}", path.as_ref());
        let csv = CsvWriter::from_path(&path).unwrap();
        Self::from(TableTarget::Csv(csv))
    }
    
    pub fn write(&mut self, row: T) {
        match &mut self.target {
            &mut TableTarget::None => (),
            &mut TableTarget::Csv(ref mut csv) => {
                csv.serialize(row).unwrap();
            },
        };
    }
}

/*

pub enum TableOutput<R: Serialize> {
    None,
    Csv(csv::Writer<File>),
}

impl<R: Serialize> TableOutput<R> {
    
}
*/