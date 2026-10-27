use crate::contract::{EssPresence, Request, Response, Status};
use crate::{
    Arguments, Collected, CommandBinding, ProviderContext, ProviderError, arg, executable_digest,
    known,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::Instant,
};

#[derive(Debug)]
pub(crate) struct Output {
    pub(crate) code: Option<i32>,
    pub(crate) stdout: Vec<u8>,
    pub(crate) stderr: Vec<u8>,
}
pub(super) fn collect(
    args: &Arguments,
    context: &ProviderContext,
) -> Result<Collected, ProviderError> {
    let binding = arg(args, "binding")?;
    let command = context
        .commands
        .get(binding)
        .ok_or_else(|| ProviderError::Invalid(format!("unregistered command binding {binding}")))?;
    match run(command, &[], context)? {
        None => Ok(Collected::Unavailable(
            "process timed out or exceeded output limit".into(),
        )),
        Some(out) => known(
            json!({"exit_code":out.code,"stdout":String::from_utf8(out.stdout).map_err(|e|ProviderError::Invalid(e.to_string()))?,"stderr":String::from_utf8(out.stderr).map_err(|e|ProviderError::Invalid(e.to_string()))?}),
        ),
    }
}
pub(super) fn external(
    operation: &str,
    args: &Arguments,
    context: &ProviderContext,
) -> Result<Collected, ProviderError> {
    let provider = &context.providers[operation];
    if provider.identity.is_empty()
        || provider.sha256.len() != 64
        || executable_digest(&provider.command.executable)? != provider.sha256
    {
        return Err(ProviderError::Invalid(
            "provider identity or executable digest mismatch".into(),
        ));
    }
    let input = serde_json::to_vec(&Request {
        format: "engineering-provider-request/1".into(),
        operation: operation.into(),
        arguments_json: serde_json::to_string(args)?,
    })?;
    if input.len() > context.max_output_bytes {
        return Err(ProviderError::Invalid(
            "provider request exceeds byte limit".into(),
        ));
    }
    let Some(out) = run(&provider.command, &input, context)? else {
        return Ok(Collected::Unavailable(
            "provider timed out or exceeded output limit".into(),
        ));
    };
    if out.code != Some(0) {
        return Ok(Collected::Unavailable(format!(
            "provider process failed with exit {:?}",
            out.code
        )));
    }
    let response: Response = serde_json::from_slice(&out.stdout)?;
    if response.format != "engineering-provider-response/1" {
        return Err(ProviderError::Invalid(
            "unsupported provider response format".into(),
        ));
    }
    match (*response.status, response.value_json, response.reason) {
        (Status::Known, EssPresence::Present(value), EssPresence::Absent) => {
            known(serde_json::from_str::<Value>(&value)?)
        }
        (Status::Unavailable, EssPresence::Absent, EssPresence::Present(reason))
            if !reason.is_empty() =>
        {
            Ok(Collected::Unavailable(reason))
        }
        _ => Err(ProviderError::Invalid(
            "inconsistent provider response status and payload".into(),
        )),
    }
}
#[cfg(unix)]
fn terminate(child: &mut std::process::Child) {
    unsafe {
        libc::kill(-(child.id() as i32), libc::SIGKILL);
    }
    let _ = child.kill();
    let _ = child.wait();
}
#[cfg(unix)]
struct ChildGuard(std::process::Child);
#[cfg(unix)]
impl Drop for ChildGuard {
    fn drop(&mut self) {
        terminate(&mut self.0);
    }
}
#[cfg(unix)]
fn nonblocking(pipe: &impl std::os::fd::AsRawFd) -> Result<(), ProviderError> {
    let fd = pipe.as_raw_fd();
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(())
}
#[cfg(unix)]
fn drain(
    pipe: &mut impl Read,
    bytes: &mut Vec<u8>,
    eof: &mut bool,
    limit: usize,
) -> Result<bool, ProviderError> {
    if *eof {
        return Ok(false);
    }
    let mut buffer = [0u8; 8192];
    match pipe.read(&mut buffer) {
        Ok(0) => *eof = true,
        Ok(n) => {
            if bytes.len() + n > limit {
                return Ok(true);
            }
            bytes.extend_from_slice(&buffer[..n]);
        }
        Err(e)
            if matches!(
                e.kind(),
                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
            ) => {}
        Err(e) => return Err(e.into()),
    }
    Ok(false)
}
#[cfg(not(unix))]
pub(crate) fn run(
    _binding: &CommandBinding,
    _input: &[u8],
    _context: &ProviderContext,
) -> Result<Option<Output>, ProviderError> {
    Err(ProviderError::Invalid(
        "bounded process supervision requires Unix in provider v1".into(),
    ))
}
#[cfg(unix)]
pub(crate) fn run(
    binding: &CommandBinding,
    input: &[u8],
    context: &ProviderContext,
) -> Result<Option<Output>, ProviderError> {
    use std::os::unix::process::CommandExt;
    if !binding.executable.is_absolute() {
        return Err(ProviderError::Invalid(match &binding.program {
            Some(program) => format!("program {program} was not resolved at admission"),
            None => "command executable must be an absolute admitted path".into(),
        }));
    }
    let mut command = Command::new(&binding.executable);
    command
        .args(&binding.args)
        .env_clear()
        .envs(&binding.env)
        .current_dir(&context.root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0);
    let mut child = ChildGuard(command.spawn()?);
    let deadline = Instant::now() + context.timeout;
    let mut stdout = child
        .0
        .stdout
        .take()
        .ok_or_else(|| ProviderError::Invalid("missing stdout pipe".into()))?;
    let mut stderr = child
        .0
        .stderr
        .take()
        .ok_or_else(|| ProviderError::Invalid("missing stderr pipe".into()))?;
    let mut stdin = child.0.stdin.take();
    nonblocking(&stdout)?;
    nonblocking(&stderr)?;
    if let Some(pipe) = &stdin {
        nonblocking(pipe)?;
    }
    let mut written = 0;
    let mut out = Vec::new();
    let mut err = Vec::new();
    let mut out_eof = false;
    let mut err_eof = false;
    let mut status = None;
    loop {
        if Instant::now() >= deadline {
            return Ok(None);
        }
        if let Some(pipe) = &mut stdin {
            if written == input.len() {
                stdin = None;
            } else {
                match pipe.write(&input[written..]) {
                    Ok(n) => written += n,
                    Err(e)
                        if matches!(
                            e.kind(),
                            std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
                        ) => {}
                    Err(e) => return Err(e.into()),
                }
            }
        }
        if drain(
            &mut stdout,
            &mut out,
            &mut out_eof,
            context.max_output_bytes,
        )? || drain(
            &mut stderr,
            &mut err,
            &mut err_eof,
            context.max_output_bytes,
        )? {
            return Ok(None);
        }
        if status.is_none() {
            status = child.0.try_wait()?;
        }
        if status.is_some() && out_eof && err_eof {
            return Ok(Some(Output {
                code: status.and_then(|s| s.code()),
                stdout: out,
                stderr: err,
            }));
        }
        thread::sleep(std::time::Duration::from_millis(2));
    }
}

/// A program name resolved at admission: where it was found and the digest of its bytes then.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedProgram {
    pub program: String,
    pub path: PathBuf,
    pub sha256: String,
}

fn refuse<T>(message: String) -> Result<T, ProviderError> {
    Err(ProviderError::Invalid(message))
}

/// Resolve a command that names a `program`. The project's own `search_path` is used when it has
/// entries, otherwise the runtime's default. Neither the caller's `PATH` nor a default of this
/// library's own is read. A command that names an `executable` is returned as it is.
pub fn admit_command(
    binding: &mut CommandBinding,
    root: &Path,
    search_path: &[PathBuf],
    runtime_default: &[PathBuf],
) -> Result<Option<ResolvedProgram>, ProviderError> {
    let Some(program) = binding.program.clone() else {
        return Ok(None);
    };
    if !binding.executable.as_os_str().is_empty() {
        return refuse(format!(
            "command names both program {program} and an executable"
        ));
    }
    let search_path = if search_path.is_empty() {
        runtime_default
    } else {
        search_path
    };
    if search_path.is_empty() {
        return refuse(format!(
            "program {program} cannot be resolved: no search_path is admitted, and a relative executable is refused"
        ));
    }
    let resolved = resolve_program(root, &program, search_path)?;
    binding.executable = resolved.path.clone();
    Ok(Some(resolved))
}

/// Find `program` on `search_path`. Every entry must be absolute and outside `root`, whether or not
/// it holds the program; the match must be an executable file whose real path is outside `root` too.
pub fn resolve_program(
    root: &Path,
    program: &str,
    search_path: &[PathBuf],
) -> Result<ResolvedProgram, ProviderError> {
    if program.is_empty()
        || program.len() > 255
        || program == "."
        || program == ".."
        || program.contains(['/', '\0'])
    {
        return refuse(format!("program must be a bare name, not {program:?}"));
    }
    let real_root = root
        .canonicalize()
        .map_err(|e| ProviderError::Invalid(format!("repository root: {e}")))?;
    let mut directories = Vec::new();
    for entry in search_path {
        if !entry.is_absolute() {
            return refuse(format!(
                "search_path entry {} must be absolute",
                entry.display()
            ));
        }
        let inside = |path: &Path| path.starts_with(&real_root) || path.starts_with(root);
        if inside(entry) {
            return refuse(format!(
                "search_path entry {} is inside the repository tree",
                entry.display()
            ));
        }
        match entry.canonicalize() {
            Ok(real) if inside(&real) => {
                return refuse(format!(
                    "search_path entry {} resolves into the repository tree",
                    entry.display()
                ));
            }
            Ok(real) => directories.push(real),
            Err(_) => {}
        }
    }
    for directory in directories {
        let candidate = directory.join(program);
        let Ok(real) = candidate.canonicalize() else {
            continue;
        };
        if !is_executable_file(&real) {
            continue;
        }
        if real.starts_with(&real_root) || real.starts_with(root) {
            return refuse(format!(
                "program {program} at {} resolves into the repository tree",
                candidate.display()
            ));
        }
        let sha256 = executable_digest(&candidate)?;
        return Ok(ResolvedProgram {
            program: program.to_owned(),
            path: candidate,
            sha256,
        });
    }
    refuse(format!(
        "program {program} not found on search_path [{}]",
        search_path
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    ))
}

#[cfg(unix)]
fn is_executable_file(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
}
#[cfg(not(unix))]
fn is_executable_file(path: &Path) -> bool {
    std::fs::metadata(path).is_ok_and(|m| m.is_file())
}
