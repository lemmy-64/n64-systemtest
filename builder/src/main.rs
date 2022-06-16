use std::path::PathBuf;

fn main() {
    let elf = nust64::elf::Elf::build(&PathBuf::from("n64-systemtest"), Some(&["--features", "default_tests"])).unwrap();
    let rom = nust64::rom::Rom::new(&elf, include_bytes!("../../mini-ipl3/mini-ipl3.bin").to_owned(), None);
    
    std::fs::write("n64-systemtest.n64", rom.to_vec()).unwrap();
}