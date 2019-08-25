use crate::api::*;
use crate::graphql::*;
use commandspec::*;
use rayon::prelude::*;
use regex::Regex;
use sha1::Digest;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io::BufReader;
use std::io::Read;
use std::process::Stdio;
use std::time::SystemTime;
use tempfile::NamedTempFile;
use zip;

/// Collection of pages
#[derive(Clone, Debug)]
pub struct Notebook {
    pub id: String,
    pub name: String,
    pub pages: Vec<Page>,
}

#[derive(Clone, Debug)]
pub struct Page {
    pub id: String,
    pub svg: Option<String>,
    pub modified: String,
}

pub fn get_slug(notebook: &str, find_index: usize) -> String {
    format!(
        "{:x}",
        sha1::Sha1::digest(format!("{} {}", notebook, find_index).as_bytes())
    )
}

/// Given the name of a notebook and the index of a file, load as an svg.
pub fn notebook_data(notebook: &str, find_index: usize) -> Result<Vec<u8>, Box<dyn Error>> {
    let slug = get_slug(notebook, find_index);

    let svgfilename = format!("./data/render/{}.svg", slug);
    if let Ok(bytes) = std::fs::read(&svgfilename) {
        return Ok(bytes);
    }
    eprintln!("<Missing cached file> {:?}", svgfilename);
    Ok(vec![])
}

pub fn notebook_render(command: SvgRenderCommand) -> Result<(), Box<dyn Error>> {
    let source = command.rm_path.to_string();

    // Load or create the outfile.
    let re_rm = Regex::new(r"\.rm$").unwrap();
    let svgfilename = re_rm.replace_all(&source, ".svg");
    println!("Rendering: {:?}", source);

    let pdffile = NamedTempFile::new()?;

    {
        use lines_are_rusty::render::*;
        use lines_are_rusty::*;
        use std::fs::File;

        // Load the file into a Vec<u8>
        let mut f = File::open(&source).unwrap();
        let mut line_file = Vec::<u8>::new();
        f.read_to_end(&mut line_file).unwrap();

        let max_size_file = 1024 * 1024 * 10; // 10mb, or 10*1024 kilobytes
        assert!(max_size_file >= line_file.len());

        // Assert fixed header.
        let pages = read_document(&line_file, max_size_file);
        println!("parsed {} pages.", pages.len());

        let output_filename = pdffile.path().to_string_lossy().to_string();
        render(&output_filename, &pages);
    }

    // println!("next......");
    let mut outfile = NamedTempFile::new()?;
    // println!("%%% %%% %%% LINESARERUSTY");
    // println!("%%% %%% %%% pdf2svg");
    sh_command!(
        r#"
            pdf2svg {infile} {outfile}
        "#,
        infile = pdffile.path().to_string_lossy().to_string(),
        outfile = outfile.path().to_string_lossy().to_string(),
    )?
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .output()
    .expect("pdf2svg");
    // println!("oh.");

    let mut svgfile = std::fs::File::create(&svgfilename.to_string())?;
    std::io::copy(&mut outfile, &mut svgfile)?;
    drop(svgfile);

    Ok(())
}

#[derive(Clone)]
pub struct SvgRenderCommand {
    rm_path: String,
    notebook_name: String,
    slug: String,
    index: usize,
}
/// Given the name of a notebook, unpack it.
pub fn notebook_generate(notebook_name: &str) -> Result<Notebook, Box<dyn Error>> {
    std::panic::catch_unwind(move || {
        // Match files in UUID-named folders with numeric filenames & suffix ".rm"
        let re_rmfile = Regex::new(r#"[a-f0-9\-]+/(\d+).rm"#).unwrap();

        let zipfilename = format!("./data/download/{}.zip", notebook_name);

        let zipfile = fs::File::open(&zipfilename)
            .map_err(|_| format!("error loading {:?}", zipfilename))
            .unwrap();
        let reader = BufReader::new(zipfile);
        let mut archive = zip::ZipArchive::new(reader).unwrap();

        let mut pages = vec![];
        let mut files = vec![];
        for i in 0..archive.len() {
            let file = archive.by_index(i).unwrap();
            let outpath = file.sanitized_name();

            if !file.name().ends_with('/') {
                let name = outpath.as_path().to_string_lossy();
                if let Some(cap) = re_rmfile.captures(&name) {
                    // Get index in folder.
                    let index: usize = cap[1].parse().unwrap();
                    let slug = get_slug(&notebook_name, index);

                    // Copy current file into a temp file.
                    let mut exists = false;
                    let bkpfilename = format!("./data/render/{}.rm", slug);
                    if let Ok(f) = std::fs::File::open(&bkpfilename) {
                        if let Ok(metadata) = f.metadata() {
                            if metadata.len() == file.size() {
                                eprintln!("Existing file: {:?}", slug);
                                exists = true;
                            }
                        }
                    }

                    // Adds to pages array.
                    let svg_url = format!("/notebook_static/{}.svg", slug);
                    let modified = format!("{}", file.last_modified().to_time().rfc822z());
                    pages.push(Page {
                        id: slug.clone(),
                        svg: if exists { Some(svg_url) } else { None },
                        modified: modified,
                    });

                    if !exists {
                        files.push((index, i, slug, bkpfilename));
                    }
                }
            }
        }

        // Publish the SvgRenderCommand
        {
            let notebook_name = notebook_name.to_string();
            std::thread::spawn(move || {
                for (index, i, _, bkpfilename) in &files {
                    let mut file = archive.by_index(*i).unwrap();
                    let mut tmpfile = std::fs::File::create(&bkpfilename).unwrap();
                    std::io::copy(&mut file, &mut tmpfile).unwrap();
                    eprintln!("rendering {} #{:?}", notebook_name, index);
                }

                files
                    .into_par_iter()
                    .for_each(move |(index, _, slug, bkpfilename)| {
                        let command = SvgRenderCommand {
                            rm_path: bkpfilename.to_owned(),
                            notebook_name: notebook_name.clone(),
                            slug: slug.clone(),
                            index: index,
                        };

                        notebook_render(command.clone()).ok();

                        let svg_url = format!("/notebook_static/{}.svg", command.slug);
                        for notebook in &mut *PUBLIC_NOTEBOOKS.write().unwrap() {
                            if notebook.name == command.notebook_name {
                                notebook.pages[command.index].svg = Some(svg_url.clone());
                            }
                        }
                    });
            });
        }

        // dbg!(&pages);

        Ok(Notebook {
            id: format!("notebook:{}", notebook_name),
            name: notebook_name.to_string(),
            pages: pages,
        })
    })
    .unwrap()
}

pub fn stat_poll(tracker: &mut HashMap<String, SystemTime>) -> Result<(), Box<dyn Error>> {
    let res = api_ls()?;

    let mut changes = 0;
    for notebook in res {
        // 2019-06-22T23:59:22.191191Z
        let systime = humantime::parse_rfc3339_weak(&notebook.ModifiedClient).unwrap();
        let name = notebook.VissibleName.clone();
        println!("compare: {:?} -> {:?}", name, systime);

        if !tracker.contains_key(&name) || (tracker[&name] != systime) {
            tracker.insert(name.to_owned(), systime);
            changes += 1;
        }
    }
    eprintln!("stat_poll: {} updates.", changes);

    if changes > 0 {
        if let Ok(mut cmd) = command!(
            r"
                cd data/download
                rmapi mget .
            ",
        ) {
            let res = cmd.output().unwrap();
            dbg!(&res);

            // Process notebooks.
            let mut notebooks = vec![];
            for (notebook_name, _) in tracker {
                eprintln!("Generating notebook {}...", notebook_name);
                notebooks.push(notebook_generate(notebook_name).unwrap());
            }
            // println!("notebooks: {:?}", notebooks);
            *PUBLIC_NOTEBOOKS.write().unwrap() = notebooks;
        }
    }

    Ok(())
}

pub fn spawn_remarkable_poller() {
    // let (tx_render, rx_render) = unbounded();

    std::thread::spawn(move || {
        let mut tracker = HashMap::new();

        eprintln!("[spawn_remarkable_poller] Starting polling thread.");
        loop {
            stat_poll(&mut tracker).ok();

            std::thread::sleep(std::time::Duration::from_millis(5000));
        }
    });
}
