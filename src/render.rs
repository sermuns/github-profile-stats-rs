use anyhow::{Context, Result, bail};
use typst_as_lib::{TypstAsLibError, TypstEngine};
use typst_library::{
    diag::Warned,
    layout::{Page, PagedDocument},
};

pub static NOTO_SANS_REGULAR: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/NotoSans-Regular.ttf"
));
pub static NOTO_SANS_ITALIC: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/NotoSans-Italic.ttf"
));
pub static NOTO_SANS_BOLD: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/NotoSans-Bold.ttf"
));
pub static NOTO_SANS_BOLD_ITALIC: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/NotoSans-BoldItalic.ttf"
));

pub fn render_svg(template_str: &str) -> Result<String> {
    let languages_template = TypstEngine::builder()
        .main_file(template_str)
        .fonts([
            NOTO_SANS_REGULAR,
            NOTO_SANS_ITALIC,
            NOTO_SANS_BOLD,
            NOTO_SANS_BOLD_ITALIC,
        ])
        .build();

    let warned_document: Warned<Result<PagedDocument, TypstAsLibError>> =
        languages_template.compile();

    let warnings = warned_document.warnings;
    if !warnings.is_empty() {
        bail!(
            "Typst had warnings: {}",
            warnings
                .iter()
                .enumerate()
                .fold(String::new(), |acc, (i, warning)| acc
                    + &format!("\n {}: {}", i + 1, warning.message))
        );
    }
    let document_pages: Vec<Page> = warned_document.output?.pages;
    if document_pages.len() > 1 {
        bail!("output document has multiple pages!")
    }
    let first_page = document_pages
        .first()
        .context("output document has no pages!")?;

    Ok(typst_svg::svg(first_page))
}
