use clap::Parser;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    file_name: PathBuf,

    ///Starts looking from system root
    #[arg(long, short, default_value_t = false)]
    root: bool,
}

impl Display for HashMap<u64, Vec<PathBuf>>{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

fn main() {
    let file = Args::parse();

    let mut data: HashMap<u64, Vec<PathBuf>> = HashMap::new();

    let file_checksum = get_4kb_checksum(&file.file_name).unwrap();
    let current_path = if file.root.eq(&true) {

        //TODO: doesn't really work yet (no access to used processes on windows, not tested elsewhere)
        if std::env::consts::OS.eq("windows") {
            "C:\\"
        } else {
            "/"
        }
    } else {
        "."
    };

    for entry in WalkDir::new(current_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if path.is_file() {
            match get_4kb_checksum(path) {
                Ok(value) => match compare_checksum(value, &file_checksum) {
                    Ok(true) => {
                        let key = data.len() as u64;
                        data.entry(key)
                            .or_insert_with(Vec::new)
                            .push(path.to_path_buf());
                    }
                    Ok(false) => continue,
                    Err(_) => panic!("Err"),
                },
                Err(_) => panic!("Err"),
            };
        }
    }

    println!("{}", data);
}

fn get_4kb_checksum(path: &Path) -> Result<String, String> {
    let file = File::open(path).expect("Err");

    let mut handle = file.take(4096);

    let mut buffer = Vec::new();
    handle.read_to_end(&mut buffer).expect("Err");

    let mut hasher = Sha256::new();
    hasher.update(&buffer);

    //TODO: change lossy
    Ok(String::from_utf8_lossy(&hasher.finalize()).into_owned())
}

//TODO: compare a) file size b) the rest of checksum

fn compare_checksum(checksum: String, target_checksum: &String) -> Result<bool, &str> {
    let mut is_equal = false;
    if checksum.eq(target_checksum) {
        is_equal = true;
        return Ok(is_equal);
    }
    Ok(is_equal)
}
