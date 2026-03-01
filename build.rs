use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct Project {
    name: String,
    description: String,
    language: String,
    owner: String,
    repo: String,
}

#[derive(Debug, Deserialize)]
struct Config {
    year: u32,
    projects: Vec<Project>,
}

fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn main() {
    use std::fmt::Write;
    println!("cargo::rerun-if-changed=config.json");

    let config_path = Path::new("config.json");
    let config_content = fs::read_to_string(config_path).expect("failed to read config.json");
    let config: Config =
        serde_json::from_str(&config_content).expect("failed to parse config.json");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("config_generated.rs");

    let mut output = String::new();

    write!(&mut output, "pub const YEAR: u32 = {};\n\n", config.year).unwrap();

    writeln!(
        &mut output,
        "pub const PROJECTS: [crate::Project; {}] = [",
        config.projects.len()
    )
    .unwrap();

    for project in &config.projects {
        write!(
            &mut output,
            r#"    crate::Project {{
        name: "{}",
        description: "{}",
        language: "{}",
        owner: "{}",
        repo: "{}",
    }},
"#,
            escape_string(&project.name),
            escape_string(&project.description),
            escape_string(&project.language),
            escape_string(&project.owner),
            escape_string(&project.repo),
        )
        .unwrap();
    }

    output.push_str("];\n");

    fs::write(&dest_path, output).expect("failed to write generated config");
}
