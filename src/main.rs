use camino::Utf8PathBuf;
use clap::Parser;
use color_eyre::eyre::{WrapErr, bail};
use derive_typst_intoval::{IntoDict, IntoValue};
use futures::{StreamExt, stream};
use indicatif::{ProgressBar, ProgressStyle};
use octocrab::models::{Repository, repos::Languages};
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs,
    io::Cursor,
    sync::{Arc, LazyLock},
};
use typst::foundations::{Dict, IntoValue, Value};

mod render;
use crate::render::compile_svg;

#[derive(Parser)]
#[clap(version, about, long_about = None)]
struct Args {
    github_username: String,

    #[arg(short, long, default_value = "languages.svg")]
    output: Utf8PathBuf,

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

    static ARGS: LazyLock<Args> = LazyLock::new(Args::parse);
    if ARGS.output.is_dir() {
        bail!("output must be a file. got `{}`", ARGS.output);
    } else if ARGS.output.extension().is_some_and(|ext| ext != "svg") {
        bail!("output must end in .svg. got `{}`", ARGS.output);
    }

    let linguist_languages: HashMap<String, LinguistLanguage> =
        serde_yaml_ng::from_reader(Cursor::new(&mut include_bytes!("../assets/languages.yml")))?;

    for skipped_lang in &ARGS.skipped_languages {
        if !linguist_languages
            .keys()
            .any(|k| k.to_lowercase() == skipped_lang.to_lowercase())
        {
            bail!("Language to skip `{}` is unknown", skipped_lang);
        }
    }

    let github_api_token = std::env::var("GITHUB_TOKEN")
        .wrap_err("GITHUB_TOKEN is needed to avoid rate-limitation")?;
    let octocrab = Arc::new(
        octocrab::Octocrab::builder()
            .personal_token(github_api_token)
            .build()?,
    );

    let user = octocrab.users(&ARGS.github_username);
    // let user_profile = user.profile().await?;
    // let user_profile_name = user_profile.name.unwrap_or(user_profile.login);

    // TODO: make this concurrent..
    let mut repos: Vec<Repository> = Vec::new();
    for page_num in 1..u32::MAX {
        let mut page = user.repos().page(page_num).send().await?;
        let is_last_page = page.next.is_none();
        repos.append(&mut page.items);
        if is_last_page {
            break;
        }
    }

    let pb = Arc::new(
        ProgressBar::new(repos.len() as u64).with_style(
            ProgressStyle::with_template("[{elapsed_precise}] {wide_bar} {pos:>7}/{len:7} {msg}")
                .unwrap(),
        ),
    );

    let repo_language_hashmaps: Vec<_> = stream::iter(repos.into_iter())
        .map(|repo| {
            let pb = pb.clone();
            let octocrab = octocrab.clone();
            async move {
                pb.set_message(repo.name);
                if ARGS.skip_forks && repo.fork.unwrap() {
                    return None;
                }
                if ARGS.skip_private && repo.private.unwrap() {
                    return None;
                }
                let repo_id = repo.id;

                // FIXME: dont' create an octocrab instance...
                let languages = octocrab
                    .repos_by_id(repo_id)
                    .list_languages()
                    .await
                    .unwrap() // FIXME:
                    .into_iter()
                    .filter(|(lang_name, _)| {
                        // FIXME: stop allocating strings.. can we precompute lowercases?
                        !ARGS.skipped_languages.contains(&lang_name.to_lowercase())
                    });

                Some(languages)
            }
        })
        .buffer_unordered(10)
        .collect()
        .await;
    pb.finish();
    let mut total_languages = Languages::new();
    for hashmap in repo_language_hashmaps.into_iter().flatten() {
        for (lang_name, bytes) in hashmap {
            total_languages
                .entry(lang_name)
                .and_modify(|total_bytes| *total_bytes += bytes)
                .or_insert(bytes);
        }
    }
    let mut total_languages_vec: Vec<_> = total_languages.into_iter().collect();
    total_languages_vec.sort_by_key(|(_, bytes)| !*bytes);

    if ARGS.num_languages != 0 {
        let other_bytes = total_languages_vec
            .iter()
            .skip(ARGS.num_languages)
            .fold(0, |acc, (_, bytes)| acc + bytes);
        total_languages_vec.truncate(ARGS.num_languages);
        total_languages_vec.push(("Other".into(), other_bytes));
    }

    let mut input = Dict::new();

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

    // HACK: because typst-as-lib uses internal blocking reqwest to resolve packages we have
    // to spawn this as blocking..
    let languages_svg = tokio::task::spawn_blocking(move || {
        compile_svg(include_str!("../src/languages.typ"), input)
    })
    .await??;

    fs::write(&ARGS.output, languages_svg)?;

    Ok(())
}
