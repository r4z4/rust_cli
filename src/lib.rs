//walks a filesystem and finds duplicate files
use indicatif::{ParallelProgressIterator, ProgressStyle};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{collections::HashMap, time::SystemTime};
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
pub fn find(files: Vec<String>, pattern: &str, dt: Option<&DateTime<Utc>>) -> Vec<String> {
    let mut matches = Vec::new();
    for file in files {
        if file.contains(pattern) {
            matches.push(file);
        }
    }
    if dt.is_some() {
        filter_session_matched(matches)
    } else {
        matches
    }
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

/*  Parallel version of checksum using rayon with a mutex to ensure
 that the HashMap is not accessed by multiple threads at the same time
*/
pub fn file_times(files: Vec<String>) -> Result<HashMap<DateTime<Utc>, Vec<String>>, Box<dyn Error>> {
    //set the progress bar style to allow for elapsed time and percentage complete
    let file_dates = std::sync::Mutex::new(HashMap::new());
    let pb = indicatif::ProgressBar::new(files.len() as u64);
    let sty = ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
        .unwrap();
    pb.set_style(sty);
    files.par_iter().progress_with(pb).for_each(|file| {
        let metadata = fs::metadata("foo.txt");
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
                let mut file_dates = file_dates.lock().unwrap();
                file_dates
                    .entry(mod_dt)
                    .or_insert_with(Vec::new)
                    .push(file.to_string());
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

pub fn find_session_files(file_times: HashMap<DateTime<Utc>, Vec<String>>, dt: DateTime<Utc>) -> Vec<Vec<String>> {
    let mut session_files = Vec::new();
    for (file_time, files) in file_times {
        if files.len() > 1 {
            session_files.push(files);
        }
    }
    session_files
}

// invoke the actions along with the path and pattern and progress bar
pub fn run_session(path: &str, pattern: &str, now: &SystemTime) -> Result<(), Box<dyn Error>> {
    let dt = systime_to_dt(now);
    let files = walk(path)?;
    let files = find(files, pattern, Some(&dt));
    println!("Found {} files matching {}", files.len(), pattern);
    let file_times = file_times(files)?;
    let session_files = find_session_files(file_times, dt);
    println!("Found {} session files(s)", session_files.len());
    for session in session_files {
        println!("{:?}", session);
    }
    Ok(())
}

// invoke the actions along with the path and pattern and progress bar
pub fn run(path: &str, pattern: &str) -> Result<(), Box<dyn Error>> {
    let files = walk(path)?;
    let files = find(files, pattern, None);
    println!("Found {} files matching {}", files.len(), pattern);
    let checksums = checksum(files)?;
    let duplicates = find_duplicates(checksums);
    println!("Found {} duplicate(s)", duplicates.len());
    for duplicate in duplicates {
        println!("{:?}", duplicate);
    }
    Ok(())
}