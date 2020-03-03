use crate::file_checker::{FileCheckResults, Mode, File};
use std::path::PathBuf;
use log::{info, debug, error};

pub struct Syncer {
    src_dir: String,
    dest_dir: String,
    op_count: usize,
    files: Option<FileCheckResults>
}

impl Syncer {
    pub fn new(src_dir: String, dest_dir: String, op_count: usize) -> Syncer {
        return Syncer {
            src_dir,
            dest_dir,
            op_count,
            files: None
        };
    }
}

impl Syncer {
    pub fn set_results(&mut self, results: FileCheckResults) {
        self.files = Some(results);
    }

    pub fn run(&mut self) {
        let mut files_option = None;
        std::mem::swap(&mut self.files, &mut files_option);
        let mut files = files_option.expect("No FileCheckResults in run");
        for _ in 0..self.op_count {
            if let Some((mode, file)) = files.next() {
                debug!("Processing {} for {}", mode, file.filename);
                match mode {
                    Mode::ADD => &self.add(file),
                    Mode::REMOVE => &self.remove(file),
                    Mode::UPDATE => &self.update(file),
                };
            } else {
                debug!("No more files to work on");
            }
        }
    }

    fn add(&mut self, file: File) {
        let mut from = PathBuf::from(&self.src_dir);
        let mut to = PathBuf::from(&self.dest_dir);
        from.push(&file.filename);
        to.push(&file.filename);

        if std::fs::metadata(&to).is_ok() {
            error!("Target file {} already exists!", file.filename);
            return
        }

        match std::fs::copy(from, to) {
            Ok(_) => info!("[ADD] Sync'd {}", file.filename),
            Err(err) => error!("Error sync'ing {}: {}", file.filename, err),
        }
    }

    fn remove(&mut self, file: File) {
        let mut target = PathBuf::from(&self.dest_dir);
        target.push(&file.filename);

        if std::fs::metadata(&target).is_err() {
            error!("Target file {} doesn't exist to delete!", file.filename);
            return
        }

        match std::fs::remove_file(target) {
            Ok(_) => info!("[DELETE] Removed {}", file.filename),
            Err(err) => error!("Error deleting {}: {}", file.filename, err),
        }
    }

    fn update(&mut self, file: File) {
        let mut from = PathBuf::from(&self.src_dir);
        let mut to = PathBuf::from(&self.dest_dir);
        from.push(&file.filename);
        to.push(&file.filename);

        if std::fs::metadata(&to).is_err() {
            error!("Target file {} doesn't exist to overwrite!", file.filename);
            return
        }

        match std::fs::copy(from, to) {
            Ok(_) => info!("[UPDATE] Sync'd {}", file.filename),
            Err(err) => error!("Error sync'ing {}: {}", file.filename, err),
        }
    }
}