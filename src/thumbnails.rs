use threadpool::ThreadPool;
use std::path::PathBuf;

use crate::entities::Entity;

const THUMBNAIL_RETRIEVAL_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);

pub struct ThumbnailRetriever {
    handle: ThreadPool,
    folder: PathBuf,
    verbose: bool,
}

impl ThumbnailRetriever {
    pub fn new(folder: impl AsRef<std::path::Path>, verbose: bool) -> Self {
        let folder = folder.as_ref().to_path_buf();
        if !(folder.exists() && folder.is_dir()) {
            std::fs::create_dir_all(&folder).expect("Invalid thumbnail folder");
        }
        Self {
            handle: threadpool::Builder::new()
                //.num_threads(42) // default to number of CPUs on the current system
                .thread_name("thumbnail.retriever.x".to_string())
                .build(),
            folder,
            verbose,
        }
    }

    pub fn retrieve(&self, metadata: &crate::DbMetaData) {
        let url = metadata.thumbnail.clone();
        let filename = format!("{} - {}.jpg",
                        metadata.id,
                        metadata.name.chars().filter(|c| c.is_ascii_alphanumeric() || c == &' ').collect::<String>(),
                    );
        let save_path = self.folder.join(filename);
        let verbose = self.verbose;
        self.handle.execute(move || retrieve_thumbnail(url, save_path, verbose));
    }

    pub fn retrieve_all_known(&self, db: &mut rusqlite::Connection) {
        let mut ret_statement = db.prepare("SELECT * from ROBOT_METADATA rm ORDER BY rm.id DESC").unwrap();
        let iter = ret_statement.query_map([], crate::DbMetaData::map_row).unwrap();
        for meta in iter {
            self.retrieve(&meta.unwrap());
        }
        if self.verbose {
            println!("Handling {} thumbnail downloads (in progress: {})", self.handle.queued_count(), self.handle.active_count());
        }
    }

    pub fn finalize(self) {
        if self.verbose {
            println!("Waiting for remaining thumbnail downloads: {} (in progress: {})", self.handle.queued_count(), self.handle.active_count());
        }
        self.handle.join();
    }
}

fn retrieve_thumbnail(url: String, path: PathBuf, verbose: bool) {
    let response = ureq::get(&url)
        .timeout(THUMBNAIL_RETRIEVAL_TIMEOUT)
        .call();
    match response {
        Err(e) => if verbose {
            eprintln!("failed to retrieve thumbnail {} (url: {}): {}", path.display(), url, e);
        },
        Ok(resp) => {
            let mut body = Vec::new(); // should be a retrieved image (jpg)
            if let Err(e) = resp.into_reader().read_to_end(&mut body) {
                if verbose {
                    eprintln!("failed to download thumbnail {} (url: {}): {}", path.display(), url, e);
                }
                return;
            }
            if let Err(e) = std::fs::write(&path, body) {
                if verbose {
                    eprintln!("failed to save thumbnail {} (url: {}): {}", path.display(), url, e);
                }
                return;
            }
        }
    }
}
