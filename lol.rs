#![feature(prelude_import)]
#![no_std]
#[prelude_import]
use std::prelude::v1::*;
#[macro_use]
extern crate std as std;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate clap;
extern crate twox_hash;
extern crate pbr;
extern crate walkdir;

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
use pbr::ProgressBar;
use walkdir::WalkDir;


// TODO idee für zusatzmodus: wenn rekursiv nur duplikate entfernen wenn beide im selbem ordner sind








// pass1
// wenn ich schon twox_hash beneutze um die dateien zu vergleichen kann ich es auch für die hashmap benutzen


// TODO dateien ignorieren die im "duplicates" ordner liegen


// let pass1_vec: Vec<_> = pass1_files.iter().filter(|&(_,y)| y.len() > 1).map(|(x,y)|y).collect();
// pass2











// TODO error handling ?


// bei zwei identischen dateinamen den mit dem kürzerem pfad aussortieren
// alles außer y behalten ( y löschen )
// dateinamen mit suffix aussortieren z.b. image.jpg und image(1).jpg
// alles außer x behalten ( x löschen )

// Dateinamen anhand bestimmter prioritäten aussortieren



// println!("{:?} -> {:?}", fp, p_bak);









#[allow(missing_copy_implementations)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
struct IS_NUMERIC {
    __private_field: (),
}
#[doc(hidden)]
static IS_NUMERIC: IS_NUMERIC = IS_NUMERIC { __private_field: () };
impl ::__Deref for IS_NUMERIC {
    type Target = Regex;
    #[allow(unsafe_code)]
    fn deref<'a>(&'a self) -> &'a Regex {
        unsafe {
            #[inline(always)]
            fn __static_ref_initialize() -> Regex {
                Regex::new(r"^[:digit:]+$").unwrap()
            }
            #[inline(always)]
            unsafe fn __stability() -> &'static Regex {
                use std::sync::ONCE_INIT;
                static mut LAZY: ::lazy::Lazy<Regex> = ::lazy::Lazy(0 as *const Regex, ONCE_INIT);
                LAZY.get(__static_ref_initialize)
            }
            __stability()
        }
    }
}
#[allow(missing_copy_implementations)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
struct IS_HEX {
    __private_field: (),
}
#[doc(hidden)]
static IS_HEX: IS_HEX = IS_HEX { __private_field: () };
impl ::__Deref for IS_HEX {
    type Target = Regex;
    #[allow(unsafe_code)]
    fn deref<'a>(&'a self) -> &'a Regex {
        unsafe {
            #[inline(always)]
            fn __static_ref_initialize() -> Regex {
                Regex::new(r"^[:xdigit:]+$").unwrap()
            }
            #[inline(always)]
            unsafe fn __stability() -> &'static Regex {
                use std::sync::ONCE_INIT;
                static mut LAZY: ::lazy::Lazy<Regex> = ::lazy::Lazy(0 as *const Regex, ONCE_INIT);
                LAZY.get(__static_ref_initialize)
            }
            __stability()
        }
    }
}
#[allow(missing_copy_implementations)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
struct IS_ALNUM {
    __private_field: (),
}
#[doc(hidden)]
static IS_ALNUM: IS_ALNUM = IS_ALNUM { __private_field: () };
impl ::__Deref for IS_ALNUM {
    type Target = Regex;
    #[allow(unsafe_code)]
    fn deref<'a>(&'a self) -> &'a Regex {
        unsafe {
            #[inline(always)]
            fn __static_ref_initialize() -> Regex {
                Regex::new(r"^[:alnum:]+$").unwrap()
            }
            #[inline(always)]
            unsafe fn __stability() -> &'static Regex {
                use std::sync::ONCE_INIT;
                static mut LAZY: ::lazy::Lazy<Regex> = ::lazy::Lazy(0 as *const Regex, ONCE_INIT);
                LAZY.get(__static_ref_initialize)
            }
            __stability()
        }
    }
}
#[allow(missing_copy_implementations)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
struct RE_WORDS {
    __private_field: (),
}
#[doc(hidden)]
static RE_WORDS: RE_WORDS = RE_WORDS { __private_field: () };
impl ::__Deref for RE_WORDS {
    type Target = Regex;
    #[allow(unsafe_code)]
    fn deref<'a>(&'a self) -> &'a Regex {
        unsafe {
            #[inline(always)]
            fn __static_ref_initialize() -> Regex {
                Regex::new(r"[:alnum:]{2,}").unwrap()
            }
            #[inline(always)]
            unsafe fn __stability() -> &'static Regex {
                use std::sync::ONCE_INIT;
                static mut LAZY: ::lazy::Lazy<Regex> = ::lazy::Lazy(0 as *const Regex, ONCE_INIT);
                LAZY.get(__static_ref_initialize)
            }
            __stability()
        }
    }
}
enum Selection<'a> {
    Ok(Vec<&'a PathBuf>),
    Cancel,
    Skip,
    Invalid,
}
fn main() {
    let matches = App::new("Chan Dupe Finder")
        .version("0.1")
        .about("Finds and removes duplicate files (prioritizing the best name)")
        .arg(Arg::with_name("recursive")
            .help("Searches duplicate files in subdirectories")
            .short("r")
            .long("recursive"))
        .arg(Arg::with_name("directory").required(true).index(1))
        .get_matches();
    let directory = matches.value_of("directory").unwrap();
    let dir = Path::new(directory);
    if !dir.is_dir() {
        {
            let r =
                &mut ::std::io::stderr().write_fmt(::std::fmt::Arguments::new_v1({
                                                                                     static __STATIC_FMTSTR:
                                                                                            &'static [&'static str]
                                                                                            =
                                                                                         &["Not valid Directory\n"];
                                                                                     __STATIC_FMTSTR
                                                                                 },
                                                                                 &match ()
                                                                                      {
                                                                                      ()
                                                                                      =>
                                                                                      [],
                                                                                  }));
            r.expect("failed printing to stderr");
        };
        return;
    }
    do_stuff(dir, matches.is_present("recursive"));
}
fn do_stuff(dir: &Path, recursive: bool) {
    let mut dup_count = 0u64;
    let mut dup_size = 0u64;
    let t = SystemTime::now();
    ::io::_print(::std::fmt::Arguments::new_v1({
                                                   static __STATIC_FMTSTR:
                                                          &'static [&'static str]
                                                          =
                                                       &["Pass1...\n"];
                                                   __STATIC_FMTSTR
                                               },
                                               &match () {
                                                   () => [],
                                               }));
    let mut pass1_files: HashMap<_, _, BuildHasherDefault<XxHash>> = Default::default();
    let mut pass1_cnt = 0u64;
    let mut pass1_size = 0u64;
    if recursive {
        {
            for entry in WalkDir::new(&dir).into_iter().filter_map(|e| e.ok()) {
                match entry.metadata() {
                    Ok(ref m) if m.file_type().is_file() => {
                        let size = m.file_size();
                        pass1_files.entry(size)
                            .or_insert_with(Vec::new)
                            .push(entry.path().to_owned());
                        pass1_cnt += 1;
                        pass1_size += size;
                    }
                    Err(e) => {
                        let r =
                            &mut ::std::io::stderr().write_fmt(::std::fmt::Arguments::new_v1({
                                                                                                 static __STATIC_FMTSTR:
                                                                                                        &'static [&'static str]
                                                                                                        =
                                                                                                     &["",
                                                                                                       "\n"];
                                                                                                 __STATIC_FMTSTR
                                                                                             },
                                                                                             &match (&e,)
                                                                                                  {
                                                                                                  (__arg0,)
                                                                                                  =>
                                                                                                  [::std::fmt::ArgumentV1::new(__arg0,
                                                                                                                               ::std::fmt::Debug::fmt)],
                                                                                              }));
                        r.expect("failed printing to stderr");
                    }
                    _ => (),
                }
            }
        }
    } else {
        {
            for entry in read_dir(&dir).unwrap().filter_map(|e| e.ok()) {
                match entry.metadata() {
                    Ok(ref m) if m.file_type().is_file() => {
                        let size = m.file_size();
                        pass1_files.entry(size)
                            .or_insert_with(Vec::new)
                            .push(entry.path().to_owned());
                        pass1_cnt += 1;
                        pass1_size += size;
                    }
                    Err(e) => {
                        let r =
                            &mut ::std::io::stderr().write_fmt(::std::fmt::Arguments::new_v1({
                                                                                                 static __STATIC_FMTSTR:
                                                                                                        &'static [&'static str]
                                                                                                        =
                                                                                                     &["",
                                                                                                       "\n"];
                                                                                                 __STATIC_FMTSTR
                                                                                             },
                                                                                             &match (&e,)
                                                                                                  {
                                                                                                  (__arg0,)
                                                                                                  =>
                                                                                                  [::std::fmt::ArgumentV1::new(__arg0,
                                                                                                                               ::std::fmt::Debug::fmt)],
                                                                                              }));
                        r.expect("failed printing to stderr");
                    }
                    _ => (),
                }
            }
        }
    }
    ::io::_print(::std::fmt::Arguments::new_v1({
                                                   static __STATIC_FMTSTR:
                                                          &'static [&'static str]
                                                          =
                                                       &["Pass2: Hashing Files\n"];
                                                   __STATIC_FMTSTR
                                               },
                                               &match () {
                                                   () => [],
                                               }));
    let mut pass2_files: HashMap<_, _, BuildHasherDefault<XxHash>> = Default::default();
    let mut pb = ProgressBar::new(pass1_files.values()
        .filter(|x| x.len() > 1)
        .flat_map(|v| v.iter())
        .count() as u64);
    pb.format("8=D~D");
    for entry in pass1_files.values().filter(|x| x.len() > 1).flat_map(|v| v.iter()) {
        let hash = hash_file(entry);
        let mut list = pass2_files.entry(hash).or_insert_with(Vec::new);
        if !list.is_empty() {
            dup_count += 1;
            if let Ok(m) = entry.metadata() {
                dup_size += m.file_size()
            };
        }
        list.push(entry);
        pb.inc();
    }
    pb.finish();
    ::io::_print(::std::fmt::Arguments::new_v1({
                                                   static __STATIC_FMTSTR:
                                                          &'static [&'static str]
                                                          =
                                                       &["\nPass2 finished\n"];
                                                   __STATIC_FMTSTR
                                               },
                                               &match () {
                                                   () => [],
                                               }));
    let dt = t.elapsed().unwrap();
    ::io::_print(::std::fmt::Arguments::new_v1({
                                                   static __STATIC_FMTSTR:
                                                          &'static [&'static str]
                                                          =
                                                       &["Time elapsed: ",
                                                         ".", "s\n"];
                                                   __STATIC_FMTSTR
                                               },
                                               &match (&dt.as_secs(),
                                                       &((dt.subsec_nanos() /
                                                              1000 / 1000) as
                                                             u64)) {
                                                    (__arg0, __arg1) =>
                                                    [::std::fmt::ArgumentV1::new(__arg0,
                                                                                 ::std::fmt::Display::fmt),
                                                     ::std::fmt::ArgumentV1::new(__arg1,
                                                                                 ::std::fmt::Display::fmt)],
                                                }));
    ::io::_print(::std::fmt::Arguments::new_v1({
                                                   static __STATIC_FMTSTR:
                                                          &'static [&'static str]
                                                          =
                                                       &["Scanned ",
                                                         " file(s) (", ")\n"];
                                                   __STATIC_FMTSTR
                                               },
                                               &match (&pass1_cnt, &bytes_to_si(pass1_size)) {
                                                   (__arg0, __arg1) =>
                                                    [::std::fmt::ArgumentV1::new(__arg0,
                                                                                 ::std::fmt::Display::fmt),
                                                     ::std::fmt::ArgumentV1::new(__arg1,
                                                                                 ::std::fmt::Display::fmt)],
                                               }));
    ::io::_print(::std::fmt::Arguments::new_v1({
                                                   static __STATIC_FMTSTR:
                                                          &'static [&'static str]
                                                          =
                                                       &["",
                                                         " duplicates founds (",
                                                         ")\n"];
                                                   __STATIC_FMTSTR
                                               },
                                               &match (&dup_count, &bytes_to_si(dup_size)) {
                                                   (__arg0, __arg1) =>
                                                    [::std::fmt::ArgumentV1::new(__arg0,
                                                                                 ::std::fmt::Display::fmt),
                                                     ::std::fmt::ArgumentV1::new(__arg1,
                                                                                 ::std::fmt::Display::fmt)],
                                               }));
    if dup_count == 0 {
        return;
    }
    select_action(pass2_files, dir);
}
fn select_action(dups: HashMap<u64, Vec<&PathBuf>, BuildHasherDefault<XxHash>>, dir: &Path) {
    loop {
        ::io::_print(::std::fmt::Arguments::new_v1({
                                                       static __STATIC_FMTSTR:
                                                              &'static [&'static str]
                                                              =
                                                           &["remove all duplicates?(backup in ",
                                                             ") ([y]es/[i]nteractive mode/[q]uit/[p]rint): "];
                                                       __STATIC_FMTSTR
                                                   },
                                                   &match (&dir.join("duplicates"),) {
                                                       (__arg0,) =>
                                                        [::std::fmt::ArgumentV1::new(__arg0,
                                                                                     ::std::fmt::Debug::fmt)],
                                                   }));
        stdout().flush().unwrap();
        let mut buf = String::new();
        stdin().read_line(&mut buf).unwrap();
        let buf = buf.trim().to_lowercase();
        if buf.starts_with('y') {
            for entry in dups.values().filter(|x| x.len() > 1) {
                let (_, remove) = select_files(entry);
                for r in remove {
                    if let Err(e) = backup_file(r, dir) {
                        {
                            let r =
                                &mut ::std::io::stderr().write_fmt(::std::fmt::Arguments::new_v1({
                                                                                                     static __STATIC_FMTSTR:
                                                                                                            &'static [&'static str]
                                                                                                            =
                                                                                                         &["",
                                                                                                           ": ",
                                                                                                           "\n"];
                                                                                                     __STATIC_FMTSTR
                                                                                                 },
                                                                                                 &match (&r,
                                                                                                         &e)
                                                                                                      {
                                                                                                      (__arg0,
                                                                                                       __arg1)
                                                                                                      =>
                                                                                                      [::std::fmt::ArgumentV1::new(__arg0,
                                                                                                                                   ::std::fmt::Debug::fmt),
                                                                                                       ::std::fmt::ArgumentV1::new(__arg1,
                                                                                                                                   ::std::fmt::Display::fmt)],
                                                                                                  }));
                            r.expect("failed printing to stderr");
                        };
                    }
                }
            }
            break;
        } else if buf.starts_with('i') {
            ::io::_print(::std::fmt::Arguments::new_v1({
                                                           static __STATIC_FMTSTR:
                                                                  &'static [&'static str]
                                                                  =
                                                               &["Interactive Mode:\n"];
                                                           __STATIC_FMTSTR
                                                       },
                                                       &match () {
                                                           () => [],
                                                       }));
            for entry in dups.values().filter(|x| x.len() > 1) {
                let (keep, remove) = select_files(entry);
                loop {
                    match interactive_selection(dir, keep, &remove) {
                        Selection::Ok(l) => {
                            for i in l {
                                if let Err(e) = backup_file(i, dir) {
                                    {
                                        let r =
                                            &mut ::std::io::stderr().write_fmt(::std::fmt::Arguments::new_v1({
                                                                                                                 static __STATIC_FMTSTR:
                                                                                                                        &'static [&'static str]
                                                                                                                        =
                                                                                                                     &["",
                                                                                                                       ": ",
                                                                                                                       "\n"];
                                                                                                                 __STATIC_FMTSTR
                                                                                                             },
                                                                                                             &match (&i,
                                                                                                                     &e)
                                                                                                                  {
                                                                                                                  (__arg0,
                                                                                                                   __arg1)
                                                                                                                  =>
                                                                                                                  [::std::fmt::ArgumentV1::new(__arg0,
                                                                                                                                               ::std::fmt::Debug::fmt),
                                                                                                                   ::std::fmt::ArgumentV1::new(__arg1,
                                                                                                                                               ::std::fmt::Display::fmt)],
                                                                                                              }));
                                        r.expect("failed printing to stderr");
                                    };
                                }
                            }
                            break;
                        }
                        Selection::Skip => break,
                        Selection::Cancel => {
                            ::io::_print(::std::fmt::Arguments::new_v1({
                                                                           static __STATIC_FMTSTR:
                                                                                  &'static [&'static str]
                                                                                  =
                                                                               &["Cancel\n"];
                                                                           __STATIC_FMTSTR
                                                                       },
                                                                       &match () {
                                                                           ()
                                                                            =>
                                                                            [],
                                                                       }));
                            return;
                        }
                        Selection::Invalid => {
                            ::io::_print(::std::fmt::Arguments::new_v1({
                                                                           static __STATIC_FMTSTR:
                                                                              &'static [&'static str]
                                                                              =
                                                                           &["invalid input\n"];
                                                                           __STATIC_FMTSTR
                                                                       },
                                                                       &match () {
                                                                           () =>
                                                                        [],
                                                                       }))
                        }
                    };
                }
            }
            break;
        } else if buf.starts_with('p') {
            for entry in dups.values().filter(|x| x.len() > 1) {
                let (keep, remove) = select_files(entry);
                ::io::_print(::std::fmt::Arguments::new_v1({
                                                               static __STATIC_FMTSTR:
                                                                      &'static [&'static str]
                                                                      =
                                                                   &["keep  : ",
                                                                     "\n"];
                                                               __STATIC_FMTSTR
                                                           },
                                                           &match (&keep.strip_prefix(dir)
                                                                       .unwrap(),) {
                                                               (__arg0,) =>
                                                                [::std::fmt::ArgumentV1::new(__arg0,
                                                                                             ::std::fmt::Debug::fmt)],
                                                           }));
                for r in remove {
                    ::io::_print(::std::fmt::Arguments::new_v1({
                                                                   static __STATIC_FMTSTR:
                                                                          &'static [&'static str]
                                                                          =
                                                                       &["delete: ",
                                                                         "\n"];
                                                                   __STATIC_FMTSTR
                                                               },
                                                               &match (&r.strip_prefix(dir)
                                                                           .unwrap(),) {
                                                                   (__arg0,)
                                                                    =>
                                                                    [::std::fmt::ArgumentV1::new(__arg0,
                                                                                                 ::std::fmt::Debug::fmt)],
                                                               }));
                }
                ::io::_print(::std::fmt::Arguments::new_v1({
                                                               static __STATIC_FMTSTR:
                                                                      &'static [&'static str]
                                                                      =
                                                                   &["\n"];
                                                               __STATIC_FMTSTR
                                                           },
                                                           &match () {
                                                               () => [],
                                                           }));
            }
        } else if buf.starts_with('q') {
            return;
        } else {
            ::io::_print(::std::fmt::Arguments::new_v1({
                                                           static __STATIC_FMTSTR:
                                                                  &'static [&'static str]
                                                                  =
                                                               &["invalid input\n"];
                                                           __STATIC_FMTSTR
                                                       },
                                                       &match () {
                                                           () => [],
                                                       }));
        }
    }
}
fn select_files<'a>(files: &[&'a PathBuf]) -> (&'a PathBuf, Vec<&'a PathBuf>) {
    let mut tmp = Vec::from(files);
    for x in files {
        let x_name = x.file_stem()
            .unwrap_or(std::ffi::OsStr::new("decoding error"))
            .to_str()
            .unwrap_or("decoding error");
        for y in files {
            let y_name = y.file_stem()
                .unwrap_or(std::ffi::OsStr::new("decoding error"))
                .to_str()
                .unwrap_or("decoding error");
            if x_name == y_name && x != y {
                if x.components().count() > y.components().count() {
                    tmp.retain(|e| e != y)
                } else if x.components().count() < y.components().count() {
                    tmp.retain(|e| e != x)
                }
            } else if x_name != y_name && x_name.starts_with(y_name) {
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
        let n = x.file_stem()
            .unwrap_or(std::ffi::OsStr::new("decoding error"))
            .to_str()
            .unwrap_or("decoding error");
        let current_n = bestname.file_stem()
            .unwrap_or(std::ffi::OsStr::new("decoding error"))
            .to_str()
            .unwrap_or("decoding error");
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
                  (bestname_prio == 4 &&
                   RE_WORDS.captures_iter(n).count() > RE_WORDS.captures_iter(current_n).count()) {
            bestname = x;
            bestname_prio = 4;
        }
    }
    let mut tmp = Vec::from(files);
    tmp.retain(|e| e != &bestname);
    (bestname, tmp)
}
fn backup_file(fp: &Path, basedir: &Path) -> Result<(), String> {
    let backupdir = basedir.join(Path::new("duplicates"));
    let p_bak =
        backupdir.join(match fp.strip_prefix(basedir) {
                           Ok(v) => v,
                           Err(e) =>
                           return Err(::fmt::format(::std::fmt::Arguments::new_v1({
                                                                                      static __STATIC_FMTSTR:
                                                                                             &'static [&'static str]
                                                                                             =
                                                                                          &["convert absolute to relative path: "];
                                                                                      __STATIC_FMTSTR
                                                                                  },
                                                                                  &match (&e,)
                                                                                       {
                                                                                       (__arg0,)
                                                                                       =>
                                                                                       [::std::fmt::ArgumentV1::new(__arg0,
                                                                                                                    ::std::fmt::Display::fmt)],
                                                                                   }))),
                       });
    if let Err(e) = create_dir_all(match p_bak.parent() {
        Some(v) => v,
        None => return Err("Something happened".to_string()),
    }) {
        return Err(::fmt::format(::std::fmt::Arguments::new_v1({
                                                                   static __STATIC_FMTSTR:
                                                                          &'static [&'static str]
                                                                          =
                                                                       &["Create Backup Dir: "];
                                                                   __STATIC_FMTSTR
                                                               },
                                                               &match (&e,) {
                                                                   (__arg0,)
                                                                    =>
                                                                    [::std::fmt::ArgumentV1::new(__arg0,
                                                                                                 ::std::fmt::Display::fmt)],
                                                               })));
    };
    if let Err(e) = rename(fp, &p_bak) {
        return Err(::fmt::format(::std::fmt::Arguments::new_v1({
                                                                   static __STATIC_FMTSTR:
                                                                          &'static [&'static str]
                                                                          =
                                                                       &["Move file: "];
                                                                   __STATIC_FMTSTR
                                                               },
                                                               &match (&e,) {
                                                                   (__arg0,)
                                                                    =>
                                                                    [::std::fmt::ArgumentV1::new(__arg0,
                                                                                                 ::std::fmt::Display::fmt)],
                                                               })));
    };
    Ok(())
}
fn interactive_selection<'a>(basedir: &Path, k: &'a PathBuf, r: &[&'a PathBuf]) -> Selection<'a> {
    let mut tmp = Vec::with_capacity(r.len() + 1);
    tmp.push(k);
    tmp.append(&mut Vec::from(r));
    for (i, v) in tmp.iter().enumerate() {
        if i == 0 {
            ::io::_print(::std::fmt::Arguments::new_v1({
                                                           static __STATIC_FMTSTR:
                                                                  &'static [&'static str]
                                                                  =
                                                               &["* (", "): ",
                                                                 "\n"];
                                                           __STATIC_FMTSTR
                                                       },
                                                       &match (&(i + 1),
                                                               &v.strip_prefix(basedir)
                                                                   .unwrap()
                                                                   .display()) {
                                                           (__arg0, __arg1)
                                                            =>
                                                            [::std::fmt::ArgumentV1::new(__arg0,
                                                                                         ::std::fmt::Display::fmt),
                                                             ::std::fmt::ArgumentV1::new(__arg1,
                                                                                         ::std::fmt::Display::fmt)],
                                                       }));
        } else {
            ::io::_print(::std::fmt::Arguments::new_v1({
                                                           static __STATIC_FMTSTR:
                                                                  &'static [&'static str]
                                                                  =
                                                               &["  (", "): ",
                                                                 "\n"];
                                                           __STATIC_FMTSTR
                                                       },
                                                       &match (&(i + 1),
                                                               &v.strip_prefix(basedir)
                                                                   .unwrap()
                                                                   .display()) {
                                                           (__arg0, __arg1)
                                                            =>
                                                            [::std::fmt::ArgumentV1::new(__arg0,
                                                                                         ::std::fmt::Display::fmt),
                                                             ::std::fmt::ArgumentV1::new(__arg1,
                                                                                         ::std::fmt::Display::fmt)],
                                                       }));
        }
    }
    ::io::_print(::std::fmt::Arguments::new_v1({
                                                   static __STATIC_FMTSTR:
                                                          &'static [&'static str]
                                                          =
                                                       &["([c]ancel/[s]skip/Enter for default [1])\n"];
                                                   __STATIC_FMTSTR
                                               },
                                               &match () {
                                                   () => [],
                                               }));
    ::io::_print(::std::fmt::Arguments::new_v1({
                                                   static __STATIC_FMTSTR:
                                                          &'static [&'static str]
                                                          =
                                                       &["select file to keep: "];
                                                   __STATIC_FMTSTR
                                               },
                                               &match () {
                                                   () => [],
                                               }));
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
    ::io::_print(::std::fmt::Arguments::new_v1({
                                                   static __STATIC_FMTSTR:
                                                          &'static [&'static str]
                                                          =
                                                       &["delete: ", "\n\n"];
                                                   __STATIC_FMTSTR
                                               },
                                               &match (&tmp,) {
                                                   (__arg0,) =>
                                                    [::std::fmt::ArgumentV1::new(__arg0,
                                                                                 ::std::fmt::Debug::fmt)],
                                               }));
    Selection::Ok(tmp)
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
        ::fmt::format(::std::fmt::Arguments::new_v1_formatted({
                                                                  static __STATIC_FMTSTR:
                                                                         &'static [&'static str]
                                                                         =
                                                                      &["",
                                                                        " "];
                                                                  __STATIC_FMTSTR
                                                              },
                                                              &match (&((size as f64) /
                                                                        1024f64.powi(p as i32)),
                                                                      &units[p]) {
                                                                  (__arg0,
                                                                    __arg1) =>
                                                                   [::std::fmt::ArgumentV1::new(__arg0,
                                                                                                ::std::fmt::Display::fmt),
                                                                    ::std::fmt::ArgumentV1::new(__arg1,
                                                                                                ::std::fmt::Display::fmt)],
                                                              },
                                                              {
                                                                  static __STATIC_FMTARGS:
                                                                         &'static [::std::fmt::rt::v1::Argument]
                                                                         =
                                                                      &[::std::fmt::rt::v1::Argument{position:
                                                                                                         ::std::fmt::rt::v1::Position::At(0usize),
                                                                                                     format:
                                                                                                         ::std::fmt::rt::v1::FormatSpec{fill:
                                                                                                                                            ' ',
                                                                                                                                        align:
                                                                                                                                            ::std::fmt::rt::v1::Alignment::Unknown,
                                                                                                                                        flags:
                                                                                                                                            0u32,
                                                                                                                                        precision:
                                                                                                                                            ::std::fmt::rt::v1::Count::Is(2usize),
                                                                                                                                        width:
                                                                                                                                            ::std::fmt::rt::v1::Count::Implied,},},
                                                                        ::std::fmt::rt::v1::Argument{position:
                                                                                                         ::std::fmt::rt::v1::Position::At(1usize),
                                                                                                     format:
                                                                                                         ::std::fmt::rt::v1::FormatSpec{fill:
                                                                                                                                            ' ',
                                                                                                                                        align:
                                                                                                                                            ::std::fmt::rt::v1::Alignment::Unknown,
                                                                                                                                        flags:
                                                                                                                                            0u32,
                                                                                                                                        precision:
                                                                                                                                            ::std::fmt::rt::v1::Count::Implied,
                                                                                                                                        width:
                                                                                                                                            ::std::fmt::rt::v1::Count::Implied,},}];
                                                                  __STATIC_FMTARGS
                                                              }))
    }
}
