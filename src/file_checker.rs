use std::fs::{read_dir, ReadDir, metadata};
use crate::Error;
use std::path::PathBuf;
use regex::Regex;
use std::error::Error as StdError;
use std::fmt::{Display, Formatter};
use std::fmt;
use log::{trace, debug, error};
use crate::file_checker::Mode::{UPDATE, ADD, REMOVE};

pub struct FileChecker {
    src_dir: String,
    dest_dir: String,
    include: Vec<Regex>,
    exclude: Vec<Regex>
}

impl FileChecker {
    pub fn new(src_dir: String, dest_dir: String, include: Vec<Regex>, exclude: Vec<Regex>) -> FileChecker {
        return FileChecker {
            src_dir,
            dest_dir,
            include,
            exclude
        };
    }
}

impl FileChecker {
    pub fn get_list_of_files(&self) -> Result<FileCheckResults, Error> {
        let src_files = read_dir(self.src_dir.clone());
        let dest_files = read_dir(self.dest_dir.clone());

        if src_files.is_ok() && dest_files.is_ok() {
            trace!("Src dir");
            let (src_files, src_errors) = self.convert_dir_entries(src_files.unwrap());
            trace!("Dest dir");
            let (dest_files, dest_errors) = self.convert_dir_entries(dest_files.unwrap());

            let mut error_text = String::new();

            if src_files.is_empty() && !src_errors.is_empty() {
                error_text = format!("Source dir: \n{}", src_errors.join("\n"));
            }

            if dest_files.is_empty() && !dest_errors.is_empty() {
                error_text = format!("{}\n\nDestination dir: \n{}", error_text, src_errors.join("\n"));
            }

            if !error_text.is_empty() {
                return Err(Error::from(error_text));
            }

            let mut in_src_but_not_dest = vec![];
            let mut not_in_src_but_in_dest = vec![];
            let mut diff_in_src_and_dest = vec![];

            let mut dest_files_to_ignore = vec![];

            for (spath, sfilename) in &src_files {
                let mut found = false;
                for (idx, (dpath, dfilename)) in dest_files.iter().enumerate() {
                    if sfilename == dfilename {
                        dest_files_to_ignore.push(idx);
                        found = true;
                        let src_metadata = metadata(spath.clone());
                        let dest_metadata = metadata(dpath.clone());
                        if src_metadata.is_ok() && dest_metadata.is_ok() {
                            let src_len = src_metadata.unwrap().len();
                            let dest_len = dest_metadata.unwrap().len();
                            if src_len != dest_len {
                                debug!("{} has changed", sfilename);
                                diff_in_src_and_dest.push(ChangedFile {
                                    file: File::new(spath.clone(), sfilename.clone()),
                                    old_size: src_len,
                                    new_size: dest_len,
                                });
                            } else {
                                debug!("{} are the same", sfilename);
                            }
                        } else {
                            error!("Error checking len of {}, src: {:?}, dest: {:?}", sfilename, src_metadata.err(), dest_metadata.err());
                        }
                    }
                }
                if !found {
                    in_src_but_not_dest.push(File::new(spath.clone(), sfilename.clone()));
                }
            }

            for (idx, (dpath, dfilename)) in dest_files.iter().enumerate() {
                if !dest_files_to_ignore.contains(&idx) {
                    let found = &src_files.iter().any(|(_, sfilename)| sfilename == dfilename);
                    if !found {
                        not_in_src_but_in_dest.push(File::new(dpath.clone(), dfilename.clone()));
                    }
                } else {
                    trace!("Ignoring dest file {}, as already checked in src", dfilename);
                }
            }

            return Ok(FileCheckResults {
                to_add: in_src_but_not_dest,
                to_delete: not_in_src_but_in_dest,
                to_rewrite: diff_in_src_and_dest,
            });
        } else {
            let mut error_text = String::new();

            if let Err(err) = src_files {
                error_text = format!("Source: {}", err.description());
            }

            if let Err(err) = dest_files {
                error_text = format!("{}\nDestination: {}", error_text, err.description());
            }

            return Err(Error::from(error_text));
        }
    }

    fn convert_dir_entries(&self, dir: ReadDir) -> (Vec<(PathBuf, String)>, Vec<String>) {
        let mut files = vec![];
        let mut errors = vec![];

        for file in dir {
            match file {
                Ok(entry) => {
                    files.push(entry.path());
                }
                Err(err) => {
                    errors.push(format!("{:?}: {}", err.kind(), err.description()));
                }
            }
        }

        trace!("Before filtering: {} files/dirs", files.len());
        let filtered = self.filter_invalid(files);
        trace!("After filtering: {} files/dirs", filtered.len());
        return (filtered, errors);
    }

    fn filter_invalid(&self, entries: Vec<PathBuf>) -> Vec<(PathBuf, String)> {
        return entries.iter()
            .filter(|entry| entry.is_file())
            .filter(|entry| entry.file_name().is_some())
            .filter_map(|entry| entry.file_name().unwrap().to_str().map(|name| (entry.clone(), name.to_string())))
            .filter(|(_, name)| !name.starts_with("."))
            .filter(|(_, name)| {
                self.include.iter().all(|pattern| pattern.is_match(name)) &&
                    self.exclude.iter().all(|pattern| !pattern.is_match(name))
            })
            .collect();
    }
}

pub struct FileCheckResults {
    to_add: Vec<File>,
    to_delete: Vec<File>,
    to_rewrite: Vec<ChangedFile>,
}

impl Display for FileCheckResults {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let to_add = self.to_add.print();
        let to_delete = self.to_delete.print();
        let to_update = self.to_rewrite.print();
        write!(f, "Pending updates:\nTo add:\n{}\n\nTo delete:\n{}\n\nTo rewrite:\n{}", to_add, to_delete, to_update)
    }
}

trait Printer {
    fn print(&self) -> String;
}

impl Printer for Vec<File> {
    fn print(&self) -> String {
        return if self.is_empty() {
            String::from("None")
        } else {
            self.iter().map(|file| file.filename.clone()).collect::<Vec<String>>().join("\n")
        };
    }
}

impl Printer for Vec<ChangedFile> {
    fn print(&self) -> String {
        return if self.is_empty() {
            String::from("None")
        } else {
            self.iter().map(|file| file.file.filename.clone()).collect::<Vec<String>>().join("\n")
        };
    }
}

impl FileCheckResults {
    pub fn has_any_operations(&self) -> bool {
        return !self.to_add.is_empty() || !self.to_delete.is_empty() || !self.to_rewrite.is_empty();
    }

    pub fn next(&mut self) -> Option<(Mode, File)> {
        return if !self.to_rewrite.is_empty() {
            self.to_rewrite.pop().and_then(|file| Some((UPDATE, file.file)))
        } else if !self.to_add.is_empty() {
            self.to_add.pop().and_then(|file| Some((ADD, file)))
        } else if !self.to_delete.is_empty() {
            self.to_delete.pop().and_then(|file| Some((REMOVE, file)))
        } else {
            None
        }
    }
}

pub enum Mode {
    ADD, REMOVE, UPDATE
}

impl Display for Mode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self {
            ADD => write!(f, "ADD"),
            REMOVE => write!(f, "REMOVE"),
            UPDATE => write!(f, "UPDATE"),
        }
    }
}

struct ChangedFile {
    file: File,
    old_size: u64,
    new_size: u64,
}

pub struct File {
    path: PathBuf,
    pub filename: String,
}

impl File {
    fn new(path: PathBuf, filename: String) -> File {
        return File {
            path,
            filename,
        };
    }
}