use std::path::PathBuf;
use clap::{AppSettings, Arg, Command};

fn main() {
    let matches = Command::new("n64-systemtest builder")
        .arg(Arg::new("ipl3")
            .takes_value(true)
            .long("ipl3")
            .help("Path to IPL3 binary file."))
        .arg(Arg::new("features")
            .takes_value(true)
            .long("features")
            .multiple_values(true)
            .allow_invalid_utf8(true))
        .global_setting(AppSettings::DeriveDisplayOrder)
        .next_line_help(true)
        .get_matches();
    
    let ipl3_path = matches.value_of("ipl3").unwrap_or("mini-ipl3/mini-ipl3.bin");
    let ipl3 = std::fs::read(ipl3_path).expect(&format!("IPL3 file doesn't exist: {}", ipl3_path));
    
    let mut features = matches.values_of_lossy("features").unwrap_or(vec!["default_tests".to_owned()]);
    features.insert(0, "--features".to_owned());
    let features = &features.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
    
    let elf = nust64::elf::Elf::build(&PathBuf::from("n64-systemtest"), Some(&features)).unwrap();
    let rom = nust64::rom::Rom::new(&elf, ipl3.try_into().expect("Failed to cast into array. Is input not exactly 4032 bytes?"), None);
    
    std::fs::write("n64-systemtest.n64", rom.to_vec()).unwrap();
}