use checkmark::graphql::*;
use checkmark::notebook::*;
use juniper::IntrospectionFormat;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use tempfile::NamedTempFile;
use warp::{http::Response, log, path, Filter};

fn main() {
    std::env::set_var("RUST_LOG", "warp_server");
    env_logger::init();

    std::fs::create_dir_all("data/download").ok();
    std::fs::create_dir_all("data/render").ok();

    // Launch networking.
    spawn_remarkable_poller();

    let log = log("warp_server");

    let homepage = warp::path::end().and(warp::fs::dir("frontend/build"));
    let homepage_static = warp::path("static").and(warp::fs::dir("frontend/build/static"));
    // let homepage_static_js =
    // warp::path("static/js").and(warp::fs::dir("frontend/build/static/js"));
    let path_schema = warp::path("schema.json").map(|| {
        // Run the built-in introspection query.
        let (res, _errors) =
            juniper::introspect(&schema(), &(), IntrospectionFormat::default()).unwrap();

        // Convert introspection result to json.
        let json_result = serde_json::to_string_pretty(&res);
        assert!(json_result.is_ok());

        Response::builder()
            .header("content-type", "application/json")
            .body(json_result.unwrap())
    });

    use warp::http::header::{HeaderMap, HeaderValue};
    let mut headers = HeaderMap::new();
    // headers.insert(
    //     "content-disposition",
    //     HeaderValue::from_static("attachment"),
    // );
    let notebook_static = path!("notebook_static")
        .and(warp::fs::dir("data/render"))
        .with(warp::reply::with::headers(headers.clone()));
    let notebook_static_png = path!("notebook_static")
        .and(warp::path::tail())
        .and_then(|tail: warp::filters::path::Tail| {
            if tail.as_str().ends_with(".png") {
                use resvg::prelude::*;

                let pngfilename = Path::new("data/render").join(tail.as_str());
                let svgf = pngfilename
                    .to_string_lossy()
                    .to_string()
                    .replace(".png", ".svg")
                    .to_string();
                let svgfilename = Path::new(&svgf);

                println!("Rendering: {:?}", pngfilename);
                println!("from: {:?}", svgfilename);

                let mut opt = resvg::Options::default();
                opt.usvg.path = Some(std::path::PathBuf::from(svgfilename.clone()));
                let rtree = usvg::Tree::from_file(svgfilename.clone(), &opt.usvg).unwrap();
                let backend = resvg::default_backend();
                let mut img = backend.render_to_image(&rtree, &opt).unwrap();
                let tmpfile = NamedTempFile::new().unwrap();
                img.save_png(tmpfile.path());

                let mut buffer = Vec::new();
                // read the whole file
                let mut f = File::open(tmpfile).unwrap();
                f.read_to_end(&mut buffer).unwrap();

                Ok(Response::builder()
                    .header("content-type", "image/png")
                    .body(buffer))
            } else {
                Err(warp::reject::not_found())
            }
        })
        .with(warp::reply::with::headers(headers.clone()));

    let state = warp::any().map(move || ());
    let graphql_filter = juniper_warp::make_graphql_filter(schema(), state.boxed());

    eprintln!("Listening on http://0.0.0.0:8080");
    warp::serve(
        warp::get2()
            .and(warp::path("graphiql"))
            .and(juniper_warp::graphiql_filter("/graphql"))
            .or(homepage)
            .or(homepage_static)
            .or(path_schema)
            // .or(homepage_static_js)
            // .or(notebook)
            .or(notebook_static)
            .or(notebook_static_png)
            .or(warp::path("graphql").and(graphql_filter))
            .with(log),
    )
    .run(([0, 0, 0, 0], 8080));
}
