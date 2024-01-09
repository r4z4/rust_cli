use chrono::{Duration, Datelike};
//walks a filesystem and finds duplicate files
use indicatif::{ParallelProgressIterator, ProgressStyle};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use walkdir::WalkDir;
use chrono::prelude::{DateTime, Utc};

pub fn walk(path: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let mut files = Vec::new();
    for entry in WalkDir::new(path) {
        let entry = entry?;
        if entry.file_type().is_file() {
            files.push(entry.path().to_str().unwrap().to_string());
        }
    }
    Ok(files)
}

pub fn filter_session_matched(matches: Vec<String>) -> Vec<String> {
    matches
}

//Find files matching a pattern
pub fn find(files: Vec<String>, pattern: &str) -> Vec<String> {
    let mut matches = Vec::new();
    for file in files {
        if file.contains(pattern) {
            matches.push(file);
        }
    }
    matches
}

/*  Parallel version of checksum using rayon with a mutex to ensure
 that the HashMap is not accessed by multiple threads at the same time
Uses indicatif to show a progress bar
*/
pub fn checksum(files: Vec<String>) -> Result<HashMap<String, Vec<String>>, Box<dyn Error>> {
    //set the progress bar style to allow for elapsed time and percentage complete
    let checksums = std::sync::Mutex::new(HashMap::new());
    let pb = indicatif::ProgressBar::new(files.len() as u64);
    let sty = ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
        .unwrap();
    pb.set_style(sty);
    files.par_iter().progress_with(pb).for_each(|file| {
        let checksum = md5::compute(std::fs::read(file).unwrap());
        let checksum = format!("{:x}", checksum);
        let mut checksums = checksums.lock().unwrap();
        checksums
            .entry(checksum)
            .or_insert_with(Vec::new)
            .push(file.to_string());
    });
    Ok(checksums.into_inner().unwrap())
}

pub fn get_last_tuesday() -> DateTime<Utc> {
    let now: DateTime<Utc> = Utc::now();
    let c:i64 = ((now.weekday().num_days_from_sunday() + 4) % 7 + 1) as i64;
    now - Duration::days(c)
}

/*  Parallel version of checksum using rayon with a mutex to ensure
 that the HashMap is not accessed by multiple threads at the same time
*/
pub fn file_times(files: Vec<String>) -> Result<HashMap<DateTime<Utc>, String>, Box<dyn Error>> {
    //set the progress bar style to allow for elapsed time and percentage complete
    let file_dates = std::sync::Mutex::new(HashMap::new());
    let pb = indicatif::ProgressBar::new(files.len() as u64);
    let sty = ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
        .unwrap();
    pb.set_style(sty);
    files.par_iter().progress_with(pb).for_each(|file| {
        let metadata = fs::metadata(file);
        match metadata {
            Ok(metadata) => {
                let modified =
                    if let Ok(time) = metadata.modified() {
                        Ok(time)
                    } else {
                        Err("Not supported on this platform")
                    };
                let modified_at = modified.unwrap();
                let mod_dt = systime_to_dt(&modified_at);
                // dbg!(&mod_dt);
                let mut file_dates = file_dates.lock().unwrap();
                file_dates.insert(mod_dt, file.to_string());
            },
            Err(_) => println!("Opps")
        }
    });
    Ok(file_dates.into_inner().unwrap())
}

/*
Find all the files with more than one entry in the HashMap
*/
pub fn find_duplicates(checksums: HashMap<String, Vec<String>>) -> Vec<Vec<String>> {
    let mut duplicates = Vec::new();
    for (_checksum, files) in checksums {
        if files.len() > 1 {
            duplicates.push(files);
        }
    }
    duplicates
}

pub fn systime_to_dt(st: &std::time::SystemTime) -> chrono::DateTime<Utc> {  
    let dt: DateTime<Utc> = st.clone().into();
    dt
    // format!("{}", dt.format("%+"))
    // formats like "2001-07-08T00:34:60.026490+09:30"
}

pub fn find_session_files(file_times: HashMap<DateTime<Utc>, String>, dt: DateTime<Utc>) -> Vec<String> {
    let mut session_files = Vec::new();
    for (file_time, file) in file_times {
        if file_time > dt {
            session_files.push(file);
        }
    }
    session_files
}

// invoke the actions along with the path and pattern and progress bar
pub fn start_session(flag: &str) -> Result<(), Box<dyn Error>> {
    let now: DateTime<Utc> = Utc::now();
    let mut sessions = HashMap::new();
    let name =
        match flag {
            "" => "_one",
            _ => flag
        };
    sessions.insert(name, now);
    Ok(())
}


// invoke the actions along with the path and pattern and progress bar
pub fn run_session(path: &str, pattern: &str, time: &String) -> Result<(), Box<dyn Error>> {
    // Just doing minutes for now
    let duration = Duration::minutes(time.parse::<i64>().unwrap());
    let now: DateTime<Utc> = Utc::now();
    let target = now - duration;
    let files = walk(path)?;
    let files = find(files, pattern);
    println!("Found {} files matching {}", files.len(), pattern);
    let file_times = file_times(files)?;
    let session_files = find_session_files(file_times, target);
    println!("Found {} session files(s)", session_files.len());
    for session in session_files {
        println!("{:?}", session);
    }
    Ok(())
}

// invoke the actions along with the path and pattern and progress bar
pub fn run(path: &str, pattern: &str) -> Result<(), Box<dyn Error>> {
    let files = walk(path)?;
    let files = find(files, pattern);
    println!("Found {} files matching {}", files.len(), pattern);
    let checksums = checksum(files)?;
    let duplicates = find_duplicates(checksums);
    println!("Found {} duplicate(s)", duplicates.len());
    for duplicate in duplicates {
        println!("{:?}", duplicate);
    }
    Ok(())
}