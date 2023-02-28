use std::path::PathBuf;
use std::fs::read_dir;

#[derive(Debug, PartialEq)]
enum Ftype {
    File,
    Symlink,
    Directory,
}

struct Traverser {
    num_files: usize,
    num_dirs: usize,
    num_symlinks: usize,
}

impl Traverser {
    fn new() -> Self {
        Traverser {
            num_files: 0,
            num_dirs: 0,
            num_symlinks: 0,
        }
    }

    fn get_ftype(&self, dir_entry: &std::fs::DirEntry) -> Result<Ftype, std::io::Error> {
        let file_type = dir_entry.file_type()?;
        let x = match (file_type.is_file(), file_type.is_symlink()) {
            (true, false) => Ok(Ftype::File),
            (false, true) => Ok(Ftype::Symlink),
            (_, _) => Ok(Ftype::Directory),
        };

        return x;
    }

    fn traverse(&mut self, path: PathBuf, depth: usize) -> Result<(), std::io::Error> {
        for wrapped_dir_entry in read_dir(path)? {
            let dir_entry = wrapped_dir_entry?;
            let ftype = self.get_ftype(&dir_entry)?;
            let indent = "=".repeat(depth);
            match ftype {
                Ftype::File => self.num_files += 1,
                Ftype::Symlink => self.num_symlinks += 1,
                Ftype::Directory => self.num_dirs += 1,
            };
            if ftype == Ftype::Directory {
                self.num_dirs += 1;
                println!("{}> dir={:?}", indent, dir_entry.path());
                if let Err(err) = self.traverse(dir_entry.path(), depth+1) {
                    return Err(err);
                }
            } else {
                self.num_dirs += 1;
                let indent = " ".repeat(depth + 1);
                println!(
                    "{} {:?}:{:?}",
                    indent,
                    ftype,
                    dir_entry.path().file_name().unwrap()
                );
            }
        }

        Ok(())
    }

    fn stats(&self) {
        println!(
            "dirs={} files={} symlinks={}",
            self.num_dirs, self.num_files, self.num_symlinks
        );
    }
}

fn main() {
    let mut traverser = Traverser::new();
    if let Err(msg) = traverser.traverse(PathBuf::from("."), 1) {
        println!("Error: {}", msg);
    }
    traverser.stats();
}
