use assertion_providers::{Collected, ProviderContext, collect};
use serde_json::json;
use std::collections::BTreeMap;
#[test]
fn absent_file_is_known_false_and_document_malformed_is_error() {
    let root = tempfile::tempdir().unwrap();
    let context = ProviderContext::new(root.path());
    let args = BTreeMap::from([("path".into(), json!("missing"))]);
    assert_eq!(
        collect("fs.file.exists", &args, &context).unwrap(),
        Collected::Known(json!(false))
    );
    std::fs::write(root.path().join("bad.json"), "{").unwrap();
    let args = BTreeMap::from([("path".into(), json!("bad.json"))]);
    assert!(collect("json.read", &args, &context).is_err());
}
#[test]
fn empty_test_run_never_certifies_and_generated_extra_file_is_detected() {
    let root = tempfile::tempdir().unwrap();
    let context = ProviderContext::new(root.path());
    std::fs::write(
        root.path().join("tests.json"),
        r#"{"format":"engineering-tests/1","inventory":[],"passed":0,"failed":0,"skipped":0}"#,
    )
    .unwrap();
    let args = BTreeMap::from([("path".into(), json!("tests.json"))]);
    assert!(collect("tests.report", &args, &context).is_err());
    std::fs::create_dir(root.path().join("expected")).unwrap();
    std::fs::create_dir(root.path().join("actual")).unwrap();
    std::fs::write(root.path().join("actual/extra"), "extra").unwrap();
    let args = BTreeMap::from([
        ("expected".into(), json!("expected")),
        ("actual".into(), json!("actual")),
    ]);
    assert_eq!(
        collect("generated.matches", &args, &context).unwrap(),
        Collected::Known(json!(false))
    );
}

fn args(pairs: &[(&str, serde_json::Value)]) -> BTreeMap<String, serde_json::Value> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_owned(), v.clone()))
        .collect()
}
#[test]
fn document_access_preserves_exact_numbers_and_local_schema() {
    let root = tempfile::tempdir().unwrap();
    let context = ProviderContext::new(root.path());
    std::fs::write(
        root.path().join("doc.json"),
        r#"{"number":9007199254740993,"ready":true}"#,
    )
    .unwrap();
    std::fs::write(
        root.path().join("schema.json"),
        r#"{"type":"object","required":["number"],"properties":{"number":{"type":"integer"}}}"#,
    )
    .unwrap();
    assert_eq!(
        collect(
            "document.get",
            &args(&[("path", json!("doc.json")), ("pointer", json!("/number"))]),
            &context
        )
        .unwrap(),
        Collected::Known(json!(9007199254740993u64))
    );
    assert_eq!(
        collect(
            "document.validates",
            &args(&[
                ("path", json!("doc.json")),
                ("schema", json!("schema.json"))
            ]),
            &context
        )
        .unwrap(),
        Collected::Known(json!(true))
    );
    std::fs::write(
        root.path().join("schema.json"),
        r#"{"$ref":"https://example.invalid/schema.json"}"#,
    )
    .unwrap();
    assert!(
        collect(
            "document.validates",
            &args(&[
                ("path", json!("doc.json")),
                ("schema", json!("schema.json"))
            ]),
            &context
        )
        .is_err()
    );
}
#[test]
fn file_paths_cannot_escape_and_generated_missing_files_fail() {
    let root = tempfile::tempdir().unwrap();
    let context = ProviderContext::new(root.path());
    assert!(
        collect(
            "fs.file.exists",
            &args(&[("path", json!("../outside"))]),
            &context
        )
        .is_err()
    );
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("/etc", root.path().join("escape")).unwrap();
        assert!(
            collect(
                "fs.file.exists",
                &args(&[("path", json!("escape/passwd"))]),
                &context
            )
            .is_err()
        );
    }
    std::fs::create_dir(root.path().join("expected")).unwrap();
    std::fs::create_dir(root.path().join("actual")).unwrap();
    std::fs::write(root.path().join("expected/missing"), "x").unwrap();
    assert_eq!(
        collect(
            "generated.matches",
            &args(&[("expected", json!("expected")), ("actual", json!("actual"))]),
            &context
        )
        .unwrap(),
        Collected::Known(json!(false))
    );
}
#[test]
fn test_inventory_counts_and_duplicate_names_are_checked() {
    let root = tempfile::tempdir().unwrap();
    let context = ProviderContext::new(root.path());
    for inventory in [json!(["one", "one"]), json!(["one"]), json!([])] {
        std::fs::write(root.path().join("tests.json"),json!({"format":"engineering-tests/1","inventory":inventory,"passed":2,"failed":0,"skipped":0}).to_string()).unwrap();
        assert!(
            collect(
                "tests.report",
                &args(&[("path", json!("tests.json"))]),
                &context
            )
            .is_err()
        );
    }
}
#[test]
fn registered_external_namespace_uses_same_collector_and_checks_identity() {
    use assertion_providers::{CommandBinding, ExternalProvider, executable_digest};
    let root = tempfile::tempdir().unwrap();
    let mut context = ProviderContext::new(root.path());
    let executable = std::path::PathBuf::from(env!("CARGO_BIN_EXE_assertion-provider-example"));
    let provider = ExternalProvider {
        identity: "example/1".into(),
        sha256: executable_digest(&executable).unwrap(),
        command: CommandBinding {
            program: None,
            argv0: None,
            executable,
            args: vec![],
            env: BTreeMap::new(),
        },
    };
    context.providers.insert("example.echo".into(), provider);
    assert_eq!(
        collect(
            "example.echo",
            &args(&[("value", json!({"answer":42}))]),
            &context
        )
        .unwrap(),
        Collected::Known(json!({"answer":42}))
    );
    context.providers.get_mut("example.echo").unwrap().sha256 = "0".repeat(64);
    assert!(collect("example.echo", &args(&[("value", json!(true))]), &context).is_err());
}
#[cfg(unix)]
#[test]
fn process_timeout_output_limit_and_explicit_environment() {
    use assertion_providers::CommandBinding;
    use std::time::{Duration, Instant};
    let root = tempfile::tempdir().unwrap();
    let mut context = ProviderContext::new(root.path());
    context.timeout = Duration::from_millis(50);
    for (binding, executable, argv) in [
        ("sleep", "/usr/bin/sleep", vec!["2".into()]),
        ("overflow", "/usr/bin/yes", vec![]),
        ("env", "/usr/bin/env", vec![]),
    ] {
        context.commands.insert(
            binding.into(),
            CommandBinding {
                program: None,
                argv0: None,
                executable: executable.into(),
                args: argv,
                env: BTreeMap::new(),
            },
        );
    }
    let started = Instant::now();
    assert!(matches!(
        collect(
            "process.run",
            &args(&[("binding", json!("sleep"))]),
            &context
        )
        .unwrap(),
        Collected::Unavailable(_)
    ));
    assert!(started.elapsed() < Duration::from_secs(1));
    context.max_output_bytes = 512;
    assert!(matches!(
        collect(
            "process.run",
            &args(&[("binding", json!("overflow"))]),
            &context
        )
        .unwrap(),
        Collected::Unavailable(_)
    ));
    let Collected::Known(env) =
        collect("process.run", &args(&[("binding", json!("env"))]), &context).unwrap()
    else {
        panic!("environment command unavailable")
    };
    assert_eq!(env["stdout"], "");
}
#[cfg(unix)]
#[test]
fn tests_run_reports_conflicting_process_status_as_error() {
    use assertion_providers::CommandBinding;
    let root = tempfile::tempdir().unwrap();
    let mut context = ProviderContext::new(root.path());
    let report=json!({"format":"engineering-tests/1","inventory":["one"],"passed":0,"failed":1,"skipped":0}).to_string();
    context.commands.insert(
        "suite".into(),
        CommandBinding {
            program: None,
            argv0: None,
            executable: "/usr/bin/printf".into(),
            args: vec!["%s".into(), report],
            env: BTreeMap::new(),
        },
    );
    assert!(collect("tests.run", &args(&[("binding", json!("suite"))]), &context).is_err());
}
#[test]
fn explicit_network_context_accepts_listener_and_refusal_is_false() {
    use assertion_providers::NetworkBinding;
    use std::net::{IpAddr, Ipv4Addr, TcpListener};
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let root = tempfile::tempdir().unwrap();
    let mut context = ProviderContext::new(root.path());
    context.networks.insert(
        "local".into(),
        NetworkBinding {
            hosts: BTreeMap::from([("service".into(), IpAddr::V4(Ipv4Addr::LOCALHOST))]),
        },
    );
    let input = args(&[
        ("host", json!("service")),
        ("port", json!(address.port())),
        ("network", json!("local")),
    ]);
    assert_eq!(
        collect("net.tcp.reachable", &input, &context).unwrap(),
        Collected::Known(json!(true))
    );
    let (accepted, _) = listener.accept().unwrap();
    drop(accepted);
    drop(listener);
    // A bound socket that never listens reserves the port without a close/exec race when other
    // parallel tests spawn children. It is a deterministic local connection-refused fixture.
    let reserved =
        socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::STREAM, None).unwrap();
    reserved
        .bind(
            &"127.0.0.1:0"
                .parse::<std::net::SocketAddr>()
                .unwrap()
                .into(),
        )
        .unwrap();
    let mut input = input;
    input.insert(
        "port".into(),
        json!(reserved.local_addr().unwrap().as_socket().unwrap().port()),
    );
    assert_eq!(
        collect("net.tcp.reachable", &input, &context).unwrap(),
        Collected::Known(json!(false))
    );
}
#[test]
fn dns_retains_ttl_and_definitive_negative_but_timeout_is_unknown() {
    use std::{net::UdpSocket, thread, time::Duration};
    let server = UdpSocket::bind("127.0.0.1:0").unwrap();
    let resolver = server.local_addr().unwrap();
    let worker = thread::spawn(move || {
        for _ in 0..2 {
            let mut query = [0u8; 512];
            let (n, peer) = server.recv_from(&mut query).unwrap();
            let mut response = query[..n].to_vec();
            response[2] = 0x81;
            response[3] = 0x80;
            response[7] = 1;
            response.extend_from_slice(&[0xc0, 0x0c, 0, 1, 0, 1, 0, 0, 0, 42, 0, 4, 127, 0, 0, 1]);
            server.send_to(&response, peer).unwrap();
        }
    });
    let root = tempfile::tempdir().unwrap();
    let mut context = ProviderContext::new(root.path());
    let input = args(&[
        ("host", json!("api.example.org")),
        ("resolver", json!(resolver.to_string())),
    ]);
    let Collected::Known(value) = collect("net.host.resolves", &input, &context).unwrap() else {
        panic!("DNS unknown")
    };
    assert_eq!(value["resolves"], true);
    assert_eq!(value["ttl_seconds"], 42);
    worker.join().unwrap();
    let server = UdpSocket::bind("127.0.0.1:0").unwrap();
    let input = args(&[
        ("host", json!("absent.example.org")),
        ("resolver", json!(server.local_addr().unwrap().to_string())),
    ]);
    let worker = thread::spawn(move || {
        let mut query = [0; 512];
        let (n, peer) = server.recv_from(&mut query).unwrap();
        query[2] = 0x81;
        query[3] = 0x83;
        server.send_to(&query[..n], peer).unwrap();
    });
    let Collected::Known(value) = collect("net.host.resolves", &input, &context).unwrap() else {
        panic!("DNS negative unknown")
    };
    assert_eq!(value["resolves"], false);
    worker.join().unwrap();
    let server = UdpSocket::bind("127.0.0.1:0").unwrap();
    context.timeout = Duration::from_millis(20);
    let input = args(&[
        ("host", json!("timeout.example.org")),
        ("resolver", json!(server.local_addr().unwrap().to_string())),
    ]);
    assert!(matches!(
        collect("net.host.resolves", &input, &context).unwrap(),
        Collected::Unavailable(_)
    ));
}

#[test]
fn actual_codegate_dependency_report_and_ess_report_are_admitted() {
    let root = tempfile::tempdir().unwrap();
    let context = ProviderContext::new(root.path());
    for (file, bytes, operation) in [
        (
            "codegate.json",
            include_str!("fixtures/codegate-report.json"),
            "codegate.dependencies",
        ),
        (
            "ess.json",
            include_str!("fixtures/ess-report.json"),
            "ess.report",
        ),
    ] {
        std::fs::write(root.path().join(file), bytes).unwrap();
        assert!(matches!(
            collect(operation, &args(&[("path", json!(file))]), &context).unwrap(),
            Collected::Known(_)
        ));
    }
    let mut report: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/ess-report.json")).unwrap();
    report["counts"]["passed"] = json!(0);
    std::fs::write(root.path().join("ess.json"), report.to_string()).unwrap();
    assert!(
        collect(
            "ess.report",
            &args(&[("path", json!("ess.json"))]),
            &context
        )
        .is_err()
    );
    let mut report: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/codegate-report.json")).unwrap();
    report["coverage"] = json!("Partial");
    std::fs::write(root.path().join("codegate.json"), report.to_string()).unwrap();
    assert!(
        collect(
            "codegate.dependencies",
            &args(&[("path", json!("codegate.json"))]),
            &context
        )
        .is_err()
    );
}

#[test]
fn missing_report_is_unavailable_but_malformed_report_is_a_hard_error() {
    let root = tempfile::tempdir().unwrap();
    let context = ProviderContext::new(root.path());
    let input = args(&[("path", json!("missing.json"))]);
    assert!(matches!(
        collect("tests.report", &input, &context).unwrap(),
        Collected::Unavailable(_)
    ));
    std::fs::write(root.path().join("missing.json"), "{}").unwrap();
    assert!(collect("tests.report", &input, &context).is_err());
}
#[test]
fn yaml_never_silently_rounds_decimal_observations() {
    let root = tempfile::tempdir().unwrap();
    let context = ProviderContext::new(root.path());
    std::fs::write(
        root.path().join("numbers.yaml"),
        "exact: 0.123456789012345678901\n",
    )
    .unwrap();
    assert!(
        collect(
            "yaml.read",
            &args(&[("path", json!("numbers.yaml"))]),
            &context
        )
        .is_err()
    );
}

#[cfg(unix)]
#[test]
fn success_boolean_and_malformed_external_envelopes_are_checked() {
    use assertion_providers::{CommandBinding, ExternalProvider, executable_digest};
    let root = tempfile::tempdir().unwrap();
    let mut context = ProviderContext::new(root.path());
    for (name, exe, expected) in [
        ("yes", "/usr/bin/true", true),
        ("no", "/usr/bin/false", false),
    ] {
        context.commands.insert(
            name.into(),
            CommandBinding {
                program: None,
                argv0: None,
                executable: exe.into(),
                args: vec![],
                env: BTreeMap::new(),
            },
        );
        assert_eq!(
            collect(
                "process.succeeded",
                &args(&[("binding", json!(name))]),
                &context
            )
            .unwrap(),
            Collected::Known(json!(expected))
        );
    }
    // Consume the request before replying. A printf fixture can exit before the
    // parent writes stdin, testing broken-pipe unavailability instead of parsing.
    let source = root.path().join("malformed_provider.rs");
    std::fs::write(
        &source,
        r#"use std::io::{Read, Write};
fn main() {
    let mut request = String::new();
    std::io::stdin().read_to_string(&mut request).unwrap();
    assert!(request.contains("engineering-provider-request/1"));
    let response = std::fs::read("response.json").unwrap();
    std::io::stdout().write_all(&response).unwrap();
}"#,
    )
    .unwrap();
    let executable = root.path().join("malformed-provider");
    assert!(
        std::process::Command::new("rustc")
            .args(["--edition=2024", "--crate-name", "malformed_provider"])
            .arg(&source)
            .arg("-o")
            .arg(&executable)
            .status()
            .unwrap()
            .success()
    );
    for response in [
        json!({"format":"engineering-provider-response/1","status":"known","value_json":"true","reason":"also unknown"}),
        json!({"format":"engineering-provider-response/1","status":"known","value_json":"true","extra":1}),
        json!({"format":"engineering-provider-response/1","status":"known","value_json":true}),
    ] {
        std::fs::write(root.path().join("response.json"), response.to_string()).unwrap();
        context.providers.insert(
            "bad.reply".into(),
            ExternalProvider {
                identity: "bad/1".into(),
                sha256: executable_digest(&executable).unwrap(),
                command: CommandBinding {
                    program: None,
                    argv0: None,
                    executable: executable.clone(),
                    args: vec![],
                    env: BTreeMap::new(),
                },
            },
        );
        let result = collect("bad.reply", &BTreeMap::new(), &context);
        assert!(result.is_err(), "malformed response {response}: {result:?}");
    }
}
