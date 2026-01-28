use color_eyre::eyre::{ContextCompat, WrapErr, bail};
use typst_as_lib::{TypstAsLibError, TypstEngine};
use typst_library::{
    diag::Warned,
    foundations::Dict,
    layout::{Page, PagedDocument},
};

const NOTO_SANS_REGULAR: &[u8] = include_bytes!("../assets/NotoSans-Regular.ttf");
// const NOTO_SANS_ITALIC: &[u8] = include_bytes!("../assets/NotoSans-Italic.ttf");
const NOTO_SANS_BOLD: &[u8] = include_bytes!("../assets/NotoSans-Bold.ttf");
// const NOTO_SANS_BOLD_ITALIC: &[u8] = include_bytes!("../assets/NotoSans-BoldItalic.ttf");
const MONASPACE_KRYPTON: &[u8] = include_bytes!("../assets/MonaspaceKrypton-Regular.otf");
const MONASPACE_KRYPTON_BOLD: &[u8] = include_bytes!("../assets/MonaspaceKrypton-Bold.otf");

pub fn compile_svg(template_str: &str, input: Dict) -> color_eyre::Result<String> {
    let languages_template = TypstEngine::builder()
        .main_file(template_str)
        .fonts([
            NOTO_SANS_REGULAR,
            // NOTO_SANS_ITALIC,
            NOTO_SANS_BOLD,
            // NOTO_SANS_BOLD_ITALIC,
            MONASPACE_KRYPTON,
            MONASPACE_KRYPTON_BOLD,
        ])
        .with_package_file_resolver()
        .build();

    let warned_document: Warned<Result<PagedDocument, TypstAsLibError>> =
        languages_template.compile_with_input(input);

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
    let document_pages: Vec<Page> = warned_document.output.wrap_err("ERROR COMPILING")?.pages;
    if document_pages.len() > 1 {
        println!("output document has multiple pages!")
    }
    let first_and_only_page = document_pages
        .first()
        .wrap_err("output document has no pages!")?;

    Ok(typst_svg::svg(first_and_only_page))
}
