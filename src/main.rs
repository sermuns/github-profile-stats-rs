use anyhow::{Context, Result, bail};
use clap::Parser;
use graphql_client::reqwest::post_graphql_blocking as post_graphql;
use reqwest::blocking::Client;
use std::{collections::HashMap, fs};

mod queries;
mod render;

use queries::*;
use render::*;

#[derive(Parser)]
struct Args {
    github_owner: String,
}

fn main() -> Result<(), anyhow::Error> {
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
    let repositories = user
        .repositories
        .nodes
        .context("no repositories found!")?
        .into_iter()
        .flatten();

    let mut total_languages_size: u64 = 0;
    let mut language_size_map: HashMap<String, u64> = HashMap::new();
    for repo in repositories {
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

    dbg!(&language_size_map);

    // let languages_svg = render_svg(include_str!(concat!(
    //     env!("CARGO_MANIFEST_DIR"),
    //     "/src/languages.typ"
    // )))?;
    // fs::write("languages.svg", languages_svg)?;

    Ok(())
}
