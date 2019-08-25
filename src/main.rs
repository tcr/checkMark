use juniper::IntrospectionFormat;
use market::graphql::*;
use market::notebook::*;
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
    let notebook_static = path!("notebook_static").and(warp::fs::dir("data/render"));

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
            .or(warp::path("graphql").and(graphql_filter))
            .with(log),
    )
    .run(([0, 0, 0, 0], 8080));
}
