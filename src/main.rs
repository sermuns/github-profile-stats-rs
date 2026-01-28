use clap::Parser;
use color_eyre::eyre::{WrapErr, bail};
use derive_typst_intoval::{IntoDict, IntoValue};
use octocrab::models::repos::Languages;
use serde::Deserialize;
use std::{collections::HashMap, fs, io::Cursor};
use typst::foundations::{Dict, IntoValue, Value};

mod render;
use crate::render::compile_svg;

#[derive(Parser)]
struct Args {
    github_username: String,

    /// don't include repos that are forks
    #[arg(long, default_value_t = true)]
    skip_forks: bool,

    /// don't include private repos
    #[arg(long, default_value_t = true)]
    skip_private: bool,

    /// don't include these languages
    #[arg(short, long, value_delimiter = ',')]
    skipped_languages: Vec<String>,

    /// how many languages to show. the rest will be merged into "Other"
    /// 0 means infinite
    #[arg(short, long, default_value_t = 5)]
    num_languages: usize,
}

#[derive(Debug, Deserialize, IntoDict, IntoValue)]
struct LinguistLanguage {
    color: Option<String>,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let linguist_languages: HashMap<String, LinguistLanguage> =
        serde_yaml_ng::from_reader(Cursor::new(&mut include_bytes!("../assets/languages.yml")))?;

    let args = Args::parse();

    for skipped_lang in args.skipped_languages {
        if !linguist_languages
            .keys()
            .any(|k| k.to_lowercase() == skipped_lang.to_lowercase())
        {
            bail!("Language to skip `{}` is unknown", skipped_lang);
        }
    }

    let github_api_token = std::env::var("GITHUB_TOKEN")
        .wrap_err("GITHUB_TOKEN is needed to avoid rate-limitation")?;
    let octocrab = octocrab::Octocrab::builder()
        .personal_token(github_api_token)
        .build()?;

    let user = octocrab.users(args.github_username);
    // let user_profile = user.profile().await?;
    // let user_profile_name = user_profile.name.unwrap_or(user_profile.login);

    let mut total_languages = Languages::new();

    // TODO: make this concurrent..
    for page_num in 1..u32::MAX {
        let page = user.repos().page(page_num).send().await?;
        for repo in page.items {
            if args.skip_forks && repo.fork.unwrap() {
                continue;
            }
            if args.skip_private && repo.private.unwrap() {
                continue;
            }
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

    let mut input = Dict::new();

    let mut total_languages_vec: Vec<_> = total_languages.into_iter().collect();

    total_languages_vec.sort_by_key(|(_, bytes)| !*bytes);

    if args.num_languages != 0 {
        let other_bytes = total_languages_vec
            .iter()
            .skip(args.num_languages)
            .fold(0, |acc, (_, bytes)| acc + bytes);
        total_languages_vec.truncate(args.num_languages);
        total_languages_vec.push(("Other".into(), other_bytes));
    }

    let languages_dict = Dict::from_iter(total_languages_vec.iter().map(|(name, bytes)| {
        (
            name.as_str().into(),
            Value::Dict(Dict::from_iter([
                (
                    "color".into(),
                    linguist_languages
                        .get(name)
                        .and_then(|l| l.color.clone())
                        .unwrap_or_else(|| "#444".to_string())
                        .into_value(),
                ),
                ("bytes".into(), bytes.into_value()),
            ])),
        )
    }));
    input.insert("languages".into(), Value::Dict(languages_dict));

    let languages_svg = tokio::task::spawn_blocking(move || {
        compile_svg(include_str!("../src/languages.typ"), input)
    })
    .await??;

    fs::write("languages.svg", languages_svg)?;

    Ok(())
}
