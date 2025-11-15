use reqwest::blocking::get;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use syntect::dumps::dump_to_file;
use syntect::highlighting::ThemeSet;
use syntect::parsing::{SyntaxDefinition, SyntaxSetBuilder};

fn main() -> Result<(), Box<dyn Error>> {
    let cargo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let syntax_dump_path = cargo_root.join("pack.dump");
    let theme_dir = cargo_root.join("themes");

    let syntax_sets = [
        "https://raw.githubusercontent.com/sublimehq/Packages/refs/heads/master/Rust/Rust.sublime-syntax",
        "https://raw.githubusercontent.com/sublimehq/Packages/refs/heads/master/Markdown/Markdown.sublime-syntax",
        "https://raw.githubusercontent.com/sublimehq/Packages/refs/heads/master/ShellScript/Bash.sublime-syntax",
        "https://raw.githubusercontent.com/sublimehq/Packages/refs/heads/master/TOML/TOML.sublime-syntax",
    ];

    let mut builder = SyntaxSetBuilder::new();
    builder.add_plain_text_syntax();

    for syntax_url in &syntax_sets {
        let contents = get(*syntax_url)?.text()?;
        let syntax_definition = SyntaxDefinition::load_from_str(&contents, false, None)?;
        builder.add(syntax_definition);
    }

    let syntax_set = builder.build();

    dump_to_file(&syntax_set, &syntax_dump_path)?;

    let themes = [
        "https://raw.githubusercontent.com/SublimeText/Spacegray/2703e93f559e212ef3895edd10d861a4383ce93d/base16-ocean.dark.tmTheme",
        "https://raw.githubusercontent.com/SublimeText/Spacegray/refs/heads/main/Spacegray%20Light.sublime-theme",
    ];

    if !theme_dir.exists() {
        fs::create_dir_all(&theme_dir)?;
    }

    for theme_url in &themes {
        let contents = get(*theme_url)?.text()?;

        // Use the filename from the URL directly
        let filename = theme_url.split('/').next_back().ok_or("Invalid URL, no filename found")?;

        let theme_path = theme_dir.join(filename);
        fs::write(&theme_path, contents)?;
    }

    let theme_set = ThemeSet::load_from_folder(&theme_dir)?;
    let theme_dump_path = cargo_root.join("theme_pack.dump");
    dump_to_file(&theme_set, &theme_dump_path)?;

    println!("Syntax set and themes dumped successfully.");
    Ok(())
}
