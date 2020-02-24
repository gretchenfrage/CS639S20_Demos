use crate::{
    {cap_parse, cpu_stat},
    navigate::{
        DemoLookup,
    },
    compile::{
        compile,
        Compiled,
    },
    quant::{
        subproc,
    },
};
use std::{
    process::Command,
    collections::HashMap,
    time::Duration,
};
use regex::{self, Regex};
use num_cpus;

/// `kernel_sum_test` task.
pub fn run(
    lookup: &DemoLookup, 
    major: u32, 
    minor: u32,
    multithreaded: bool,
) -> Result<(), ()> 
{
    println!("[INFO] running kernel time sum test on demo {:?}", (major, minor));
    println!("[INFO] multithreading = {}", multithreaded);
    cpu_stat();
    let cpu = match multithreaded {
        true => num_cpus::get(),
        false => 1,
    };
    
    let Compiled { workdir, binary } = compile(lookup, major, minor)?;
    
    println!("[INFO] running");
    let (status, lines) = subproc(
        Command::new(&binary)
            .current_dir(&workdir)
            .env("OMP_NUM_THREADS", cpu.to_string()), true);
            
    if !status.success() {
        println!("[ERROR] exit code {}", status.code().unwrap());
        return Err(());
    }   
    
    let mut sums: HashMap<String, Duration> = HashMap::new();
    let mut total: Option<Duration> = None;
    
    for line in lines {
        if let Some((name, time)) = parse_kernel_run_line(&line) {
            *sums.entry(name).or_insert(Duration::from_secs(0)) += time;
        } else if let Some(time) = parse_entire_run_line(&line) {
            if total.is_some() {
                println!("[WARN] \"Entire Run\" line occurred in duplicate");
                println!("       Old time = {:?}", total.unwrap());
                println!("       New time = {:?}", time);
            }
            
            total = Some(time);
        }
    }
    
    let total = total
        .ok_or_else(|| {
            println!("[ERROR] program did not report \"Entire Run\" time");
        })?;
    
    let grand_sum: Duration = sums.iter()
        .map(|(_, &time)| time)
        .sum();
        
    let unaccounted = total.checked_sub(grand_sum);
    
    println!();
    println!("[INFO] displaying kernel times:");
    println!();
    {
        let mut times: Vec<(&str, Duration)> = sums.iter().map(|(s, &d)| (s.as_str(), d)).collect();
        times.sort_by_key(|&(_, d)| -(d.as_millis() as i128));
        let max_len = times.iter().map(|&(s, _)| s.len()).max().unwrap();
        
        let max_millis: u128 = times.iter().map(|&(_, d)| d.as_millis()).max().unwrap();
        let max_bars = 40;
        
        for &(s, d) in &times {
            print!("       ");
            print!("{}", s);
            for _ in 0..(max_len - s.len() + 1) {
                print!(" ");
            }
            print!("{:.2}s", d.as_secs_f32());
            
            print!(" ");
            let bars: u32 = ((d.as_millis() as f64 / max_millis as f64) * max_bars as f64) as u32;
            for _ in 0..bars {
                print!("=");
            }
            println!();
        }
    }
    
    println!();
    println!("[INFO] program-reported total time = {:.3}s", total.as_secs_f32());
    println!("[INFO] sum of kernel times = {:.3}s", grand_sum.as_secs_f32());
    if let Some(unaccounted) = unaccounted {
        println!("[INFO] unaccounted time = {:.3}s", unaccounted.as_secs_f32());
    } else {
        println!("[INFO] unaccounted time = None");
    }
    println!();
    println!("[INFO] done");
    Ok(())
}

fn parse_kernel_run_line(line: &str) -> Option<(String, Duration)> {
    let pat = r##"^\[KERNEL (?P<name>.+) : Time = (?P<ms>\d+(?:\.\d+)?)ms\]$"##;
    let pat = Regex::new(pat).unwrap();
    pat
        .captures(line)
        .map(|caps| {
            let ms = cap_parse::<f64>(&caps, "ms").unwrap();
            let time = Duration::from_secs_f64(ms / 1000.0);
            
            let name = cap_parse::<String>(&caps, "name").unwrap();
            
            (name, time)
        })
}

fn parse_entire_run_line(line: &str) -> Option<Duration> {
    let pat = r##"^\[Entire Run : (?P<ms>\d+(?:\.\d+)?)ms\]$"##;
    let pat = Regex::new(pat).unwrap();
    pat
        .captures(line)
        .map(|caps| cap_parse::<f64>(&caps, "ms").unwrap())
        .map(|ms| Duration::from_secs_f64(ms / 1000.0))
}