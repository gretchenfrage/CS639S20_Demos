
use crate::{
    cap_parse,
};
use std::{
    process::{
        Command,
        Stdio,
        ExitStatus,
    },
    sync::mpsc,
    thread,
    io::{Read, BufRead, BufReader},
    borrow::BorrowMut,
    time::Duration,
};
use regex::Regex;

pub fn parse_elapsed_time_line(line: &str) -> Option<Duration> {
    let pat = r##"^Running test iteration\s+\d+\s+\[Elapsed time : (?P<ms>\d+(?:\.\d+)?)ms\]$"##;
    let pat = Regex::new(pat).unwrap();
        
    pat
        .captures(line)
        .map(|caps| cap_parse::<f64>(&caps, "ms").unwrap())
        .map(|ms| Duration::from_secs_f64(ms * 1000.0))
}

pub fn demo_min_time<I, L>(lines: I) -> Duration 
where
    I: IntoIterator<Item=L>,
    L: AsRef<str>,
{
    lines.into_iter()
        .flat_map(|line| parse_elapsed_time_line(line.as_ref()))
        .min()
        .unwrap()
}

/// Spawn a sub-process, and by the power of threads,
/// elevate its stdout and stderr to the parent while
/// also merging them together into a line stream,
/// then collecting them.
pub fn subproc<B>(mut command: B) -> (ExitStatus, Vec<String>) 
where
    B: BorrowMut<Command>,
{
    let cmd = command.borrow_mut();
    
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    
    let (send_0, recv) = mpsc::sync_channel::<String>(10000);
    let send_1 = send_0.clone();
    
    let mut child = cmd.spawn().unwrap();
    
    let stdout = child.stdout.take().unwrap();
    let stdout = Box::new(stdout) as Box<dyn Read + Send>;
    
    let stderr = child.stderr.take().unwrap();
    let stderr = Box::new(stderr) as Box<dyn Read + Send>;
    
    let mut threads = Vec::new();
    for (read, send) in vec![
        (stdout, send_0),
        (stderr, send_1),
    ] {
        let thread = thread::spawn(move || {
            let read = BufReader::new(read);
            for line in read.lines() {
                let line = line.unwrap();
                println!("{}", line);
                let _ = send.send(line);
            }
        });
        threads.push(thread);
    }
    
    let status = child.wait().unwrap();
    
    for thread in threads {
        thread.join().unwrap();
    }
    
    let mut lines = Vec::new();
    while let Ok(line) = recv.try_recv() {
        lines.push(line);
    }
    
    (status, lines)
}