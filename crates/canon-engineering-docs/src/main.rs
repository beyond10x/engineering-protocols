#![forbid(unsafe_code)]

//! Generates every derived page of the engineering protocols documentation site into the Docusaurus
//! docs tree
//! (`website/docs/`): one page per protocol document, compiled by Canon, the protocols index and
//! the vocabulary. The site build itself is Docusaurus; nothing here touches the network.

mod catalog_archive;
mod generate;
mod graph;
mod lint;
mod manifest;
mod markdown;
mod protocol;
mod routes;
mod status;

use std::path::PathBuf;

use anyhow::{Result, ensure};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(about = "Generate the derived pages of the engineering protocols documentation site")]
struct Cli {
    #[command(subcommand)]
    command: Action,
}

#[derive(Subcommand)]
enum Action {
    /// Render protocol pages, protocol graphs and the vocabulary into `<root>/website/`.
    Generate {
        /// The repository root that holds `protocols/` and `website/`.
        #[arg(long, default_value = ".")]
        root: PathBuf,
        /// Change nothing; fail when a generated page differs from a fresh render.
        #[arg(long)]
        check: bool,
    },
    /// Declare what a built site is: write `.well-known/b10x-site.json`, its route inventory
    /// `.well-known/b10x-routes.json` from the same commit (and `.nojekyll`) into it.
    SiteManifest {
        /// The built site, normally `website/build`.
        #[arg(long)]
        site: PathBuf,
        /// The full Git revision the site was built from.
        #[arg(long)]
        commit: String,
    },
    /// Write the protocol catalog as one reproducible `engineering-protocols-catalog-<version>.tar.gz`
    /// and its `.sha256` into `--out`: the files `protocols/manifest.yaml` lists, and the manifest.
    CatalogArchive {
        /// The repository root that holds `protocols/` and `schema/`.
        #[arg(long, default_value = ".")]
        root: PathBuf,
        /// The directory the archive and its digest file are written into; created when missing.
        #[arg(long)]
        out: PathBuf,
    },
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Action::Generate { root, check } => {
            let model = canon_engineering_assertions::documentation::expression_model();
            let model_path = root.join("ess/domains/expr.yaml");
            if check {
                ensure!(
                    std::fs::read_to_string(&model_path).ok().as_deref() == Some(model.as_str()),
                    "vendored assertion ESS model drift; run `canon-engineering-docs generate`"
                );
            } else {
                std::fs::create_dir_all(model_path.parent().expect("model has parent"))?;
                std::fs::write(&model_path, model)?;
            }
            let inputs = generate::Inputs::read(&root)?;
            let files = generate::render(&inputs)?;
            let drift = generate::apply(&root.join(generate::SITE), &files, check)?;
            let raw = lint::check_tree(&root.join(generate::SITE).join("docs"))?;
            ensure!(
                raw.is_empty(),
                "raw admonition openers (`:::kind Title` renders as text; write `:::kind[Title]`):\n  {}",
                raw.join("\n  ")
            );
            if check {
                println!(
                    "canon-engineering-docs: {} generated files fresh ({} protocols)",
                    files.len(),
                    inputs.protocols.len()
                );
            } else {
                println!(
                    "canon-engineering-docs: {} generated files, {} updated ({} protocols)",
                    files.len(),
                    drift.len(),
                    inputs.protocols.len()
                );
            }
        }
        Action::SiteManifest { site, commit } => {
            let routes = manifest::write(&site, &commit)?;
            println!(
                "canon-engineering-docs: {} declared as engineering-protocols at {commit}, {routes} routes",
                site.display()
            );
        }
        Action::CatalogArchive { root, out } => {
            let written = catalog_archive::write(&root, &out)?;
            println!(
                "canon-engineering-docs: {} files in {}/{}, sha256 {}",
                written.files,
                out.display(),
                written.archive,
                written.sha256
            );
        }
    }
    Ok(())
}
