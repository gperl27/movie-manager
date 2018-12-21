#[macro_use]
extern crate serde_derive;
extern crate glob;
extern crate lib;
extern crate open;
extern crate serde_json;
extern crate web_view;

extern crate dotenv;

use dotenv::dotenv;
use std::env;

use glob::glob;
use lib::*;
use web_view::*;

fn is_in_production() -> bool {
    match env::var("PRODUCTION") {
        Ok(val) => val == "true",
        Err(_) => false
    }
}

fn main() {
    dotenv().ok();

    let mut state = State::new();
    let mut cache: Cache<Movie> = Cache::new();
    cache.initialize();

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
                            movies: &cache.search_files(&keyword, &state.get_folders()),
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
                            movies: &cache
                                .search_files(&state.search_keyword, &state.get_folders()),
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
                            movies: &cache
                                .search_files(&state.search_keyword, &state.get_folders()),
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

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "_type")]
enum Action {
    OpenFolder,
    Search { keyword: String },
    Play { movie: Movie },
    ClickFolder { folder: String },
    UnclickFolder { folder: String },
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
    match serde_json::to_string(data) {
        Ok(json) => match webview.eval(&format!("toFrontEnd({})", json)) {
            Ok(_) => println!("Sent to UI"),
            Err(error) => println!("failed to send to ui because {}", error),
        },
        Err(error) => println!("failed to serialize for ui because {}", error),
    };
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
        elmJs = if is_in_production() {
            include_str!("../client/main.min.js")
        } else {
            include_str!("../client/main.js")
        },
        portsJs = PORTS_JS,
        bulma = include_str!("../client/vendor/bulma-0.7.2/css/bulma.min.css"),
        fontAwesome = include_str!("../client/vendor/fontawesome-free-5.6.1-web/js/all.min.js"),
        customCss = include_str!("../client/main.css")
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
