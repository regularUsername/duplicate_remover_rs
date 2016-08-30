#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate clap;
extern crate twox_hash;
extern crate pbr;
extern crate walkdir;


use std::path::{Path, PathBuf};
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
#[cfg(windows)]
use std::os::windows::fs::MetadataExt;

use std::collections::HashMap;
use std::fs::{File, create_dir_all, rename, read_dir};
use std::io::{BufReader, BufRead,stdin, stdout,Write};
use std::hash::Hasher;
use std::time::SystemTime;
use regex::Regex;
use clap::{Arg, App};
use twox_hash::XxHash;
use pbr::ProgressBar;
use walkdir::WalkDir;

lazy_static! {
    static ref IS_NUMERIC:Regex = Regex::new(r"^[:digit:]+$").unwrap();
    static ref IS_HEX:Regex = Regex::new(r"^[:xdigit:]+$").unwrap();
    static ref IS_ALNUM:Regex = Regex::new(r"^[:alnum:]+$").unwrap();
}

// TODO zusatzmodus: wenn rekursiv nur duplikate entfernen wenn beide im selbem ordner sind
macro_rules! println_stderr(
    ($($arg:tt)*) => { {
        let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
        r.expect("failed printing to stderr");
    } }
);

#[cfg(unix)]
macro_rules! get_size {
    ($x:expr) => ($x.size())
}

#[cfg(windows)]
macro_rules! get_size {
    ($x:expr) => ($x.file_size())
}

macro_rules! file_stem {
    ($x:expr) => ($x.file_stem().unwrap_or(std::ffi::OsStr::new("decoding error")).to_str().unwrap_or("decoding error"))
}

macro_rules! file_name {
    ($x:expr) => ($x.file_name().unwrap_or(std::ffi::OsStr::new("decoding error")).to_str().unwrap_or("decoding error"))
}

enum Foobar<'a> {
    Ok(Vec<&'a PathBuf>),
    Cancel,
    Invalid,
}

fn main() {
    let matches = App::new("Chan Dupe Finder")
        .version("0.1")
        .about("Finds and removes duplicate files (prioritizing the best name)")
        .arg(Arg::with_name("recursive")
            .help("Searches duplicates in subdirectories")
            .short("r")
            .long("recursive"))
        .arg(Arg::with_name("directory")
            .required(true)
            .index(1))
        .get_matches();

    let directory = matches.value_of("directory").unwrap();
    let dir = Path::new(directory);
    if !dir.is_dir() {
        println_stderr!("Not valid Directory");
        return;
    }
     do_stuff(dir, matches.is_present("recursive"));
}

// TODO diesen fall erkennen "tmp_3336-28f644e8585f21d51362c3b847580546123518181.png" vs "1239833_-_Jigglybutts_Mew_Porkyman.png"
fn select_files<'a>(files: &[&'a PathBuf]) -> (&'a PathBuf, Vec<&'a PathBuf>) {
    let mut tmp = Vec::from(files);
    for x in files {
        for y in files {
            let x_name = file_stem!(x);
            let y_name = file_stem!(y);
            if x_name != y_name && x_name.starts_with(y_name) {
                if x_name.len() - y_name.len() <= 5 {
                    tmp.retain(|e| e != x)
                } else {
                    tmp.retain(|e| e != y)
                }
            }
        }
    }

    let mut bestname = tmp[0];
    let mut bestname_prio = 0;
    for x in &tmp {
        let n = file_stem!(x);
        let current_n = file_stem!(bestname);
        if IS_NUMERIC.is_match(n) {
            if bestname_prio < 1 || (bestname_prio == 1 && n.len() > current_n.len()) {
                bestname = x;
                bestname_prio = 1;
            }
        } else if IS_HEX.is_match(n) {
            if bestname_prio < 2 || (bestname_prio == 2 && n.len() > current_n.len()) {
                bestname = x;
                bestname_prio = 2;
            }
        } else if IS_ALNUM.is_match(n) {
            if bestname_prio < 3 || (bestname_prio == 3 && n.len() > current_n.len()) {
                bestname = x;
                bestname_prio = 3;
            }
        } else if bestname_prio < 4 || (bestname_prio == 4 && n.len() > current_n.len()) {
            bestname = x;
            bestname_prio = 4;
        }
    }
    let mut tmp = Vec::from(files);
    tmp.retain(|e| e != &bestname);

    // println!("keep: {:?}",bestname);
    // println!("delete: {:?}\n",tmp);
    (bestname, tmp)
}

fn backup_file(fp: &Path, basedir: &Path) -> Result<(), String> {
    let backupdir = basedir.join(Path::new("duplicates"));
    let p_bak = backupdir.join(match fp.strip_prefix(basedir) {
        Ok(v) => v,
        Err(e) => return Err(format!("convert absolute to relative path: {}", e)),
    });

    // println!("{:?} -> {:?}", fp, p_bak);
    if let Err(e) =  create_dir_all(
        match p_bak.parent() {
            Some(v) => v,
            None => return Err("Something happened".to_string()),
        })
    {
        return Err(format!("Create Backup Dir: {}", e));
    };

    if let Err(e) = rename(fp, &p_bak) {
        return Err(format!("Move file: {}", e));
    };

    Ok(())
}

fn select_action(dups: HashMap<u64,Vec<&PathBuf>>,dir: &Path) {
    loop {
        print!("remove all duplicates?(backup in {:?}) ([y]es/[i]nteractive mode/[n]o/[p]rint): ",dir.join("duplicates"));
        stdout().flush().unwrap();
        let mut buf = String::new();
        stdin().read_line(&mut buf).unwrap();
        let buf = buf.trim().to_lowercase();

        if buf.starts_with('y') {
            for entry in dups.values().filter(|x| x.len() > 1) {
                let (_, remove) = select_files(entry);
                for r in remove {
                    if let Err(e) = backup_file(r, dir) {
                        println_stderr!("{:?}: {}",r,e);
                    }
                }
            }
            break;
        } else if buf.starts_with('i') {
            println!("Interactive Mode:");
            for entry in dups.values().filter(|x| x.len() > 1) {
                let (keep, remove) = select_files(entry);
                loop {
                    match interactive_selection(keep,&remove){
                        Foobar::Ok(l) => {
                            for i in l {
                                if let Err(e) = backup_file(i, dir) {
                                    println_stderr!("{:?}: {}",i,e);
                                }
                            }
                            break;
                        },
                        Foobar::Cancel => {
                            println!("Cancel");
                            return;
                            },
                        Foobar::Invalid => println!("invalid input"),
                    };
                }
                
            }
            break;
        } else if buf.starts_with('p') {
            for entry in dups.values().filter(|x| x.len() > 1) {
                let (keep, remove) = select_files(entry);
                println!("keep: {:?}",
                         keep.file_name().unwrap_or(std::ffi::OsStr::new("decoding error")));
                for r in remove {
                    println!("delete: {:?}",
                             r.file_name().unwrap_or(std::ffi::OsStr::new("decoding error")));
                }
                println!("");
            }

        } else if buf.starts_with('n') {
            return;
        } else {
            println!("invalid input");
        }
    }
}

fn interactive_selection<'a>(k:&'a PathBuf,r: &[&'a PathBuf]) -> Foobar<'a> {
    let mut tmp = Vec::with_capacity(r.len()+1);
    tmp.push(k);
    tmp.append(&mut Vec::from(r));

    for (i,v) in tmp.iter().enumerate(){
        if i == 0 {
            println!("* ({}): {}",i+1,file_name!(v));
        } else {
            println!("  ({}): {}",i+1,file_name!(v));
        }
    }
    println!("([c]ancel/Return for default [1])");
    print!("select file to keep: ");
    stdout().flush().unwrap();
    let mut buf = String::new();
    stdin().read_line(&mut buf).unwrap();
    let buf = buf.trim().to_lowercase();


    if IS_NUMERIC.is_match(&buf) {
        let sel: usize = match buf.parse(){
            Ok(v) => v,
            _ => return Foobar::Invalid,
        };
        if sel > tmp.len() {
            return Foobar::Invalid;
        }
        tmp.remove(sel - 1);
    } else if buf.is_empty() {
        tmp.remove(0);
    } else if buf.starts_with('c') {
        return Foobar::Cancel;
    } else {
        return Foobar::Invalid;
    }
    println!("delete: {:?}\n",tmp);

    Foobar::Ok(tmp)
}

fn do_stuff(dir: &Path, recursive: bool) {
    let mut dup_count = 0;
    let t = SystemTime::now();
    println!("Pass1...");
    // pass1
    let mut pass1_files = HashMap::new();

    if recursive {
        for entry in WalkDir::new(&dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let meta = entry.metadata().unwrap();
            if meta.file_type().is_file(){
                let size = get_size!(meta);
                pass1_files.entry(size).or_insert_with(Vec::new).push(entry.path().to_owned());
            }
        }
    } else {
        for entry in read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
        {
            let meta = entry.metadata().unwrap();
            if meta.file_type().is_file(){
                let size = get_size!(meta);
                pass1_files.entry(size).or_insert_with(Vec::new).push(entry.path().to_owned());
            }
        }
    }


    // let pass1_vec: Vec<_> = pass1_files.iter().filter(|&(_,y)| y.len() > 1).map(|(x,y)|y).collect();
    println!("Pass2: Hashing Files");
    // pass2
    let mut pass2_files = HashMap::new();
    let mut pb = ProgressBar::new(pass1_files.values().filter(|x| x.len() > 1).flat_map(|v|v.iter()).count() as u64);
    pb.format("8=D~D");

    for entry in pass1_files.values().filter(|x| x.len() > 1).flat_map(|v|v.iter()) {
        let hash = hash_file(entry);

        let mut list = pass2_files.entry(hash).or_insert_with(Vec::new);
        if !list.is_empty() {
            dup_count += 1;
        }
        list.push(entry);

        pb.inc();
    }
    pb.finish();

    println!("\nPass2 finished");
    let dt = t.elapsed().unwrap();
    println!("Time elapsed: {}.{}s",
             dt.as_secs(),
             (dt.subsec_nanos() / 1000 / 1000) as u64);

    println!("{} duplicates founds", dup_count);

    if dup_count == 0 {
        return
    }

    select_action(pass2_files,dir);
}

fn hash_file(path: &Path) -> u64 {
    let mut hasher = XxHash::with_seed(0);
    let fd = File::open(path).unwrap();
    let mut br = BufReader::new(&fd);
    loop {
        let buf_size = {
            let buf = br.fill_buf().unwrap();
            if buf.is_empty() {
                break;
            } else {
                hasher.write(buf);
            }
            buf.len()
        };
        br.consume(buf_size);
    }
    hasher.finish()
}

fn bytes_to_si(size: usize) -> String {
    let suffix = ["B", "KiB", "MiB", "GiB", "TiB"];
    if size == 0 {
        "0 B".to_string()
    } else {
        let mut px = (size as f64).log(1024.0) as usize;
        if px > suffix.len() - 1 {
            px = suffix.len() - 1;
        }
        format!("{:.2} {}",
                (size as f64) / 1024_f64.powi(px as i32), suffix[px])
    }
}