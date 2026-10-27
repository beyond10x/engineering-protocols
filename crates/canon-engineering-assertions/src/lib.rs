//! Engineering acquisition is an explicit edge around Canon's pure assertion evaluator.
#![forbid(unsafe_code)]
pub mod cli;
pub mod documentation;
mod retention;
mod snapshot;

use anyhow::{Context, Result, bail, ensure};
pub use assertion_providers::Change;
use assertion_providers::{
    Collected, CommandBinding, ExternalProvider, NetworkBinding, ProviderContext,
};
use canon_expr::{
    Bindings, Catalog, EvalContext, Evaluation, Implementation, Observation, Outcome, Plan, Report,
    Truth, Value,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

pub const BUILTIN_CATALOG: &str = include_str!("../../../catalog/engineering.yaml");
const MAX_DOCUMENT: u64 = 4 * 1024 * 1024;

#[path = "../generated/types.rs"]
#[rustfmt::skip]
#[allow(dead_code,clippy::derivable_impls)]
mod contract;
pub use contract::{Document, EssPresence, Gate};

/// Project configuration is trusted admission data, not part of the expression language.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub format: String,
    #[serde(default)]
    pub variables: BTreeMap<String, Value>,
    #[serde(default)]
    pub catalogs: Vec<PathBuf>,
    /// Additional inputs (including ignored build/report files) consumed by external providers.
    #[serde(default)]
    pub inputs: Vec<PathBuf>,
    #[serde(default)]
    pub commands: BTreeMap<String, CommandBinding>,
    #[serde(default)]
    pub providers: BTreeMap<String, ExternalProvider>,
    #[serde(default)]
    pub networks: BTreeMap<String, NetworkBinding>,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
    #[serde(default = "default_max_output")]
    pub max_output_bytes: usize,
    #[serde(default = "default_freshness")]
    pub freshness_seconds: i64,
}
fn default_timeout() -> u64 {
    10
}
fn default_max_output() -> usize {
    4 * 1024 * 1024
}
fn default_freshness() -> i64 {
    300
}
impl Default for Project {
    fn default() -> Self {
        Self {
            format: "engineering-assertions-project/1".into(),
            variables: BTreeMap::new(),
            catalogs: vec![],
            inputs: vec![],
            commands: BTreeMap::new(),
            providers: BTreeMap::new(),
            networks: BTreeMap::new(),
            timeout_seconds: default_timeout(),
            max_output_bytes: default_max_output(),
            freshness_seconds: default_freshness(),
        }
    }
}

/// What the runtime knows when it admits a project: the change under test and, for story
/// delivery, the story it belongs to. Both enter the context identity, so evidence collected for
/// one change cannot be replayed as evidence for another.
#[derive(Debug, Clone, Default)]
pub struct Admission {
    pub change: Change,
}

/// One gate's result. `NotApplicable` is written only when the gate's `applies_when` evaluated to
/// false; it is never `Pass`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateResult {
    Pass,
    Fail,
    Unknown,
    NotApplicable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GateOutcome {
    pub id: String,
    pub result: GateResult,
    pub reasons: Vec<String>,
}

/// The run's report: Canon's per-expression evaluations, each gate's result and the aggregate.
/// `result` is `fail` when any gate fails, else `unknown` when any is unknown, else `pass` when any
/// passes, else `not_applicable`. `truth` is Canon's value for the aggregate and is `true` only for
/// `pass`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GatesReport {
    pub plan_id: String,
    pub truth: Truth,
    pub result: GateResult,
    pub gates: Vec<GateOutcome>,
    pub assertions: Vec<Evaluation>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bundle {
    pub format: String,
    pub document: Document,
    pub plan: Plan,
    pub source_identity: String,
    pub context_identity: String,
    pub evaluated_at: i64,
    pub observations: Vec<Observation>,
}

pub fn builtin_catalog() -> Result<Catalog> {
    Ok(serde_yaml_ng::from_str(BUILTIN_CATALOG)?)
}
pub fn read<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let data = read_bytes(path)?;
    serde_yaml_ng::from_slice(&data)
        .with_context(|| format!("{}: invalid document", path.display()))
}
fn read_bytes(path: &Path) -> Result<Vec<u8>> {
    bounded_read(path, MAX_DOCUMENT)
}
pub(crate) fn bounded_read(path: &Path, limit: u64) -> Result<Vec<u8>> {
    use std::io::Read;
    ensure!(
        fs::metadata(path)?.len() <= limit,
        "{}: document exceeds byte limit",
        path.display()
    );
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(limit + 1)
        .read_to_end(&mut bytes)?;
    ensure!(bytes.len() as u64 <= limit, "document grew past byte limit");
    Ok(bytes)
}
pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub struct Prepared {
    pub document: Document,
    pub plan: Plan,
    pub context: ProviderContext,
    pub bindings: Bindings,
    pub freshness_seconds: i64,
    root: PathBuf,
    excluded_output: Option<PathBuf>,
    source_inputs: BTreeSet<PathBuf>,
    executable_locks: BTreeMap<PathBuf, String>,
}

fn sources(document: &Document) -> Result<Vec<String>> {
    ensure!(
        document.format == "engineering-gates/1",
        "unsupported gates format"
    );
    ensure!(
        document.language == "canon-expr/1",
        "unsupported expression language"
    );
    ensure!(
        !document.gates.is_empty(),
        "a gates document cannot be empty"
    );
    let mut ids = BTreeSet::new();
    let mut sources = Vec::new();
    for gate in &document.gates {
        ensure!(
            !gate.id.trim().is_empty() && ids.insert(&gate.id),
            "empty or duplicate gate id"
        );
        ensure!(
            !gate.text.trim().is_empty(),
            "gate {} needs readable text",
            gate.id
        );
        ensure!(
            !gate.assertions.is_empty(),
            "gate {} has no assertions",
            gate.id
        );
        if let EssPresence::Present(applies_when) = &gate.applies_when {
            ensure!(
                !applies_when.trim().is_empty(),
                "gate {} has an empty applies_when",
                gate.id
            );
            sources.push(applies_when.clone());
        }
        sources.extend(gate.assertions.iter().cloned());
    }
    Ok(sources)
}

fn command_paths(project: &mut Project, root: &Path) -> Result<()> {
    for command in project
        .commands
        .values_mut()
        .chain(project.providers.values_mut().map(|p| &mut p.command))
    {
        ensure!(
            !command.executable.as_os_str().is_empty(),
            "empty executable"
        );
        if command.executable.is_relative() {
            command.executable = root.join(&command.executable);
        }
        command.executable = command
            .executable
            .canonicalize()
            .context("resolve admitted executable (PATH lookup is not used)")?;
        ensure!(command.executable.is_file(), "executable is not a file");
    }
    Ok(())
}

/// No observation runs here. Reading configuration and source identity is separate from acquisition.
pub fn prepare(
    root: &Path,
    file: &Path,
    project_file: Option<&Path>,
    excluded_output: Option<&Path>,
) -> Result<Prepared> {
    prepare_with(
        root,
        file,
        project_file,
        excluded_output,
        &Admission::default(),
    )
}

/// [`prepare`] with the runtime's admission data: the change under test and its story.
pub fn prepare_with(
    root: &Path,
    file: &Path,
    project_file: Option<&Path>,
    excluded_output: Option<&Path>,
    admission: &Admission,
) -> Result<Prepared> {
    let root = root.canonicalize()?;
    ensure!(
        !file.starts_with(".engineering/assertions")
            && !project_file.is_some_and(|p| p.starts_with(".engineering/assertions")),
        "gate/config inputs cannot be retained evidence paths"
    );
    let document: Document = read(&root.join(file))?;
    let expressions = sources(&document)?;
    let mut project: Project = match project_file {
        Some(path) => read(&root.join(path))?,
        None => Project::default(),
    };
    ensure!(
        project.format == "engineering-assertions-project/1",
        "unsupported project format"
    );
    ensure!(
        (1..=300).contains(&project.timeout_seconds),
        "timeout_seconds must be 1..300"
    );
    ensure!(
        (1..=16 * 1024 * 1024).contains(&project.max_output_bytes),
        "max_output_bytes must be 1..16777216"
    );
    ensure!(
        (1..=86400).contains(&project.freshness_seconds),
        "freshness_seconds must be 1..86400"
    );
    ensure!(
        project.catalogs.len() <= 64
            && project.providers.len() <= 1024
            && project.commands.len() <= 1024,
        "project registration limit exceeded"
    );
    command_paths(&mut project, &root)?;
    let mut catalog = builtin_catalog()?;
    for path in &project.catalogs {
        let extension: Catalog = read(&root.join(path))?;
        ensure!(
            extension.format == catalog.format,
            "unsupported extension catalog format"
        );
        for (name, function) in extension.functions {
            ensure!(
                !catalog.functions.contains_key(&name),
                "catalog function {name} is already defined"
            );
            if let Implementation::Observation(provider) = &function.implementation {
                ensure!(
                    provider.provider != "engineering.builtin/1",
                    "extension observations must name an external registered provider; use a recipe to compose built-ins"
                );
            }
            catalog.functions.insert(name, function);
        }
    }
    let mut executable_locks = BTreeMap::new();
    for command in project.commands.values() {
        executable_locks.insert(
            command.executable.clone(),
            assertion_providers::executable_digest(&command.executable)?,
        );
    }
    for (operation, provider) in &project.providers {
        ensure!(
            !provider.identity.is_empty(),
            "external provider identity is empty"
        );
        let actual = assertion_providers::executable_digest(&provider.command.executable)?;
        ensure!(
            actual == provider.sha256,
            "external provider {operation} executable digest differs from admission"
        );
        executable_locks.insert(provider.command.executable.clone(), actual);
    }
    // The executable includes this catalog and built-in collector code. Explicit bindings and
    // command bytes also enter the identity: a binary replacement cannot reuse prior evidence.
    let implementation = assertion_providers::executable_digest(&std::env::current_exe()?)?;
    let context_identity = digest(&serde_json::to_vec(&(
        &project,
        &executable_locks,
        &implementation,
        &admission.change,
    ))?);
    for function in catalog.functions.values_mut() {
        if let Implementation::Observation(provider) = &mut function.implementation {
            if provider.provider == "engineering.builtin/1" {
                ensure!(
                    !project.providers.contains_key(&provider.operation),
                    "external provider cannot replace built-in {}",
                    provider.operation
                );
                provider.digest = context_identity.clone();
            } else {
                let external = project
                    .providers
                    .get(&provider.operation)
                    .with_context(|| {
                        format!("unregistered external operation {}", provider.operation)
                    })?;
                ensure!(
                    provider.provider == external.identity && provider.digest == external.sha256,
                    "catalog provider identity/digest differs from explicit admission"
                );
                provider.digest = digest(&serde_json::to_vec(&(external, &context_identity))?);
            }
        }
    }
    let excluded_output = excluded_output.map(|p| root.join(p));
    let source_identity = "pending-source-snapshot".to_owned();
    let mut bindings = Bindings {
        values: project.variables.clone(),
        source_identity,
        context_identity,
    };
    let provisional = canon_expr::compile(&expressions, &catalog, &bindings)
        .map_err(|e| anyhow::anyhow!("{}: {e:?}", file.display()))?;
    let mut source_inputs: BTreeSet<PathBuf> = project
        .inputs
        .iter()
        .chain(project.catalogs.iter())
        .cloned()
        .collect();
    source_inputs.insert(file.to_owned());
    if let Some(path) = project_file {
        source_inputs.insert(path.to_owned());
    }
    for request in &provisional.requests {
        for key in ["path", "schema", "expected", "actual"] {
            if let Some(Value::String(path)) = request.arguments.get(key) {
                source_inputs.insert(PathBuf::from(path));
            }
        }
    }
    bindings.source_identity =
        source_identity_with_inputs(&root, excluded_output.as_deref(), &source_inputs)?;
    let plan = canon_expr::compile(&expressions, &catalog, &bindings)
        .map_err(|e| anyhow::anyhow!("{}: {e:?}", file.display()))?;
    let mut context = ProviderContext::new(&root);
    context.change = admission.change.clone();
    context.commands = project.commands;
    context.providers = project.providers;
    context.networks = project.networks;
    context.timeout = Duration::from_secs(project.timeout_seconds);
    context.max_output_bytes = project.max_output_bytes;
    // Refuse missing command/network admission before running any earlier request.
    for request in &plan.requests {
        if matches!(
            request.operation.as_str(),
            "process.run" | "process.succeeded" | "ess.run" | "tests.run"
        ) {
            let Some(Value::String(binding)) = request.arguments.get("binding") else {
                bail!("command binding must be a string")
            };
            ensure!(
                context.commands.contains_key(binding),
                "command binding {binding} is not registered"
            );
        }
        if request.operation == "net.tcp.reachable" {
            let Some(Value::String(network)) = request.arguments.get("network") else {
                bail!("network must be a string")
            };
            ensure!(
                context.networks.contains_key(network),
                "network {network} is not registered"
            );
            let Some(Value::String(host)) = request.arguments.get("host") else {
                bail!("host must be a string")
            };
            ensure!(
                context.networks[network].hosts.contains_key(host),
                "host {host} is not registered in network {network}"
            );
            let Some(Value::Integer(port)) = request.arguments.get("port") else {
                bail!("port must be an integer")
            };
            ensure!(
                port.parse::<u16>().is_ok_and(|p| p > 0),
                "TCP port must be 1..65535"
            );
        }
    }
    Ok(Prepared {
        document,
        plan,
        context,
        bindings,
        freshness_seconds: project.freshness_seconds,
        root,
        excluded_output,
        source_inputs,
        executable_locks,
    })
}

fn source_identity_with_inputs(
    root: &Path,
    output: Option<&Path>,
    inputs: &BTreeSet<PathBuf>,
) -> Result<String> {
    ensure!(inputs.len() <= 4096, "too many source inputs");
    let mut hashes = vec![(String::new(), snapshot::identity(root, output)?)];
    for input in inputs {
        ensure!(
            input.is_relative()
                && !input
                    .components()
                    .any(|c| matches!(c, std::path::Component::ParentDir)),
            "input path must stay below source root"
        );
        let path = root.join(input);
        ensure!(
            output != Some(path.as_path()) && !input.starts_with(".engineering/assertions"),
            "retained evidence cannot be a source input"
        );
        let mut ancestor = path.as_path();
        while !ancestor.exists() {
            ancestor = ancestor
                .parent()
                .context("input has no existing ancestor")?;
        }
        let resolved = ancestor.canonicalize()?;
        ensure!(
            resolved.starts_with(root),
            "input symlink escapes source root"
        );
        let hash = if path.exists() {
            snapshot::full_identity(&path.canonicalize()?)?
        } else {
            "absent".into()
        };
        hashes.push((input.to_string_lossy().into_owned(), hash));
    }
    Ok(digest(&serde_json::to_vec(&hashes)?))
}

pub fn collect(prepared: Prepared, now: i64) -> Result<Bundle> {
    let mut observations = Vec::new();
    let started = std::time::Instant::now();
    for request in &prepared.plan.requests {
        ensure!(
            started.elapsed() < Duration::from_secs(300),
            "total collection budget (300 seconds) exceeded"
        );
        verify_executables(&prepared.executable_locks)?;
        let args = request
            .arguments
            .iter()
            .map(|(k, v)| v.to_json().map(|v| (k.clone(), v)))
            .collect::<std::result::Result<BTreeMap<_, _>, _>>()
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;
        let mut context = prepared.context.clone();
        context.timeout = context
            .timeout
            .min(Duration::from_secs(300).saturating_sub(started.elapsed()));
        ensure!(
            !context.timeout.is_zero(),
            "total collection budget exceeded"
        );
        let collected = assertion_providers::collect(&request.operation, &args, &context)?;
        verify_executables(&prepared.executable_locks)?;
        let mut lifetime = prepared.freshness_seconds;
        let outcome = match collected {
            Collected::Known(value) => {
                if request.operation == "net.host.resolves" {
                    let ttl = value
                        .get("ttl_seconds")
                        .and_then(serde_json::Value::as_i64)
                        .context("DNS observation missing TTL")?;
                    ensure!(ttl >= 0, "negative DNS TTL");
                    lifetime = lifetime.min(ttl);
                }
                Outcome::Known(Value::from_json(&value, &request.result_type).map_err(|e| {
                    anyhow::anyhow!(
                        "provider {} returned invalid value: {e:?}",
                        request.operation
                    )
                })?)
            }
            Collected::Unavailable(reason) => Outcome::Unavailable(reason),
        };
        observations.push(Observation {
            request_id: request.id.clone(),
            plan_id: prepared.plan.id.clone(),
            observed_at: now,
            valid_until: Some(
                now.checked_add(lifetime)
                    .context("observation expiry overflow")?,
            ),
            outcome,
        });
    }
    ensure!(
        source_identity_with_inputs(
            &prepared.root,
            prepared.excluded_output.as_deref(),
            &prepared.source_inputs
        )? == prepared.bindings.source_identity,
        "source changed during collection; observations cannot certify this revision"
    );
    Ok(Bundle {
        format: "engineering-gate-evidence/1".into(),
        document: prepared.document,
        plan: prepared.plan,
        source_identity: prepared.bindings.source_identity,
        context_identity: prepared.bindings.context_identity,
        evaluated_at: now,
        observations,
    })
}

fn verify_executables(locks: &BTreeMap<PathBuf, String>) -> Result<()> {
    for (path, expected) in locks {
        ensure!(
            &assertion_providers::executable_digest(path)? == expected,
            "admitted executable changed after planning: {}",
            path.display()
        );
    }
    Ok(())
}

/// Pure replay. The caller chooses time and identities; this never invokes collectors or reads paths.
pub fn evaluate(
    bundle: &Bundle,
    now: i64,
    source_identity: Option<&str>,
    context_identity: Option<&str>,
) -> Result<Report> {
    ensure!(
        bundle.format == "engineering-gate-evidence/1",
        "unsupported evidence format"
    );
    let expected = sources(&bundle.document)?;
    let report = canon_expr::evaluate(
        &bundle.plan,
        &bundle.observations,
        &EvalContext {
            now,
            source_identity: source_identity.unwrap_or(&bundle.source_identity).into(),
            context_identity: context_identity.unwrap_or(&bundle.context_identity).into(),
        },
    )
    .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    ensure!(
        report
            .assertions
            .iter()
            .map(|a| &a.expression)
            .eq(expected.iter()),
        "bundle gate source differs from its checked plan"
    );
    Ok(report)
}

/// [`evaluate`] with each gate's result. A gate whose `applies_when` is true is judged by its
/// assertions; false makes it `not_applicable`; unknown makes it `unknown`. Pure replay, like
/// [`evaluate`].
pub fn evaluate_gates(
    bundle: &Bundle,
    now: i64,
    source_identity: Option<&str>,
    context_identity: Option<&str>,
) -> Result<GatesReport> {
    let report = evaluate(bundle, now, source_identity, context_identity)?;
    let mut cursor = 0;
    let mut gates = Vec::new();
    for gate in &bundle.document.gates {
        let applicability = match gate.applies_when {
            EssPresence::Present(_) => {
                cursor += 1;
                Some(&report.assertions[cursor - 1])
            }
            EssPresence::Absent => None,
        };
        let own = &report.assertions[cursor..cursor + gate.assertions.len()];
        cursor += gate.assertions.len();
        let (result, reasons) = match applicability.map(|a| a.truth) {
            Some(Truth::False) => (
                GateResult::NotApplicable,
                vec!["applies_when is false".into()],
            ),
            Some(Truth::Unknown) => (
                GateResult::Unknown,
                applicability.map(|a| a.reasons.clone()).unwrap_or_default(),
            ),
            Some(Truth::True) | None => {
                let reasons = own.iter().flat_map(|a| a.reasons.clone()).collect();
                if own.iter().any(|a| a.truth == Truth::False) {
                    (GateResult::Fail, reasons)
                } else if own.iter().all(|a| a.truth == Truth::True) {
                    (GateResult::Pass, vec![])
                } else {
                    (GateResult::Unknown, reasons)
                }
            }
        };
        gates.push(GateOutcome {
            id: gate.id.clone(),
            result,
            reasons,
        });
    }
    let has = |wanted| gates.iter().any(|g| g.result == wanted);
    let result = if has(GateResult::Fail) {
        GateResult::Fail
    } else if has(GateResult::Unknown) {
        GateResult::Unknown
    } else if has(GateResult::Pass) {
        GateResult::Pass
    } else {
        GateResult::NotApplicable
    };
    Ok(GatesReport {
        plan_id: report.plan_id,
        truth: match result {
            GateResult::Pass => Truth::True,
            GateResult::Fail => Truth::False,
            GateResult::Unknown | GateResult::NotApplicable => Truth::Unknown,
        },
        result,
        gates,
        assertions: report.assertions,
    })
}
