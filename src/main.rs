use bincode::config;
use clap::Parser;
use color_eyre::eyre::Context;
use octocrab::models::repos::Languages;
use prettytable::*;
use reqwest::blocking::Client;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
use std::ops::DerefMut;
use std::{collections::HashMap, fs, iter::Flatten, vec::IntoIter};
use tokio_stream::StreamExt;
use typst::compile;
use typst::foundations::{Array, IntoValue, Str, panic};
use typst::foundations::{Dict, Value};

#[derive(Parser)]
struct Args {
    github_username: String,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let args = Args::parse();

    let github_api_token = std::env::var("GITHUB_TOKEN")?;
    let octocrab = octocrab::Octocrab::builder()
        .personal_token(github_api_token)
        .build()?;

    let user = octocrab.users(args.github_username);

    let mut total_languages = Languages::new();

    // TODO: make this concurrent..
    for page_num in 1..u32::MAX {
        let page = user.repos().page(page_num).send().await?;
        for repo in page.items {
            let repo_id = repo.id;
            let languages = octocrab.repos_by_id(repo_id).list_languages().await?;
            for (language_name, bytes) in languages {
                total_languages
                    .entry(language_name)
                    .and_modify(|b| *b += bytes)
                    .or_insert(bytes);
            }
        }
        if page.next.is_none() {
            break;
        }
    }
    dbg!(total_languages);

    let mut table = prettytable::Table::new();
    table.set_format(*prettytable::format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    table.set_titles(row!(b => "language", "size", "percentage"));
    table.printstd();

    // let mut input = Dict::new();
    // input.insert("languages".into(), Value::Array(Array::new()));
    // let mut input_languages = input.at_mut("languages").unwrap();
    // let languages_svg = compile_svg(
    //     include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/languages.typ")),
    //     input,
    // )?;
    // fs::write("languages.svg", languages_svg)?;
    //
    Ok(())
}
