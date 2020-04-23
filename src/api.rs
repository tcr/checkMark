use serde::Deserialize;
use std::error::Error;

#[allow(non_snake_case)]
#[derive(Deserialize)]
pub struct NotebookBlob {
    pub BlobURLGet: String,
    pub BlobURLGetExpires: String,
    pub Bookmarked: bool,
    pub CurrentPage: usize,
    pub ID: String,
    pub Message: String,
    pub ModifiedClient: String,
    pub Parent: String,
    pub Success: bool,
    pub Type: String,
    pub Version: usize,
    pub VissibleName: String,
}

pub fn api_ls() -> Result<Vec<NotebookBlob>, Box<dyn Error>> {
    let docs_url = "https://document-storage-production-dot-remarkable-production.appspot.com/document-storage/json/2/docs";
    let authorization = std::env::var("AUTH").expect("Please set AUTH environment variable to Authorization header ('Bearer <Token>')");
    let mut req = reqwest::Request::new(reqwest::Method::GET, reqwest::Url::parse(docs_url)?);
    req.headers_mut().insert(
        reqwest::header::AUTHORIZATION,
        authorization.parse().unwrap(),
    );
    Ok(reqwest::Client::new().execute(req)?.json()?)
}
