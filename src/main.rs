use colored::Colorize;
use std::collections::HashMap;
use std::env;
use std::env::set_current_dir;
use std::fs::read_dir;
use std::path::{Component, PathBuf};

#[derive(Debug, PartialEq)]
enum Ftype {
    File,
    Symlink,
    Directory,
}

struct Traverser {
    start_dir: PathBuf,
    num_files: usize,
    num_dirs: usize,
    num_symlinks: usize,
    finished_dirs: HashMap<PathBuf, bool>,
}

const INTER_LEVEL_INDENT: u8 = 3;

impl Traverser {
    fn new(start: &str) -> Self {
        if let Err(err) = set_current_dir(start) {
            panic!("Couldn't change dir to {}: {}", start, err);
        }

        Traverser {
            start_dir: PathBuf::from(start),
            num_files: 0,
            num_dirs: 0,
            num_symlinks: 0,
            finished_dirs: HashMap::new(),
        }
    }

    fn print_ascii_row(&self, depth: usize, entry: &PathBuf, is_last_entry: bool) {
        let mut sym;
        let mut indent;
        let mut lookup_path: PathBuf = PathBuf::from(".");
        // Each step of the loop build up the path starting from ".". This
        // becomes our lookup key in each iteration.
        for (i, component) in entry.components().enumerate() {
            if i > depth {
                break;
            }

            lookup_path = match component {
                Component::Normal(dir) => lookup_path.join(dir),
                // if the element is ".", return the current lookup. this
                // should happen only during the first iteration.
                Component::CurDir => lookup_path,
                _ => panic!("Unexpected type of component: {:?}", component),
            };

            if i == 0 {
                indent = 0;
            } else {
                indent = INTER_LEVEL_INDENT;
            }
            if i == depth {
                if is_last_entry {
                    sym = "└──";
                } else {
                    sym = "├──";
                }
            } else if self.finished_dirs.get(&lookup_path) == Some(&true) {
                // No | if the directory is finished already
                sym = " ";
            } else {
                sym = "│";
            }
            print!("{}{}", " ".repeat(indent as usize), sym);
        }
    }

    fn get_ftype(&self, path: &PathBuf) -> Result<Ftype, std::io::Error> {
        let x = match (path.is_file(), path.is_symlink()) {
            (true, false) => Ok(Ftype::File),
            (false, true) => Ok(Ftype::Symlink),
            (_, _) => Ok(Ftype::Directory),
        };

        return x;
    }

    fn basename(&self, path: &PathBuf) -> String {
        path.file_name()
            .expect(&format!("Error getting file_name from '{:?}'", path))
            .to_str()
            .expect(&format!("Could not convert '{:?}' to string", path))
            .to_owned()
    }

    fn traverse(&mut self) {
        if let Err(msg) = self._traverse(&PathBuf::from("."), 0) {
            println!("Error: {}", msg);
        }
        self.stats();
    }

    fn _traverse(&mut self, path: &PathBuf, depth: usize) -> Result<(), std::io::Error> {
        let mut entries: Vec<PathBuf> = read_dir(&path)?.map(|x| x.unwrap().path()).collect();
        entries.sort_by_key(|entry| self.basename(entry));
        let num_entries = entries.len();

        // Print the starting directory (special case)
        if depth == 0 {
            println!("{}", self.start_dir.to_str().unwrap().bright_blue());
            // println!("{}", self.basename(&self.start_dir).bright_blue());
        }

        for (index, entry) in entries.into_iter().enumerate() {
            let ftype = self.get_ftype(&entry)?;
            match ftype {
                Ftype::File => self.num_files += 1,
                Ftype::Symlink => self.num_symlinks += 1,
                Ftype::Directory => self.num_dirs += 1,
            };
            let is_last_entry = index + 1 == num_entries;
            if is_last_entry == true {
                // println!("last entry: {:?} depth={}", entry, depth);
                if depth == 0 {
                    // Mark finishing the top directory
                    self.finished_dirs.insert(PathBuf::from("."), true);
                } else {
                    // println!("recording {:?}", entry.parent().unwrap());
                    self.finished_dirs
                        .insert(PathBuf::from(entry.parent().unwrap()), true);
                }
            }
            if ftype == Ftype::Directory {
                if entry.as_path().to_str().unwrap().starts_with("./.") {
                    continue;
                }
                self.print_ascii_row(depth, &entry, is_last_entry);
                println!(" {}", self.basename(&entry).bright_blue(),);
                if let Err(err) = self._traverse(&entry, depth + 1) {
                    return Err(err);
                }
            } else {
                self.print_ascii_row(depth, &entry, is_last_entry);
                println!(" {}", entry.file_name().unwrap().to_str().unwrap(),);
            }
        }

        Ok(())
    }

    fn stats(&self) {
        println!(
            "\ndirs={} files={} symlinks={}",
            self.num_dirs, self.num_files, self.num_symlinks
        );
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let dir = args.get(1).expect("Must have a dir path");
    let mut x = Traverser::new(dir);
    x.traverse();
    // dbg!(&x.finished_dirs);
}
