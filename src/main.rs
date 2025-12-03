use anyhow::{Context, Result, bail};
use clap::Parser;
use graphql_client::reqwest::post_graphql_blocking as post_graphql;
use log::*;
use prettytable::*;
use reqwest::blocking::Client;

mod queries;

use queries::*;

#[derive(Parser)]
struct Args {
    repo: String,
}

fn main() -> Result<(), anyhow::Error> {
    env_logger::init();

    let github_api_token =
        std::env::var("GITHUB_API_TOKEN").context("Missing GITHUB_API_TOKEN env var")?;

    let args = Args::parse();

    let repo = args.repo;
    let (owner, name) = match repo.split_once('/') {
        Some((owner, name)) => (owner, name),
        _ => bail!(
            "wrong format for the repository name argument (we expect something like ratatui/ratatui)"
        ),
    };

    let variables = basic_stats::Variables {
        owner: owner.to_string(),
    };

    let client = Client::builder()
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

    let response_body =
        post_graphql::<BasicStats, _>(&client, "https://api.github.com/graphql", variables)?;

    info!("{:?}", response_body);

    let response_data: basic_stats::ResponseData =
        response_body.data.expect("missing response data");

    let user = response_data.user.context("NO USER")?;

    println!("{}", user.followers.total_count);

    // println!("{}/{} - 🌟 {}", owner, name, stars.unwrap_or(0),);
    //
    // let mut table = prettytable::Table::new();
    // table.set_format(*prettytable::format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    // table.set_titles(row!(b => "issue", "comments"));
    //
    // for issue in response_data
    //     .repository
    //     .expect("missing repository")
    //     .issues
    //     .nodes
    //     .expect("issue nodes is null")
    //     .iter()
    //     .flatten()
    // {
    //     table.add_row(row!(issue.title, issue.comments.total_count));
    // }
    //
    // table.printstd();
    Ok(())
}
