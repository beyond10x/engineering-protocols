//! Adversary pass 1 on story:ess-0-69-2-pin. ESS 0.69.2 renamed the generated provider
//! `Status` variants from `V0`/`V1` to `Known`/`Unavailable`. These cases pin the wire names
//! and drive `process::external` from the README's response contract ("Unavailable responses
//! carry `reason` and omit `value_json`. Known responses omit `reason`."), so a swapped or
//! dropped status check in the response match is caught.
#![cfg(unix)]

use assertion_providers::contract::{Response, Status};
use assertion_providers::{
    Collected, CommandBinding, ExternalProvider, ProviderContext, collect, executable_digest,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[test]
fn generated_status_keeps_the_wire_names() {
    assert_eq!(serde_json::to_value(Status::Known).unwrap(), json!("known"));
    assert_eq!(
        serde_json::to_value(Status::Unavailable).unwrap(),
        json!("unavailable")
    );
    assert_eq!(
        serde_json::from_value::<Status>(json!("known")).unwrap(),
        Status::Known
    );
    assert_eq!(
        serde_json::from_value::<Status>(json!("unavailable")).unwrap(),
        Status::Unavailable
    );
    for rust_name in ["Known", "Unavailable", "V0", "V1"] {
        assert!(
            serde_json::from_value::<Status>(json!(rust_name)).is_err(),
            "{rust_name} is a Rust name, not a wire name"
        );
    }
    let response: Response = serde_json::from_value(json!({
        "format": "engineering-provider-response/1",
        "status": "unavailable",
        "reason": "down"
    }))
    .unwrap();
    assert_eq!(*response.status, Status::Unavailable);
}

fn provider_context(root: &std::path::Path) -> ProviderContext {
    let source = root.join("reply_provider.rs");
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
    let executable = root.join("reply-provider");
    assert!(
        std::process::Command::new("rustc")
            .args(["--edition=2024", "--crate-name", "reply_provider"])
            .arg(&source)
            .arg("-o")
            .arg(&executable)
            .status()
            .unwrap()
            .success()
    );
    let mut context = ProviderContext::new(root);
    context.providers.insert(
        "adv.reply".into(),
        ExternalProvider {
            identity: "adv/1".into(),
            sha256: executable_digest(&executable).unwrap(),
            command: CommandBinding {
                executable,
                args: vec![],
                env: BTreeMap::new(),
            },
        },
    );
    context
}

fn reply(
    root: &std::path::Path,
    context: &ProviderContext,
    response: &Value,
) -> Result<Collected, assertion_providers::ProviderError> {
    std::fs::write(root.join("response.json"), response.to_string()).unwrap();
    collect("adv.reply", &BTreeMap::new(), context)
}

#[test]
fn external_response_status_and_payload_follow_the_documented_contract() {
    let root = tempfile::tempdir().unwrap();
    let context = provider_context(root.path());
    let format = "engineering-provider-response/1";

    assert_eq!(
        reply(
            root.path(),
            &context,
            &json!({"format":format,"status":"known","value_json":"false"})
        )
        .unwrap(),
        Collected::Known(json!(false))
    );
    assert_eq!(
        reply(
            root.path(),
            &context,
            &json!({"format":format,"status":"unavailable","reason":"provider offline"})
        )
        .unwrap(),
        Collected::Unavailable("provider offline".into())
    );

    for response in [
        json!({"format":format,"status":"unavailable","value_json":"true"}),
        json!({"format":format,"status":"unavailable","value_json":"true","reason":"down"}),
        json!({"format":format,"status":"unavailable"}),
        json!({"format":format,"status":"unavailable","reason":""}),
        json!({"format":format,"status":"known","reason":"down"}),
        json!({"format":format,"status":"known"}),
        json!({"format":format,"status":"Known","value_json":"true"}),
        json!({"format":format,"status":"V0","value_json":"true"}),
    ] {
        let result = reply(root.path(), &context, &response);
        assert!(
            result.is_err(),
            "inconsistent response {response} must be refused, got {result:?}"
        );
    }
}
