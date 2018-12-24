//#![windows_subsystem = "windows"]

#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate glob;
extern crate open;
extern crate web_view;

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
        println!("{}", CACHE_FILENAME);
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

    pub fn search_files(&mut self, search: &str, folders: &Vec<String>) -> Vec<&Movie> {
        let files = self.get_files();
        let search = &search.to_lowercase();

        let mut found_folder_files = vec![];

        for file in files.into_iter() {
            if folders.len() == 0 {
                found_folder_files.push(file);
            }

            for folder in folders.iter() {
                if file.folder == *folder {
                    found_folder_files.push(file);
                }
            }
        }

        let mut found_files = vec![];

        for file in found_folder_files.into_iter() {
            if file.filename.to_lowercase().contains(search) {
                found_files.push(file);
            }
        }

        found_files
    }
}

#[derive(Deserialize, Serialize, Debug, Eq, Ord, PartialEq, PartialOrd)]
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
