extern crate glob;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate web_view;

use glob::glob;
use std::path::PathBuf;
use web_view::*;

struct State {
    files: Box<Vec<Movie>>,
}

impl State {
    fn new() -> State {
        State {
            files: Box::new(vec![]),
        }
    }

    fn get_files(&self) -> &Vec<Movie> {
        &self.files
    }

    fn search_files(&self, search: &str) -> Vec<&Movie> {
        let t = self.get_files();

        let mut found_files = vec![];

        for f in t.into_iter() {
            if f.filename.contains(search) {
                found_files.push(f);
            }
        }

        found_files
        // .cloned()
        // .filter(|x| x.filename.contains(search))
        // .collect()

        // b.files.iter().filter(|x| true)
        // &self.files
    }
}

fn main() {
    let mut state = State::new();

    let webview = web_view::builder()
        .title("Movie Manager")
        .content(Content::Html(create_html()))
        .size(800, 800)
        .resizable(true)
        .debug(true)
        .user_data(())
        .invoke_handler(|webview, arg| {
            match arg {
                "openFolder" => match webview
                    .dialog()
                    .choose_directory("Please choose a folder...", "")?
                {
                    Some(path) => {
                        let state = &mut state;
                        state.files.clear();

                        let mut path = path.into_os_string().into_string().unwrap();
                        &path.push_str("/*.mp4");

                        for entry in glob(&path).unwrap() {
                            let movie = Movie::new(entry.unwrap());
                            state.files.push(movie);
                        }
                        send_to_ui(webview, state.get_files());
                    }
                    None => println!("Cancelled opening folder"),
                },
                "search" => {
                    // let files = state.search_files("Sample");
                    send_to_ui(webview, &state.search_files("Sample"));
                }
                _ => {
                    println!("got an ipc but doesnt match");
                }
            };

            // let state = webview.user_data_mut();
            // let movies = state.get_files();
            // // println!("{:?}", movies.get_files());
            // {

            // send_to_ui(webview, movies);
            // }

            Ok(())
        }).build()
        .unwrap();

    webview.run().unwrap();
}

#[derive(Serialize, Debug)]
struct Movie {
    filepath: String,
    filename: String,
}

impl Movie {
    fn new(entry: PathBuf) -> Movie {
        let filepath = String::from(entry.to_str().unwrap());
        let filename = String::from(entry.file_name().unwrap().to_str().unwrap());

        Movie { filepath, filename }
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
    </head>
    <body>
        <div id="view"></div>
        <script>
            {elmJs}
            {portsJs}
        </script> 
        
    </body>
    </html>
    "#,
        // elmJs = include_str!("/Users/gperlman/Documents/side/rust/projects/mm/client/main.js"),
        elmJs = include_str!("/home/greg/Documents/code/rust/movie_maker/client/main.js"),
        portsJs = PORTS_JS
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
