extern crate web_view;
use std::env;
use web_view::*;

fn main() {
    let webview = web_view::builder()
        .title("Movie Manager")
        .content(Content::Html(create_html()))
        .size(320, 480)
        .resizable(true)
        .debug(true)
        .user_data(())
        .invoke_handler(|webview, arg| {
            // let data = webview.user_data();
            // println!("{:?}", arg);
            // println!("{:?}", data);
            match arg {
                "openFolder" => match webview
                    .dialog()
                    .choose_directory("Please choose a folder...", "")?
                {
                    Some(path) => println!("Folder at {}", path.to_string_lossy()),
                    None => println!("Cancelled opening folder"),
                },
                _ => {
                    println!("got an ipc but doesnt match");
                }
            };
            Ok(())
        }).build()
        .unwrap();

    webview.run().unwrap();
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
"#;
