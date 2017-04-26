#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate clap;
extern crate twox_hash;
extern crate indicatif;
extern crate walkdir;

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
#[cfg(windows)]
use std::os::windows::fs::MetadataExt;

use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::fs::{File, create_dir_all, rename, read_dir};
use std::io::{BufReader, BufRead, stdin, stdout, Write};
use std::hash::{Hasher, BuildHasherDefault};
use std::time::SystemTime;
use regex::Regex;
use clap::{Arg, App};
use twox_hash::XxHash;
use indicatif::{ProgressBar,ProgressStyle};
use walkdir::WalkDir;
use std::hash::BuildHasher;

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

macro_rules! count_words {
    ($x:expr) => (RE_WORDS.captures_iter($x).count())
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
        self.0 = i ^ i >> 7;
    }
}
type NaiveBuildHasher = BuildHasherDefault<NaiveHasher>;

fn main() {
    let matches = App::new("Chan Dupe Finder")
        .version("0.1")
        .about("Finds and removes duplicate files (prioritizing the best name)")
        .arg(Arg::with_name("recursive")
                 .help("Searches duplicate files in subdirectories")
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

fn do_stuff(dir: &Path, recursive: bool) {
    let mut dup_count = 0u64;
    let mut dup_size = 0u64;
    let t = SystemTime::now();
    // println!("Pass1...");
    // pass1
    let mut pass1_files: HashMap<_, _, NaiveBuildHasher> = Default::default();
    let mut pass1_cnt = 0u64;
    let mut pass1_size = 0u64;

    let pb1 = ProgressBar::new(0);
    pb1.set_style(ProgressStyle::default_spinner());
    pb1.set_message("Pass1: Searching Files");

    if recursive {
        for entry in WalkDir::new(&dir)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok()) {
            match entry.metadata() {
                Ok(ref m) if m.file_type().is_file() => {
                    let p = entry.path();
                    if !p.components().any(|x| x.as_os_str() == "duplicates") {
                        let size = get_size!(m);
                        pass1_files
                            .entry(size)
                            .or_insert_with(Vec::new)
                            .push(p.to_owned());
                        pass1_cnt += 1;
                        pass1_size += size;
                        pb1.inc(1);
                    }
                }
                Err(e) => println_stderr!("{:?}", e),
                _ => (),
            }
        }
    } else {
        for entry in read_dir(&dir).unwrap().filter_map(|e| e.ok()) {
            match entry.metadata() {
                Ok(ref m) if m.file_type().is_file() => {
                    let size = get_size!(m);
                    pass1_files
                        .entry(size)
                        .or_insert_with(Vec::new)
                        .push(entry.path());
                    pass1_cnt += 1;
                    pass1_size += size;
                    pb1.inc(1);
                }
                Err(e) => println_stderr!("{:?}", e),
                _ => (),
            }
        }
    }
    pb1.finish();


    // let pass1_vec: Vec<_> = pass1_files.iter().filter(|&(_,y)| y.len() > 1).map(|(x,y)|y).collect();
    println!("Pass2: Hashing Files");
    // pass2
    let mut pass2_files: HashMap<_, _, NaiveBuildHasher> = Default::default();
    let pb = ProgressBar::new(pass1_files
                                      .values()
                                      .filter(|x| x.len() > 1)
                                      .flat_map(|v| v)
                                      .count() as u64);


    for entry in pass1_files
            .values()
            .filter(|x| x.len() > 1)
            .flat_map(|v| v) {
        let hash = hash_file(entry);

        let mut list = pass2_files.entry(hash).or_insert_with(Vec::new);
        if !list.is_empty() {
            dup_count += 1;
            if let Ok(m) = entry.metadata() {
                dup_size += get_size!(m)
            };
        }
        list.push(entry);

        pb.inc(1);
    }
    pb.finish();

    println!("\nPass2 finished");
    let dt = t.elapsed().unwrap();
    println!("Time elapsed: {}.{}s",
             dt.as_secs(),
             (dt.subsec_nanos() / 1000 / 1000) as u64);

    println!("Scanned {} file(s) ({})",
             pass1_cnt,
             bytes_to_si(pass1_size));
    println!("{} duplicates founds ({})",
             dup_count,
             bytes_to_si(dup_size));

    if dup_count == 0 {
        return;
    }

    select_action(&pass2_files, dir);
}

fn select_action<S: BuildHasher>(dups: &HashMap<u64, Vec<&PathBuf>, S>, dir: &Path) {
    loop {
        print!("remove all duplicates?(backup in {:?}) ([y]es/[i]nteractive \
                mode/[q]uit/[p]rint): ",
               dir.join("duplicates"));
        stdout().flush().unwrap();
        let mut buf = String::new();
        stdin().read_line(&mut buf).unwrap();
        let buf = buf.trim().to_lowercase();

        if buf.starts_with('y') {
            for entry in dups.values().filter(|x| x.len() > 1) {
                let (_, remove) = select_files(entry);
                for r in remove {
                    if let Err(e) = backup_file(r, dir) {
                        println_stderr!("{:?}: {}", r, e);
                    }
                }
            }
            break;
        } else if buf.starts_with('i') {
            println!("Interactive Mode:");
            for entry in dups.values().filter(|x| x.len() > 1) {
                let (keep, remove) = select_files(entry);
                loop {
                    match interactive_selection(dir, keep, &remove) {
                        Selection::Ok(l) => {
                            for i in l {
                                if let Err(e) = backup_file(i, dir) {
                                    println_stderr!("{:?}: {}", i, e);
                                }
                            }
                            break;
                        }
                        Selection::Skip => break,
                        Selection::Cancel => {
                            println!("Cancel");
                            return;
                        }
                        Selection::Invalid => println!("invalid input"),
                    };
                }
            }
            break;
        } else if buf.starts_with('p') {
            for entry in dups.values().filter(|x| x.len() > 1) {
                let (keep, remove) = select_files(entry);
                println!("keep  : {:?}", keep.strip_prefix(dir).unwrap());
                for r in remove {
                    println!("delete: {:?}", r.strip_prefix(dir).unwrap());
                }
                println!("");
            }

        } else if buf.starts_with('q') {
            return;
        } else {
            println!("invalid input");
        }
    }
}

pub fn select_files<'a>(files: &[&'a PathBuf]) -> (&'a PathBuf, Vec<&'a PathBuf>) {
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
            } else if x_name != y_name && x_name.starts_with(y_name) && x_name.len() - y_name.len() <= 5 {
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
        } else if bestname_prio < 4 ||
                  (bestname_prio == 4 && count_words!(n) > count_words!(current_n)) {
            bestname = x;
            bestname_prio = 4;
        }
    }
    let mut tmp = Vec::from(files);
    tmp.retain(|e| *e != bestname);

    (bestname, tmp)
}

fn backup_file(fp: &Path, basedir: &Path) -> Result<(), String> {
    let backupdir = basedir.join(Path::new("duplicates"));
    let p_bak = backupdir.join(match fp.strip_prefix(basedir) {
        Ok(v) => v,
        Err(e) => {return Err(format!("convert absolute to relative path: {}", e))}
    });

    // println!("{:?} -> {:?}", fp, p_bak);
    if let Err(e) = create_dir_all(
        match p_bak.parent() {
            Some(v) => v,
            None => return Err("Something happened".to_string()),
        }) {
    return Err(format!("Create Backup Dir: {}", e));
    };

    if let Err(e) = rename(fp, &p_bak) {
        return Err(format!("Move file: {}", e));
    };

    Ok(())
}

fn interactive_selection<'a>(basedir: &Path, k: &'a PathBuf, r: &[&'a PathBuf]) -> Selection<'a> {
    let mut tmp = Vec::with_capacity(r.len() + 1);
    tmp.push(k);
    tmp.append(&mut Vec::from(r));

    for (i, v) in tmp.iter().enumerate() {
        if i == 0 {
            println!("* ({}): {}",
                     i + 1,
                     v.strip_prefix(basedir).unwrap().display());
        } else {
            println!("  ({}): {}",
                     i + 1,
                     v.strip_prefix(basedir).unwrap().display());
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

fn hash_reader<R: Read, H: Hasher>(reader: R, mut hasher: H) -> u64 {
    let mut br = BufReader::new(reader);
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_files1() {
        let a = PathBuf::from("12345.bin");
        let b = PathBuf::from("3898d553.bin");
        let (x, _) = select_files(&[&a, &b]);
        assert_eq!(&b, x);
    }

    #[test]
    fn select_files2() {
        let a = PathBuf::from("we1223ffqwe21.bin");
        let b = PathBuf::from("3898d553.bin");
        let c = PathBuf::from("3898d553.bin");
        let (x, _) = select_files(&[&a, &b, &c]);
        assert_eq!(&a, x);
    }

    #[test]
    fn select_files3() {
        let a = PathBuf::from("12351235.bin");
        let b = PathBuf::from("512363453534.bin");
        let (x, _) = select_files(&[&a, &b]);
        assert_eq!(&b, x);
    }

    #[test]
    fn select_files4() {
        let a = PathBuf::from("5123.bin");
        let b = PathBuf::from("12351235.bin");
        let c = PathBuf::from("12351235(1).bin");
        let (x, _) = select_files(&[&a, &b, &c]);
        assert_eq!(&b, x);
    }

    #[test]
    fn select_files5() {
        let a = PathBuf::from("blabla.bin");
        let b = PathBuf::from("blablablabla foobar.bin");
        let c = PathBuf::from("bla bla foo bar.bin");
        let (keep, _) = select_files(&[&a, &b, &c]);
        assert_eq!(&c, keep);
    }

    #[test]
    fn select_files6() {
        let a = PathBuf::from("12351235.bin");
        let b = PathBuf::from("12351235(1).bin");
        let c = PathBuf::from("12351235(1)(1).bin");
        let (keep, _) = select_files(&[&a, &b, &c]);
        assert_eq!(&a, keep);
    }
}
