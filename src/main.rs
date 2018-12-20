extern crate glob;
extern crate open;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate web_view;

use glob::glob;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;
use web_view::*;

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "_type")]
enum Action {
    OpenFolder,
    Search { keyword: String },
    Play { movie: Movie },
    ClickFolder { folder: String },
    UnclickFolder { folder: String },
}

struct Cache<T> {
    data: Box<Vec<T>>,
}

impl<T> Cache<T> {
    fn new() -> Cache<T> {
        Cache {
            data: Box::new(vec![]),
        }
    }

    fn get_data_from_storage(&self) -> String {
        let mut file = File::open(".cache").expect("file not found");

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("something went wrong reading the file");

        // println!("cache contains: {:?}", contents);

        contents
    }

    fn set_data(&mut self, data: Box<Vec<T>>) {
        self.data = data;
    }

    fn write(&self, data: String) {
        fs::write(".cache", data).expect("could not write to cache");
    }
}

impl Cache<Movie> {
    fn initialize(&mut self) {
        let data = self.get_data_from_storage();

        let movies: Box<Vec<Movie>> = match serde_json::from_str(&data) {
            Ok(data) => data,
            Err(_) => Box::new(vec![]),
        };

        self.set_data(movies);
    }

    fn serialize(&self) -> String {
        serde_json::to_string(&self.data).unwrap()
    }

    fn get_files(&mut self) -> &Vec<Movie> {
        for file in self.data.iter_mut() {
            let path = Path::new(&file.filepath);
            if !path.exists() {
                file.exists = false
            }
        }

        &self.data.sort_by(|a, b| a.filename.cmp(&b.filename));
        &self.data
    }

    fn insert(&mut self, movie: Movie) {
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

    fn get_folders(&mut self) -> Vec<String> {
        let files = self.get_files();
        let mut folders = vec![];

        for file in files.iter() {
            if !folders.contains(&file.folder) {
                folders.push(file.folder.clone());
            }
        }

        folders
    }

    // make trait for iteratable for Movie
    fn search_files(&mut self, search: &str) -> Vec<&Movie> {
        let files = self.get_files();
        let search = &search.to_lowercase();
        let mut found_files = vec![];

        for file in files.into_iter() {
            if file.filename.to_lowercase().contains(search) {
                found_files.push(file);
            }
        }

        found_files
    }
}

#[derive(Serialize, Debug)]
struct State {
    chosen_folders: Vec<String>,
    search_keyword: String,
}

impl State {
    fn new() -> State {
        State {
            chosen_folders: vec![],
            search_keyword: String::from(""),
        }
    }

    fn get_folders(&self) -> &Vec<String> {
        &self.chosen_folders
    }

    fn add_folder(&mut self, folder: String) {
        if !self.chosen_folders.contains(&folder) {
            self.chosen_folders.push(folder);
        }
    }

    fn remove_folder(&mut self, folder: String) {
        let index = self.chosen_folders.iter().position(|x| *x == folder);

        if index.is_some() {
            self.chosen_folders.remove(index.unwrap());
        }
    }

    fn update_keyword(&mut self, keyword: &str) {
        self.search_keyword = keyword.to_string();
    }
}

fn main() {
    let mut state = State::new();
    let mut cache: Cache<Movie> = Cache::new();
    cache.initialize();

    // println!("data: {:?}", cache.data);

    let webview = web_view::builder()
        .title("Movie Manager")
        .content(Content::Html(create_html()))
        .size(800, 800)
        .resizable(true)
        .debug(true)
        .user_data(())
        .invoke_handler(|webview, arg| {
            match serde_json::from_str(arg) {
                Ok(Action::OpenFolder) => match webview
                    .dialog()
                    .choose_directory("Please choose a folder...", "")?
                {
                    Some(path) => {
                        let cloned_path = path.clone();
                        let folder =
                            String::from(cloned_path.file_name().unwrap().to_str().unwrap());

                        let mut path = path.into_os_string().into_string().unwrap();
                        println!("{:?}", path);
                        &path.push_str("/*.mp4");

                        for entry in glob(&path).unwrap() {
                            let movie = Movie::new(entry.unwrap(), &folder);

                            cache.insert(movie);
                        }

                        // update cache with files found from current folder
                        cache.write(cache.serialize());

                        send_to_ui(
                            webview,
                            &ToUiCommand::OpenFolder {
                                movies: &cache.get_files(),
                            },
                        );

                        send_to_ui(
                            webview,
                            &ToUiCommand::Folders {
                                folders: &cache.get_folders(),
                            },
                        );

                        send_to_ui(
                            webview,
                            &ToUiCommand::ChosenFolders {
                                chosen_folders: &state.get_folders(),
                            },
                        );
                    }
                    None => println!("Cancelled opening folder"),
                },
                Ok(Action::Search { keyword }) => {
                    state.update_keyword(&keyword);
                    send_to_ui(
                        webview,
                        &ToUiCommand::Search {
                            movies: &cache.search_files(&keyword),
                        },
                    );
                    send_to_ui(
                        webview,
                        &ToUiCommand::Folders {
                            folders: &cache.get_folders(),
                        },
                    );
                    send_to_ui(
                        webview,
                        &ToUiCommand::ChosenFolders {
                            chosen_folders: &state.get_folders(),
                        },
                    );
                }
                Ok(Action::Play { movie }) => {
                    movie.play();
                }
                Ok(Action::ClickFolder { folder }) => {
                    state.add_folder(folder);

                    send_to_ui(
                        webview,
                        &ToUiCommand::Search {
                            movies: &cache.search_files(&state.search_keyword),
                        },
                    );

                    send_to_ui(
                        webview,
                        &ToUiCommand::ChosenFolders {
                            chosen_folders: &state.get_folders(),
                        },
                    );
                }
                Ok(Action::UnclickFolder { folder }) => {
                    state.remove_folder(folder);

                    send_to_ui(
                        webview,
                        &ToUiCommand::Search {
                            movies: &cache.search_files(&state.search_keyword),
                        },
                    );

                    send_to_ui(
                        webview,
                        &ToUiCommand::ChosenFolders {
                            chosen_folders: &state.get_folders(),
                        },
                    );
                }
                Err(error) => println!("Unable to parse [{}] because {}", arg, error),
            }
            Ok(())
        }).build()
        .unwrap();

    webview.run().unwrap();
}

#[derive(Deserialize, Serialize, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Movie {
    filepath: String,
    filename: String,
    exists: bool,
    folder: String,
}

impl Movie {
    fn new(entry: PathBuf, folder: &String) -> Movie {
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

    fn play(&self) {
        if open::that(&self.filepath).is_ok() {
            println!("Opening file...");
        }
    }
}

#[derive(Serialize, Debug)]
#[serde(tag = "data")]
pub enum ToUiCommand<'a, 'b> {
    Folders { folders: &'b Vec<String> },
    OpenFolder { movies: &'a Vec<Movie> },
    Search { movies: &'a Vec<&'a Movie> },
    ChosenFolders { chosen_folders: &'b Vec<String> },
}

pub fn send_to_ui<'a, S, T>(webview: &mut WebView<'a, T>, data: &S)
where
    S: serde::ser::Serialize,
{
    let data2 = serde_json::to_string(data);
    println!("data: {:?}", data2);
    match serde_json::to_string(data) {
        Ok(json) => match webview.eval(&format!("toFrontEnd({})", json)) {
            Ok(_) => println!("Sent to UI"),
            Err(error) => println!("failed to send to ui because {}", error),
        },
        Err(error) => println!("failed to serialize for ui because {}", error),
    };
}

fn create_html() -> String {
    format!(
        r#"
    <!DOCTYPE html>
    <html>
    <head>
        <meta charset="utf-8">
        <meta name="viewport" content="width=device-width">
        <style>{bulma}</style>
        <style>{customCss}</style>
    </head>
    <body>
        <div id="view"></div>
        <script>
            {elmJs}
            {portsJs}
            {fontAwesome}
        </script> 
        
    </body>
    </html>
    "#,
        elmJs = include_str!("/Users/gperlman/Documents/side/rust/projects/mm/client/main.js"),
        // elmJs = include_str!("/home/greg/Documents/code/rust/movie_maker/client/main.js"),
        portsJs = PORTS_JS,
        // bulma = include_str!("/home/greg/Documents/code/rust/movie_maker/client/vendor/css/bulma-0.7.2/css/bulma.min.css"),
        bulma = include_str!("/Users/gperlman/Documents/side/rust/projects/mm/client/vendor/bulma-0.7.2/css/bulma.min.css"),
        // fontAwesome = include_str!("/home/greg/Documents/code/rust/movie_maker/client/vendor/fontawesome-free-5.6.1-web/js/all.min.js"),
        fontAwesome = include_str!("/Users/gperlman/Documents/side/rust/projects/mm/client/vendor/fontawesome-free-5.6.1-web/js/all.min.js"),
        customCss = include_str!("/Users/gperlman/Documents/side/rust/projects/mm/client/main.css")
    )
}

const PORTS_JS: &'static str = r#"
    var app = Elm.Main.init({node: document.getElementById("view")});

    app.ports.toBackEnd.subscribe(function (str) {
        window.external.invoke(str);
    });

    function toFrontEnd(str) {
      app.ports.toFrontEnd.send(str);
    }
"#;
