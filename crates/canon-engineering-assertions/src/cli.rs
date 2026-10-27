//! Clap definitions are public so documentation walks the actual command surface.
use crate::{Admission, Bundle, Change, GateResult, builtin_catalog, prepare_with};
use anyhow::{Context, Result, ensure};
use clap::{Args, Subcommand};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Subcommand)]
pub enum Assertions {
    /// List the released catalog's typed functions and recipes.
    List,
    /// Describe a released function's signature and implementation.
    Describe { name: String },
}
#[derive(Debug, Args)]
pub struct Input {
    /// Gates document, relative to root.
    #[arg(long, default_value = ".engineering/gates.yaml")]
    pub file: PathBuf,
    /// Source root whose dirty and untracked files enter the evidence identity.
    #[arg(long, default_value = ".")]
    pub root: PathBuf,
    /// Explicit command/provider/variable registration; omitted means no process admission.
    #[arg(long)]
    pub project: Option<PathBuf>,
    /// Revision the change is measured from; `git.changed` is unknown without it.
    #[arg(long)]
    pub base: Option<String>,
    /// Revision under test; `git.changed` is unknown without it.
    #[arg(long)]
    pub subject: Option<String>,
    /// The story the change belongs to; `story.id` is unknown without it.
    #[arg(long)]
    pub story_id: Option<String>,
    /// A tag of that story, repeatable; giving none leaves `story.has_tag` unknown.
    #[arg(long = "story-tag")]
    pub story_tags: Vec<String>,
}
impl Default for Input {
    fn default() -> Self {
        Self {
            file: PathBuf::from(".engineering/gates.yaml"),
            root: PathBuf::from("."),
            project: None,
            base: None,
            subject: None,
            story_id: None,
            story_tags: Vec::new(),
        }
    }
}
impl Input {
    fn admission(&self) -> Admission {
        Admission {
            change: Change {
                base: self.base.clone(),
                subject: self.subject.clone(),
                story_id: self.story_id.clone(),
                story_tags: (self.story_id.is_some() || !self.story_tags.is_empty())
                    .then(|| self.story_tags.clone()),
            },
        }
    }
}
#[derive(Debug, Subcommand)]
pub enum Gates {
    /// Parse, type-check and plan every assertion without collecting observations.
    Validate {
        #[command(flatten)]
        input: Input,
    },
    /// Explain the checked plan, resolved requests and bound implementation identities.
    Explain {
        #[command(flatten)]
        input: Input,
    },
    /// Collect all planned observations, retain evidence and evaluate the gate.
    Run {
        #[command(flatten)]
        input: Input,
        /// Evidence output under .engineering/assertions/, relative to root.
        #[arg(long, default_value = ".engineering/assertions/evidence.json")]
        out: PathBuf,
        /// Evaluation instant in Unix seconds; the edge reads the clock when omitted.
        #[arg(long)]
        now: Option<i64>,
    },
    /// Replay retained evidence without executing commands, providers or network operations.
    Evaluate {
        #[arg(long)]
        evidence: PathBuf,
        /// Defaults to the retained evaluation instant for reproducible replay.
        #[arg(long)]
        now: Option<i64>,
        /// Expected source identity. A mismatch makes the retained evidence unavailable.
        #[arg(long)]
        source_identity: Option<String>,
        #[arg(long)]
        context_identity: Option<String>,
    },
}
fn json(value: &impl serde::Serialize) -> Result<String> {
    Ok(serde_json::to_string_pretty(value)? + "\n")
}

pub fn assertions(command: Assertions) -> Result<(String, u8)> {
    let catalog = builtin_catalog()?;
    Ok((
        match command {
            Assertions::List => json(&catalog)?,
            Assertions::Describe { name } => json(
                catalog
                    .functions
                    .get(&name)
                    .with_context(|| format!("unknown assertion {name}"))?,
            )?,
        },
        0,
    ))
}
pub fn gates(command: Gates) -> Result<(String, u8)> {
    match command {
        Gates::Validate { input } => {
            let prepared = prepare_with(
                &input.root,
                &input.file,
                input.project.as_deref(),
                None,
                &input.admission(),
            )?;
            let assertions: usize = prepared
                .document
                .gates
                .iter()
                .map(|g| g.assertions.len())
                .sum();
            Ok((
                json(
                    &serde_json::json!({"valid":true,"gates":prepared.document.gates.len(),"assertions":assertions,"requests":prepared.plan.requests.len(),"plan_id":prepared.plan.id}),
                )?,
                0,
            ))
        }
        Gates::Explain { input } => {
            let prepared = prepare_with(
                &input.root,
                &input.file,
                input.project.as_deref(),
                None,
                &input.admission(),
            )?;
            Ok((json(&prepared.plan)?, 0))
        }
        Gates::Run { input, out, now } => {
            ensure!(
                out.is_relative()
                    && out
                        .components()
                        .all(|c| matches!(c, std::path::Component::Normal(_)))
                    && out.starts_with(".engineering/assertions")
                    && out.extension().is_some_and(|e| e == "json"),
                "evidence output must be a normal relative JSON path under .engineering/assertions/"
            );
            let mut checked = input.root.canonicalize()?;
            for component in out.components() {
                checked.push(component);
                if let Ok(metadata) = fs::symlink_metadata(&checked) {
                    ensure!(
                        !metadata.file_type().is_symlink(),
                        "evidence output may not traverse a symlink"
                    );
                }
            }
            ensure!(
                out != input.file && input.project.as_ref() != Some(&out),
                "evidence output cannot overwrite input"
            );
            let live_clock = now.is_none();
            let now = match now {
                Some(now) => now,
                None => i64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())?,
            };
            let prepared = prepare_with(
                &input.root,
                &input.file,
                input.project.as_deref(),
                Some(&out),
                &input.admission(),
            )?;
            let mut bundle = crate::collect(prepared, now)?;
            if live_clock {
                bundle.evaluated_at =
                    i64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())?;
            }
            let report = crate::evaluate_gates(&bundle, bundle.evaluated_at, None, None)?;
            let encoded = json(&bundle)?;
            ensure!(
                encoded.len() <= 64 * 1024 * 1024,
                "evidence exceeds 64 MiB retention limit"
            );
            crate::retention::write(&input.root, &out, encoded.as_bytes())?;
            let code = result_exit(report.result);
            Ok((json(&report)?, code))
        }
        Gates::Evaluate {
            evidence,
            now,
            source_identity,
            context_identity,
        } => {
            ensure!(
                fs::metadata(&evidence)?.len() <= 64 * 1024 * 1024,
                "evidence exceeds 64 MiB"
            );
            let data = crate::bounded_read(&evidence, 64 * 1024 * 1024)?;
            ensure!(data.len() <= 64 * 1024 * 1024, "evidence exceeds 64 MiB");
            let bundle: Bundle =
                serde_json::from_slice(&data).context("invalid retained evidence")?;
            let report = crate::evaluate_gates(
                &bundle,
                now.unwrap_or(bundle.evaluated_at),
                source_identity.as_deref(),
                context_identity.as_deref(),
            )?;
            let code = result_exit(report.result);
            Ok((json(&report)?, code))
        }
    }
}
/// 0 pass, 1 fail, 3 unknown, 4 not applicable (never 0: nothing was shown to hold).
fn result_exit(result: GateResult) -> u8 {
    match result {
        GateResult::Pass => 0,
        GateResult::Fail => 1,
        GateResult::Unknown => 3,
        GateResult::NotApplicable => 4,
    }
}
