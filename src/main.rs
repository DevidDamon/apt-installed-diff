#![allow(non_upper_case_globals)]
use std::fs::{remove_file, rename, OpenOptions, File};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::Command;
use std::str::{self, Utf8Error};
use structopt::StructOpt;

type Result<T> = std::result::Result<T, ScriptError>;

const olddbfname: &str = "old-apt-list";
const newdbfname: &str = "new-apt-list";
const oldnewdiff: &str = "old-new-diff";

#[derive(thiserror::Error, Debug)]
pub enum ScriptError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("Type convert {0}")]
    Convert(#[from] Utf8Error),
    #[error("Unknown error")]
    Unknown,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "BlaBlaDiff", about = "Show package diff after last system update")]
struct AppOpt {
    /// Clean temp files
    #[structopt(short = "c", long = "clean")]
    clean: bool,

    /// Last result
    #[structopt(short)]
    last: bool,
}

fn main() -> Result<()> {
    let AppOpt { clean, last } = AppOpt::from_args();

    if clean {
        println!("[Start] clean working file");
        if let Ok(..) = clean_files(&vec![olddbfname, newdbfname, oldnewdiff]) {
            println!("[Done]");
        }
        return Ok(());
    }

    if last {
        if Path::new(oldnewdiff).exists() {
            let f = File::open(oldnewdiff)?;
            let mut reader = BufReader::new(f).lines();

            while let Some(Ok(line)) = reader.next() {
                println!("{}", line);
            }
        };
        return Ok(());
    }

    let r = Command::new("apt").arg("list").output().expect("fail");
    let r = str::from_utf8(&r.stdout).map_err(ScriptError::Convert)?;

    let mut newdbfile = OpenOptions::new()
        .create(true)
        .write(true)
        .open(&newdbfname)
        .map_err(ScriptError::Io)?;

    write!(newdbfile, "{}", r).map_err(ScriptError::Io)?;

    match Path::new(olddbfname).exists() {
        true => {
            let output = Command::new("diff")
                .arg(olddbfname)
                .arg(newdbfname)
                .arg("--unified=0")
                .output()
                .expect("fail");
            let output = str::from_utf8(&output.stdout).map_err(ScriptError::Convert)?;

            if output.len() > 0 {
                println!("[Diff] Changes");
                println!("{}", &output);
                let mut difffile = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .open(&oldnewdiff)
                    .map_err(ScriptError::Io)?;
                let _ = write!(difffile, "{}", output);
            } else {
                println!("[Done] No changes in apt list");
                if Path::new(oldnewdiff).exists() {
                    println!("Last diff in `{oldnewdiff}` file");
                }
            }

            rename(newdbfname, olddbfname)?;
        }
        false => {
            rename(newdbfname, olddbfname)?;
            println!("[Done] Result was recorded to {newdbfname}");
        }
    }

    Ok(())
}

fn clean_files<T>(files: &[T]) -> Result<()>
where
    T: AsRef<Path> + std::fmt::Display,
{
    for file in files {
        match remove_file(file).map_err(ScriptError::Io) {
            Ok(_) => continue,
            Err(e) => eprintln!("{} {:?}", file, e.to_string())
        }
    }

    Ok(())
}
