#![allow(unused)]

use anyhow::{Context, Result, bail};
use bincode::config;
use clap::Parser;
use graphql_client::reqwest::post_graphql_blocking as post_graphql;
use prettytable::*;
use reqwest::blocking::Client;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
use std::ops::DerefMut;
use std::{collections::HashMap, fs, iter::Flatten, vec::IntoIter};
use typst::compile;
use typst::foundations::{Array, IntoValue, Str, panic};
use typst::foundations::{Dict, Value};

mod queries;
mod render;

use crate::queries::repo_stats::RepoStatsUserRepositoriesNodes;
use crate::queries::*;
use crate::render::*;

#[derive(Parser)]
struct Args {
    github_owner: String,
}

fn get_repos() -> Result<Vec<RepoStatsUserRepositoriesNodes>> {
    let github_api_token = std::env::var("GITHUB_TOKEN").context("missing GITHUB_TOKEN env var")?;

    let args = Args::parse();

    let request_client = Client::builder()
        .user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION"),
        ))
        .default_headers(
            std::iter::once((
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", github_api_token))
                    .unwrap(),
            ))
            .collect(),
        )
        .build()?;

    let repo_stats_response = post_graphql::<RepoStats, _>(
        &request_client,
        "https://api.github.com/graphql",
        repo_stats::Variables {
            owner: args.github_owner.to_string(),
            end_cursor: None,
        },
    )?
    .data
    .context("missing response data")?;

    let user = repo_stats_response.user.context("no user found")?;
    let repositories: Vec<RepoStatsUserRepositoriesNodes> = user
        .repositories
        .nodes
        .context("no repositories found!")?
        .into_iter()
        .flatten()
        .collect();

    Ok(repositories)
}

fn main() -> Result<(), anyhow::Error> {
    // let repositories = get_repos()?;
    // let repositories_bin = bincode::encode_to_vec(repositories, bincode::config::standard())?;
    // fs::write("repos.bin", repositories_bin)?;
    // return Ok(());
    let repositories_file = File::open("repos.bin")?;
    let repositories: Vec<RepoStatsUserRepositoriesNodes> =
        bincode::decode_from_reader(BufReader::new(repositories_file), config::standard())?;

    let mut total_languages_size: u64 = 0;
    let mut language_size_map: BTreeMap<String, i64> = BTreeMap::new();
    for repo in repositories {
        if repo.is_fork || repo.is_private {
            continue;
        }
        for language in repo.languages.as_ref().unwrap().edges.as_ref().unwrap() {
            let language = language.as_ref().unwrap();
            *language_size_map
                .entry(language.node.name.clone())
                .or_insert(0) += language.size;
            total_languages_size += u64::try_from(language.size).unwrap();
        }
    }

    let mut sorted_language_sizes: Vec<_> = language_size_map.iter().collect();
    sorted_language_sizes.sort_by_key(|(k, v)| *v);
    sorted_language_sizes.reverse();

    let mut input = Dict::new();
    input.insert("languages".into(), Value::Array(Array::new()));
    let mut input_languages = input.at_mut("languages").unwrap();

    let mut table = prettytable::Table::new();
    table.set_format(*prettytable::format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    table.set_titles(row!(b => "language", "size", "percentage"));

    for (name, size) in sorted_language_sizes {
        table.add_row(row!(
            name,
            size,
            (100 * size) as f32 / (total_languages_size as f32)
        ));
        let percentage_times_1000 = (1000 * 100 * size) / (total_languages_size as i64);
        let array = Array::from_iter([
            Value::Str((*name.clone()).into()),
            Value::Int(percentage_times_1000),
        ]);
        match input.at_mut("languages").unwrap() {
            Value::Array(a) => a.push(Value::Array(array)),
            _ => bail!("NON-ARRAY IN LANGUAGES"),
        };
    }
    table.printstd();

    let languages_svg = compile_svg(
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/languages.typ")),
        input,
    )?;
    fs::write("languages.svg", languages_svg)?;

    Ok(())
}
