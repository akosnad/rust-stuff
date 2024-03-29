use std::process::Command;

fn main() {
    let git_output = Command::new("git").args(&["rev-parse", "HEAD"]).output().unwrap();
    let git_hash = String::from_utf8(git_output.stdout).unwrap();
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);

    let date_output = Command::new("git").args(&["show", "-s", "--format=%ci", "HEAD"]).output().unwrap();
    let date = String::from_utf8(date_output.stdout).unwrap();
    println!("cargo:rustc-env=GIT_HASH_DATE={}", date);
}