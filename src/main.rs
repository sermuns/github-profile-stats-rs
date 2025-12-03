use anyhow::{Context, Result, bail};
use clap::Parser;
use graphql_client::reqwest::post_graphql_blocking as post_graphql;
use reqwest::blocking::Client;
use std::fs;

mod queries;
mod render;

use queries::*;
use render::*;

#[derive(Parser)]
struct Args {
    repo_path: String,
}

fn main() -> Result<(), anyhow::Error> {
    //
    // let github_api_token =
    //     std::env::var("GITHUB_API_TOKEN").context("Missing GITHUB_API_TOKEN env var")?;
    //
    // let args = Args::parse();
    //
    // let (owner, _repo_name) = match args.repo_path.split_once('/') {
    //     Some(pair) => pair,
    //     _ => bail!(
    //         "wrong format for the repository name argument (we expect something like ratatui/ratatui)"
    //     ),
    // };
    //
    // let variables = basic_stats::Variables {
    //     owner: owner.to_string(),
    // };
    //
    // let client = Client::builder()
    //     .user_agent(concat!(
    //         env!("CARGO_PKG_NAME"),
    //         "/",
    //         env!("CARGO_PKG_VERSION"),
    //     ))
    //     .default_headers(
    //         std::iter::once((
    //             reqwest::header::AUTHORIZATION,
    //             reqwest::header::HeaderValue::from_str(&format!("Bearer {}", github_api_token))
    //                 .unwrap(),
    //         ))
    //         .collect(),
    //     )
    //     .build()?;
    //
    // let response_data: basic_stats::ResponseData =
    //     post_graphql::<BasicStats, _>(&client, "https://api.github.com/graphql", variables)?
    //         .data
    //         .context("missing response data")?;
    //
    // let user = response_data.user.context("no user in response")?;
    //
    // dbg!(&user.pull_requests);

    let svg = render_svg(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/languages.typ"
    )))?;
    fs::write("out.svg", svg)?;

    Ok(())
}
