#![allow(unused)]

use anyhow::{Context, Result, bail};
use bincode::config;
use clap::Parser;
use clap::builder::Str;
use graphql_client::reqwest::post_graphql_blocking as post_graphql;
use prettytable::*;
use reqwest::blocking::Client;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
use std::{collections::HashMap, fs, iter::Flatten, vec::IntoIter};

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
    let mut language_size_map: BTreeMap<String, u64> = BTreeMap::new();
    for repo in repositories {
        if repo.is_fork || repo.is_private {
            continue;
        }
        for language in repo.languages.as_ref().unwrap().edges.as_ref().unwrap() {
            let language = language.as_ref().unwrap();
            let language_size = u64::try_from(language.size).unwrap();
            language_size_map
                .entry(language.node.name.clone())
                .and_modify(|size| *size += language_size)
                .or_insert(language_size);
            total_languages_size += language_size;
        }
    }

    let mut table = prettytable::Table::new();
    table.set_format(*prettytable::format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    table.set_titles(row!(b => "language", "size", "percentage"));

    for (name, size) in language_size_map {
        table.add_row(row!(
            name,
            size,
            (100 * size) as f32 / (total_languages_size as f32)
        ));
    }
    table.printstd();

    // let languages_svg = render_svg(include_str!(concat!(
    //     env!("CARGO_MANIFEST_DIR"),
    //     "/src/languages.typ"
    // )))?;
    // fs::write("languages.svg", languages_svg)?;

    Ok(())
}
