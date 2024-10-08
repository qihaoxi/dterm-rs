// build.rs
use std::process::Command;
fn main() {
    // note: add error checking yourself.
    let output = Command::new("git").args(&["rev-parse", "HEAD"]).output().unwrap();
    let git_hash = String::from_utf8(output.stdout).unwrap();
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);

    // add build date
    let output = Command::new("date").output().unwrap();
    let date = String::from_utf8(output.stdout).unwrap();
    println!("cargo:rustc-env=BUILD_DATE={}", date);
}
