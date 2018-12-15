extern crate glob;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate web_view;

use glob::glob;
use web_view::*;

fn main() {
    let webview = web_view::builder()
        .title("Movie Manager")
        .content(Content::Html(create_html()))
        .size(800, 800)
        .resizable(true)
        .debug(true)
        .user_data(vec![])
        .invoke_handler(|webview, arg| {
            // let data = webview.user_data();
            // println!("{:?}", karg);
            // println!("{:?}", data);
            match arg {
                "openFolder" => match webview
                    .dialog()
                    .choose_directory("Please choose a folder...", "")?
                {
                    Some(path) => {
                        let movies = webview.user_data_mut();
                        movies.clear();

                        let mut path = path.into_os_string().into_string().unwrap();
                        &path.push_str("/*.*");

                        for entry in glob(&path).unwrap() {
                            movies.push(entry.unwrap());
                        }
                    }
                    None => println!("Cancelled opening folder"),
                },
                _ => {
                    println!("got an ipc but doesnt match");
                }
            };

            let render_movies = {
                let movies = webview.user_data();

                let mut movie_vec = vec![];

                for movie in movies {
                    let movie = Movie { file: movie };
                    movie_vec.push(movie)
                }

                let movies = serde_json::to_string(&movie_vec).unwrap();

                format!("toFrontEnd({})", movies)
            };

            println!("{:#?}", &render_movies);
            webview.eval(&render_movies).unwrap();

            Ok(())
        }).build()
        .unwrap();

    webview.run().unwrap();
}

#[derive(Serialize, Debug)]
struct Movie<'a> {
    file: &'a std::path::PathBuf,
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
        elmJs = include_str!("/Users/gperlman/Documents/side/rust/projects/mm/client/main.js"),
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
