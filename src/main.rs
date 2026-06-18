use clap::Parser;
use colored::Colorize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::{File, metadata};
use std::io::Read;
use std::process::exit;
use std::path::{Path, PathBuf};
use std::env::consts::OS;
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    file_name: PathBuf,

    ///Starts looking from system root
    //TODO: doesn't really work yet (no access to used processes on windows, not tested elsewhere)
    #[arg(long, short, default_value_t = false)]
    root: bool,
}

#[derive(Default)]
struct PathMap(HashMap<u64, Vec<PathBuf>>);

impl PathMap {
    fn new() -> Self {
        PathMap(HashMap::new())
    }
}

impl Display for PathMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Found {} matches", self.0.len().to_string().red())?;
        for v in self.0.values() {
            for path in v {
                let path_str = path.to_string_lossy();
                let mut parts: Vec<String> = path_str.split('\\').map(|s| s.to_string()).collect();

                if let Some(last) = parts.last_mut() {
                    *last = last.green().to_string();
                }

                writeln!(f, "{}", parts.join("/"))?;
            }
        }
        Ok(())
    }
}

fn main() {
    let mut data: PathMap = PathMap::new();
    let args = Args::parse();

    let file_metadata = metadata(&args.file_name).unwrap_or_else(|err| {
        eprintln!("{}: {}", "Error".red().bold(), err);
        exit(1);
    });
    let file_size = file_metadata.len();

    let file_4kb_checksum = get_checksum(&args.file_name, 4096).unwrap_or_else(|err| {
        eprintln!("{}: {}", "Error".red().bold(), err);
        exit(1);
    });

    let file_full_checksum = get_checksum(&args.file_name, 0).unwrap_or_else(|err| {
        eprintln!("{}: {}", "Error".red().bold(), err);
        exit(1);
    });

    let current_path = if args.root.eq(&true) {
        if OS.eq("windows") {
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

        if !path.is_file() {
            continue;
        }

        if file_size != path.metadata().unwrap().len() {
            continue;
        }

        let checksum_4kb = get_checksum(path, 4096).unwrap_or_else(|err| {
            eprintln!("{}", err);
            exit(1);
        });
        if checksum_4kb != file_4kb_checksum {
            continue;
        }

        let checksum_full = get_checksum(path, 0).unwrap_or_else(|err| {
            eprintln!("{}", err);
            exit(1);
        });
        if checksum_full != file_full_checksum {
            continue;
        }

        let key = data.0.len() as u64;
        data.0
            .entry(key)
            .or_insert_with(Vec::new)
            .push(path.to_path_buf());
    }

    println!("{}", data);
}

fn get_checksum(path: &Path, limit: u64) -> Result<String, String> {
    let file = File::open(path).map_err(|err| {
        format!("{}: {}", "Error".red().bold(), err)
    })?;

    let mut handle: Box<dyn Read> = if limit == 0 {
        Box::new(file)
    } else {
        Box::new(file.take(limit))
    };

    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];

    loop {
        match handle.read(&mut buffer) {
            Ok(0) => break,
            Ok(bytes_read) => hasher.update(&buffer[..bytes_read]),
            Err(err) => {
                return Err(format!("{}: {}", "Error".red().bold(), err));
            }
        }
    }

    Ok(hex::encode(hasher.finalize()))
}
