//#![windows_subsystem = "windows"]

#[macro_use]
extern crate serde_derive;
extern crate glob;
extern crate open;
extern crate serde_json;
extern crate web_view;

use glob::glob;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;

static CACHE_FILENAME: &'static str = ".cache.json";

pub struct Cache<T> {
    data: Box<Vec<T>>,
}

impl<T> Cache<T> {
    pub fn new() -> Cache<T> {
        Cache {
            data: Box::new(vec![]),
        }
    }

    pub fn get_data_from_storage(&self) -> String {
        let mut file = match File::open(CACHE_FILENAME) {
            Ok(file) => file,
            Err(_) => {
                self.write(String::from(""));
                File::open(CACHE_FILENAME).expect("could not initialize cache")
            }
        };

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("something went wrong reading the file");

        contents
    }

    pub fn set_data(&mut self, data: Box<Vec<T>>) {
        self.data = data;
    }

    pub fn write(&self, data: String) {
        fs::write(CACHE_FILENAME, data).expect("could not write to cache");
    }
}

impl Cache<Movie> {
    pub fn initialize(&mut self) {
        let data = self.get_data_from_storage();

        // return a new vec if our cache is improper
        let movies: Box<Vec<Movie>> = match serde_json::from_str(&data) {
            Ok(data) => data,
            Err(_) => Box::new(vec![]),
        };

        self.set_data(movies);
    }

    pub fn serialize(&self) -> String {
        serde_json::to_string(&self.data).unwrap()
    }

    pub fn get_files(&mut self) -> &Vec<Movie> {
        // before returning files, update if it exists on the system
        for file in self.data.iter_mut() {
            let path = Path::new(&file.filepath);
            if !path.exists() {
                file.exists = false
            }
        }

        &self.data.sort_by(|a, b| a.filename.cmp(&b.filename));
        &self.data
    }

    pub fn insert(&mut self, movie: Movie) {
        // remove occurrence of same filename
        // ie. file.mp4 gets moved from USB A to USB B
        let index = self
            .data
            .iter()
            .position(|x| &x.filename == &movie.filename);

        if index.is_some() {
            self.data.remove(index.unwrap());
        }

        self.data.push(movie);
    }

    pub fn get_folders(&mut self) -> Vec<String> {
        let files = self.get_files();
        let mut folders = vec![];

        for file in files.iter() {
            if !folders.contains(&file.folder) {
                folders.push(file.folder.clone());
            }
        }

        folders
    }

    // clone here so we don't mutate our base cache
    pub fn search_files(&mut self, search: &str, folders: &Vec<String>) -> Vec<Movie> {
        let files = self.get_files().clone();
        let search = &search.to_lowercase();

        let found_files: Vec<Movie> = files
            .iter()
            .cloned()
            .filter(|x| x.filename.to_lowercase().contains(search))
            .collect();

        let mut found_folder_files = vec![];
        let no_filters_selected = folders.len() == 0;

        for file in found_files.iter().cloned() {
            let mut has_folder = false;

            for folder in folders.iter() {
                if file.folder == *folder {
                    has_folder = true;
                }
            }

            if no_filters_selected || has_folder {
                found_folder_files.push(file);
            }
        }

        found_folder_files
    }

    pub fn update_cache_from_directory(&mut self, path: &str, folder: &String) {
        for entry in glob(path).unwrap() {
            match entry {
                Ok(path) => {
                    let movie = Movie::new(path, folder);

                    self.insert(movie);
                }
                Err(e) => println!("{:?}", e),
            }
        }

        // update self with files found from current folder
        self.write(self.serialize());
    }
}

#[derive(Deserialize, Serialize, Debug, Eq, Ord, PartialEq, PartialOrd, Clone)]
pub struct Movie {
    filepath: String,
    filename: String,
    exists: bool,
    folder: String,
}

impl Movie {
    pub fn new(entry: PathBuf, folder: &String) -> Movie {
        let filepath = String::from(entry.to_str().unwrap());
        let filename = String::from(entry.file_name().unwrap().to_str().unwrap());
        let folder = folder.to_string();

        Movie {
            filepath,
            filename,
            folder,
            exists: true,
        }
    }

    pub fn play(&self) {
        if open::that(&self.filepath).is_ok() {
            println!("Opening file...");
        }
    }
}
