use std::path::PathBuf;
use std::process::{Command, Stdio};
use clap::{AppSettings, Arg};

const ROM_TITLE: &'static str = "n64-systemtest";
const FILE_NAME: &'static str = "n64-systemtest.n64";

fn main() {
    let matches = clap::Command::new("n64-systemtest builder")
        .arg(Arg::new("ipl3")
            .takes_value(true)
            .long("ipl3")
            .help("Path to IPL3 binary file."))
        .arg(Arg::new("features")
            .takes_value(true)
            .long("features")
            .help("Specify list of feature flags which enable different tests. Multiple values allowed.")
            .multiple_values(true)
            .allow_invalid_utf8(true))
        .arg(Arg::new("unfloader")
            .long("unf")
            .help("Will upload with UNFLoader using successfully built rom."))
        .arg(Arg::new("usb64")
            .long("usb64")
            .help("Will upload with usb64 using successfully built rom."))
        .global_setting(AppSettings::DeriveDisplayOrder)
        .next_line_help(true)
        .get_matches();
    
    let ipl3_path = matches.value_of("ipl3").unwrap_or("mini-ipl3/mini-ipl3.bin");
    let ipl3 = std::fs::read(ipl3_path).expect(&format!("IPL3 file doesn't exist: {}", ipl3_path));
    
    let mut features = matches.values_of_lossy("features").unwrap_or(vec!["default_tests".to_owned()]);
    features.insert(0, "--features".to_owned());
    let features = &features.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
    
    let elf = nust64::elf::Elf::build(&PathBuf::from("n64-systemtest"), Some(&features)).unwrap();
    let rom = nust64::rom::Rom::new(&elf, ipl3.try_into().expect("Failed to cast into array. Is input not exactly 4032 bytes?"), Some(ROM_TITLE));
    
    let outpath = PathBuf::from(FILE_NAME);
    match std::fs::write(&outpath, rom.to_vec()) {
        Ok(_) => println!("Rom successfully compiled: {}", outpath.canonicalize().unwrap_or(outpath).display()),
        Err(err) => panic!("Unable to save rom file: {}", err)
    }
    
    
    if matches.is_present("unfloader") {
        Command::new("UNFLoader")
            .args(["-r", FILE_NAME])
            .stderr(Stdio::inherit())
            .status()
            .expect("Failed to execute UNFLoader");
    }
    
    if matches.is_present("usb64") {
        Command::new("usb64")
            .args([&format!("-rom={}", FILE_NAME), "-start"])
            .stderr(Stdio::inherit())
            .status()
            .expect("Failed to execute UNFLoader");
    }
}