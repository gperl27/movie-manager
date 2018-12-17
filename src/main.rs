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

        println!("cache contains: {:?}", contents);

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

        &self.data
    }

    fn insert(&mut self, movie: Movie) {
        // remove occurence of same filename
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

fn main() {
    let mut cache: Cache<Movie> = Cache::new();
    cache.initialize();

    println!("data: {:?}", cache.data);

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
                        let mut path = path.into_os_string().into_string().unwrap();
                        &path.push_str("/*.mp4");

                        for entry in glob(&path).unwrap() {
                            let movie = Movie::new(entry.unwrap());

                            cache.insert(movie);
                        }

                        // update cache with files found from current folder
                        cache.write(cache.serialize());

                        send_to_ui(webview, &cache.get_files());
                    }
                    None => println!("Cancelled opening folder"),
                },
                Ok(Action::Search { keyword }) => {
                    send_to_ui(webview, &cache.search_files(&keyword));
                }
                Ok(Action::Play { movie }) => {
                    movie.play();
                }

                Err(error) => println!("Unable to parse [{}] because {}", arg, error),
            }
            Ok(())
        }).build()
        .unwrap();

    webview.run().unwrap();
}

#[derive(Deserialize, Serialize, Debug)]
struct Movie {
    filepath: String,
    filename: String,
    exists: bool,
}

impl Movie {
    fn new(entry: PathBuf) -> Movie {
        let filepath = String::from(entry.to_str().unwrap());
        let filename = String::from(entry.file_name().unwrap().to_str().unwrap());

        Movie {
            filepath,
            filename,
            exists: true,
        }
    }

    fn play(&self) {
        if open::that(&self.filepath).is_ok() {
            println!("Opening file...");
        }
    }
}

fn send_to_ui<'a, S, T>(webview: &mut WebView<'a, T>, data: &S)
where
    S: serde::ser::Serialize,
{
    let render_movies = {
        let data = serde_json::to_string(&data).unwrap();

        format!("toFrontEnd({})", data)
    };

    println!("{:#?}", &render_movies);
    webview.eval(&render_movies).unwrap();
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
