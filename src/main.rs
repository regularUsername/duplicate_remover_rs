#![feature(test)]
#[macro_use]
extern crate lazy_static;
extern crate clap;
extern crate indicatif;
extern crate regex;
extern crate twox_hash;

#[cfg(test)]
mod tests;

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
#[cfg(windows)]
use std::os::windows::fs::MetadataExt;

use clap::{App, Arg};
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::fs::{create_dir_all, read_dir, remove_file, rename, File};
use std::hash::BuildHasher;
use std::hash::{BuildHasherDefault, Hasher};
use std::io::{stdin, stdout, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use twox_hash::XxHash;

lazy_static! {
    static ref IS_NUMERIC: Regex = Regex::new("^[[:digit:]]+$").unwrap();
    static ref IS_HEX: Regex = Regex::new("^[[:xdigit:]]+$").unwrap();
    static ref IS_ALNUM: Regex = Regex::new("^[[:alnum:]]+$").unwrap();
    static ref RE_WORDS: Regex = Regex::new("[[:alnum:]]{2,}").unwrap();
}

// TODO idee für zusatzmodus: wenn rekursiv nur duplikate entfernen wenn beide im selbem ordner sind
macro_rules! println_stderr(
    ($($arg:tt)*) => { {
        let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
        r.expect("failed printing to stderr");
    } }
);

// metadata um eine platformunabhängige methode 
// zum abfragen der dateigröße erweitern
 trait GetFileSize{
     fn get_file_size(&self)-> u64;
 }

impl GetFileSize for std::fs::Metadata {
    #[cfg(windows)]
    fn get_file_size(&self)-> u64 {
        self.file_size()
    }
    #[cfg(unix)]
    fn get_file_size(&self)-> u64 {
        self.size()
    }
}

macro_rules! file_stem {
    ($x:expr) => {
        $x.file_stem()
            .unwrap_or(std::ffi::OsStr::new("decoding error"))
            .to_str()
            .unwrap_or("decoding error")
    };
}

macro_rules! count_words {
    ($x:expr) => {
        RE_WORDS.captures_iter($x).count()
    };
}

enum Selection<'a> {
    Ok(Vec<&'a PathBuf>),
    Cancel,
    Skip,
    Invalid,
}

// die Keys in den hashmaps sind bereits hashes daher ist kein echter hasher mehr nötig
struct NaiveHasher(u64);
impl Default for NaiveHasher {
    fn default() -> Self {
        NaiveHasher(0)
    }
}
impl Hasher for NaiveHasher {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, _: &[u8]) {
        unimplemented!()
    }
    fn write_u64(&mut self, i: u64) {
        self.0 = i ^ i >> 7; // absolut keine ahnung wofür das ist aber war so in der Dokumentation
    }
}
type NaiveBuildHasher = BuildHasherDefault<NaiveHasher>;

fn main() {
    let matches = App::new("Dupe Finder")
        .version("0.1")
        .about("Finds and removes duplicate files (prioritizing the best name)")
        .arg(
            Arg::with_name("recursive")
                .help("Searches duplicate files in subdirectories")
                .short("r")
                .long("recursive"),
        )
        .arg(
            Arg::with_name("no_backup")
                .help("Delete files instead of just moving them")
                .long("no-backup"),
        )
        .arg(Arg::with_name("directory").required(true).multiple(true))
        .get_matches();

    let directories: Vec<_> = matches
        .values_of("directory")
        .unwrap()
        .map(|arg| Path::new(arg))
        .collect();
    if directories.iter().any(|p| !p.is_dir()) {
        println_stderr!("Not a valid Directory");
        return;
    }
    println!("{:?}", directories);
    do_stuff(
        &directories,
        matches.is_present("recursive"),
        !matches.is_present("no_backup"),
    );
}

fn visit_dirs(
    dir: &Path,
    hm: &mut HashMap<u64, Vec<PathBuf>, impl BuildHasher>,
    cnt: &mut u64,
    fs: &mut u64,
    pb: &ProgressBar,
    recursive: bool,
) {
    for entry in read_dir(dir).unwrap().filter_map(|e| e.ok()) {
        match entry.metadata() {
            Ok(ref m) if m.file_type().is_file() => {
                let p = entry.path();
                if !p.iter().any(|x| x == "duplicates") {
                    // let size = get_size!(m);
                    let size = m.get_file_size();
                    hm.entry(size).or_insert_with(Vec::new).push(p.to_owned());
                    *cnt += 1;
                    *fs += size;
                    pb.inc(1);
                }
            }
            Ok(ref m) if recursive && m.file_type().is_dir() => {
                visit_dirs(&entry.path(), hm, cnt, fs, pb, recursive);
            }
            Err(e) => println_stderr!("{:?}", e),
            _ => (),
        }
    }
}

fn do_stuff(dirs: &[&Path], recursive: bool, backup: bool) {
    let mut dup_count = 0u64;
    let mut dup_size = 0u64;
    let t = SystemTime::now();
    // println!("Pass1...");
    // pass1
    let mut pass1_files: HashMap<_, _, NaiveBuildHasher> = Default::default();
    let mut pass1_cnt = 0u64;
    let mut pass1_size = 0u64;

    let pb1 = ProgressBar::new_spinner();
    pb1.set_style(
        ProgressStyle::default_spinner()
            .template("Pass1: Searching Files {spinner:.green} [{elapsed_precise}]"),
    );
    for dir in dirs {
        visit_dirs(
            dir,
            &mut pass1_files,
            &mut pass1_cnt,
            &mut pass1_size,
            &pb1,
            recursive,
        )
    }
    pb1.finish();

    // let pass1_vec: Vec<_> = pass1_files.iter().filter(|&(_,y)| y.len() > 1).map(|(x,y)|y).collect();
    // pass2
    let mut pass2_files: HashMap<_, _, NaiveBuildHasher> = Default::default();
    let pb2 = ProgressBar::new(
        pass1_files
            .values()
            .filter(|x| x.len() > 1)
            .flat_map(|v| v)
            .count() as u64,
    );
    pb2.set_style(ProgressStyle::default_bar()
    .template("Pass2: Hashing Files {spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} ETA: {eta} ")
    .progress_chars("=D~8"));

    for entry in pass1_files.values().filter(|x| x.len() > 1).flat_map(|v| v) {
        let hash = hash_file(entry);
        let list = pass2_files.entry(hash).or_insert_with(Vec::new);
        if !list.is_empty() {
            dup_count += 1;
            if let Ok(m) = entry.metadata() {
                // dup_size += get_size!(m)
                dup_size += m.get_file_size();
            };
        }
        list.push(entry);

        pb2.inc(1);
    }
    pb2.finish_with_message("done");

    println!("\nPass2 finished");
    let dt = t.elapsed().unwrap();
    println!(
        "Time elapsed: {}.{}s",
        dt.as_secs(),
        dt.subsec_nanos() / 1000 / 1000
    );
    println!( "Scanned {} file(s) ({})", pass1_cnt, bytes_to_si(pass1_size));
    println!( "{} duplicates founds ({})", dup_count, bytes_to_si(dup_size));

    if dup_count == 0 {
        return;
    }

    if let Err(e) = select_action(&pass2_files, backup) {
        println_stderr!("Error: {}", e);
    }
}

fn select_action(
    dups: &HashMap<u64, Vec<&PathBuf>, impl BuildHasher>,
    backup: bool,
) -> Result<(), Box<dyn Error>> {
    let backup_dir = std::env::current_dir()?.join("duplicates");
    loop {
        if backup {
            print!("remove all duplicates?(backup in {:?}) ([y]es/[i]nteractive mode/[q]uit/[p]rint): ", backup_dir);
        } else {
            print!("permanently delete all duplicates?([y]es/[i]nteractive mode/[q]uit/[p]rint");
        }
        stdout().flush()?;
        let mut buf = String::new();
        stdin().read_line(&mut buf)?;
        let buf = buf.to_lowercase();
        if buf.is_empty() {
            continue
        }
        match buf.chars().nth(0).unwrap() { // unwrap hier ok weil buf bereits darauf geprüft wurde ob er leer ist
            'y' => {
                for entry in dups.values().filter(|x| x.len() > 1) {
                    let (_, remove) = select_files(entry);
                    for r in remove {
                        delete_file(r, backup, &backup_dir)?
                    }
                }
                break;
            }
            'i' => {
                println!("Interactive Mode:");
                let cnt = dups.values().filter(|x| x.len() > 1).count();
                for (idx,entry) in dups.values().filter(|x| x.len() > 1).enumerate() {
                    println!("File {} of {}",idx+1,cnt);
                    let (keep, remove) = select_files(entry);
                    loop {
                        match interactive_selection(keep, &remove) {
                            Selection::Ok(l) => {
                                for i in l {
                                    delete_file(i, backup, &backup_dir)?
                                }
                                break;
                            }
                            Selection::Skip => break,
                            Selection::Cancel => {
                                println!("Cancel");
                                return Ok(());
                            }
                            Selection::Invalid => println!("invalid input"),
                        };
                    }
                }
                break;
            }
            'p' => {
                for entry in dups.values().filter(|x| x.len() > 1) {
                    let (keep, remove) = select_files(entry);
                    println!("keep  : {:?}", keep);
                    for r in remove {
                        println!("delete: {:?}", r);
                    }
                    println!();
                }
            }
            'q' => return Ok(()),
            _ => println!("invalid input"),
        }
    }
    Ok(())
}

fn select_files<'a>(files: &[&'a PathBuf]) -> (&'a PathBuf, Vec<&'a PathBuf>) {
    // TODO error handling ?
    let mut tmp = Vec::from(files);

    for x in files {
        let x_name = file_stem!(x);
        for y in files {
            let y_name = file_stem!(y);

            if x_name == y_name && x != y {
                // bei zwei identischen dateinamen den mit dem kürzerem pfad aussortieren
                if x.components().count() > y.components().count() {
                    tmp.retain(|e| e != y) // alles außer y behalten ( y löschen )
                } else if x.components().count() < y.components().count() {
                    tmp.retain(|e| e != x)
                }
            } else if x_name != y_name
                && x_name.starts_with(y_name)
                && x_name.len() - y_name.len() <= 5
            {
                // dateinamen mit suffix aussortieren z.b. image.jpg und image(1).jpg
                tmp.retain(|e| e != x) // alles außer x behalten ( x löschen )
            }
        }
    }

    // Dateinamen anhand bestimmter prioritäten aussortieren
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
        } else if bestname_prio < 4
            || (bestname_prio == 4 && count_words!(n) > count_words!(current_n))
        {
            bestname = x;
            bestname_prio = 4;
        }
    }
    let mut tmp = Vec::from(files);
    tmp.retain(|e| *e != bestname);

    (bestname, tmp)
}

fn delete_file(fp: &Path, backup: bool, backup_dir: &Path) -> Result<(), Box<dyn Error>> {
    if backup {
        let p_bak = backup_dir.join(fp.file_name().ok_or("can't get filename")?);
        create_dir_all(p_bak.parent().ok_or("can't get parent directory")?)?;
        rename(fp, &p_bak)?;
    } else {
        remove_file(fp)?
    }
    Ok(())
}

fn interactive_selection<'a>(k: &'a PathBuf, r: &[&'a PathBuf]) -> Selection<'a> {
    let mut tmp = Vec::with_capacity(r.len() + 1);
    tmp.push(k);
    tmp.append(&mut Vec::from(r));

    for (i, v) in tmp.iter().enumerate() {
        if i == 0 {
            println!("* ({}): {}", i + 1, v.display());
        } else {
            println!("  ({}): {}", i + 1, v.display());
        }
    }
    println!("([c]ancel/[s]skip/Enter for default [1])");
    print!("select file to keep: ");
    stdout().flush().unwrap();
    let mut buf = String::new();
    stdin().read_line(&mut buf).unwrap();
    let buf = buf.trim().to_lowercase();

    if IS_NUMERIC.is_match(&buf) {
        let sel: usize = match buf.parse() {
            Ok(v) => v,
            _ => return Selection::Invalid,
        };
        if sel > tmp.len() {
            return Selection::Invalid;
        }
        tmp.remove(sel - 1);
    } else if buf.is_empty() {
        tmp.remove(0);
    } else if buf.starts_with('c') {
        return Selection::Cancel;
    } else if buf.starts_with('s') {
        return Selection::Skip;
    } else {
        return Selection::Invalid;
    }
    println!("delete: {:?}\n", tmp);

    Selection::Ok(tmp)
}
use std::io::Read;

fn hash_reader(reader: impl Read, mut hasher: impl Hasher) -> u64 {
    let mut br = BufReader::new(reader);
    // nutzt den buffer vom BufReader direkt zum hashen
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

fn hash_file(path: &Path) -> u64 {
    let mut hasher = XxHash::with_seed(0);
    let fd = File::open(path).unwrap();
    let mut br = BufReader::new(&fd);
    // nutzt den buffer vom BufReader direkt zum hashen
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

fn bytes_to_si(size: u64) -> String {
    let units = ["B", "KiB", "MiB", "GiB", "TiB"];
    if size == 0 {
        "0 B".to_string()
    } else {
        let mut p = (size as f64).log(1024.0) as usize;
        if p > units.len() - 1 {
            p = units.len() - 1;
        }
        format!("{:.2} {}", (size as f64) / 1024_f64.powi(p as i32), units[p])
    }
}
