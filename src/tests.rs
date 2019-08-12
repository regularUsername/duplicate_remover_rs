
use super::*;

#[test]
fn test_select_files1() {
    let a = PathBuf::from("12345.bin");
    let b = PathBuf::from("3898d553.bin");
    let (x, _) = select_files(&[&a, &b]);
    assert_eq!(&b, x);
}

#[test]
fn test_select_files2() {
    let a = PathBuf::from("we1223ffqwe21.bin");
    let b = PathBuf::from("3898d553.bin");
    let c = PathBuf::from("3898d553.bin");
    let (x, _) = select_files(&[&a, &b, &c]);
    assert_eq!(&a, x);
}

#[test]
fn test_select_files3() {
    let a = PathBuf::from("12351235.bin");
    let b = PathBuf::from("512363453534.bin");
    let (x, _) = select_files(&[&a, &b]);
    assert_eq!(&b, x);
}

#[test]
fn test_select_files4() {
    let a = PathBuf::from("5123.bin");
    let b = PathBuf::from("12351235.bin");
    let c = PathBuf::from("12351235(1).bin");
    let (x, _) = select_files(&[&a, &b, &c]);
    assert_eq!(&b, x);
}

#[test]
fn test_select_files5() {
    let a = PathBuf::from("blabla.bin");
    let b = PathBuf::from("blablablabla foobar.bin");
    let c = PathBuf::from("bla bla foo bar.bin");
    let (keep, _) = select_files(&[&a, &b, &c]);
    assert_eq!(&c, keep);
}

#[test]
fn test_select_files6() {
    let a = PathBuf::from("12351235.bin");
    let b = PathBuf::from("12351235(1).bin");
    let c = PathBuf::from("12351235(1)(1).bin");
    let (keep, _) = select_files(&[&a, &b, &c]);
    assert_eq!(&a, keep);
}
