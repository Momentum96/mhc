use maho_cli::cli::host_command::*;

#[cfg(windows)]
#[tokio::test]
async fn windows_host_session_list_authenticates_and_returns_session_rows() {
    use maho_rpc::{host_lifecycle::win32_pipe, socket_transport};
    use serde_json::{Value, json};
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("host.sock");
    let endpoint = path.to_str().unwrap();
    let key_path = socket_transport::socket_secret_path(endpoint).unwrap();
    let secret = socket_transport::create_socket_secret(std::path::Path::new(&key_path)).unwrap();
    let address =
        socket_transport::resolve_socket_transport_address(endpoint, "win32", Some(&secret))
            .unwrap();
    let mut pipe = win32_pipe::create_first_instance(&address).unwrap();
    let (finish, finished) = tokio::sync::oneshot::channel();
    let serve = async {
        tokio::time::timeout(std::time::Duration::from_secs(10), pipe.connect())
            .await
            .expect("CLI connected to the Windows named pipe")
            .unwrap();
        socket_transport::authenticate_socket(&mut pipe, &secret)
            .await
            .unwrap();
        let (read, mut write) = tokio::io::split(pipe);
        let mut lines = BufReader::new(read).lines();
        for (kind, data) in [
            (
                "list_sessions",
                json!({"sessions":[{"sessionId":"rpc-win-test"}]}),
            ),
            (
                "get_protocol_info",
                json!({"capabilities":["detach_connection"]}),
            ),
            ("detach_connection", json!({"deferred":[]})),
        ] {
            let line = tokio::time::timeout(std::time::Duration::from_secs(3), lines.next_line())
                .await
                .unwrap_or_else(|_| panic!("timed out waiting for CLI command {kind}"))
                .unwrap()
                .unwrap_or_else(|| panic!("CLI exited before sending {kind}"));
            let request: Value = serde_json::from_str(&line).unwrap();
            assert_eq!(request["type"], kind);
            let response = json!({"type":"response","id":request["id"],"success":true,"data":data});
            write
                .write_all(format!("{response}\n").as_bytes())
                .await
                .unwrap();
        }
        // Leave the pipe open until the client has received and processed the detach ack.
        finished.await.unwrap();
    };
    let run = async {
        let result = tokio::process::Command::new(env!("CARGO_BIN_EXE_mhc"))
            .args(["host", "session", "list", "--socket", endpoint, "--json"])
            .env("HOME", dir.path())
            .env("USERPROFILE", dir.path())
            .env("OMO_CODING_AGENT_DIR", dir.path())
            .kill_on_drop(true)
            .output()
            .await
            .unwrap();
        finish.send(()).unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let reply: Value = serde_json::from_slice(&result.stdout).unwrap();
        assert_eq!(reply["sessions"][0]["sessionId"], "rpc-win-test");
    };
    tokio::time::timeout(std::time::Duration::from_secs(25), async {
        tokio::join!(run, serve);
    })
    .await
    .expect("Windows CLI completed the authenticated named-pipe round trip");
}

#[test]
fn drain_wait_accepts_upstream_numeric_forms_but_rejects_unsafe_integers() {
    for (text, expected) in [(" 1e2 ", 100), ("0x20", 32), ("+3.0", 3)] {
        let args = ["stop", "--drain", "--wait-ms", text].map(str::to_owned);
        assert_eq!(parse_host_args(&args).unwrap().wait_ms, Some(expected));
    }
    for text in ["0", "1.5", "0x+1", "9007199254740992", "Infinity"] {
        let args = ["stop", "--drain", "--wait-ms", text].map(str::to_owned);
        assert!(parse_host_args(&args).is_err(), "{text}");
    }
}

#[test]
fn ensure_cannot_claim_an_extension_with_no_native_port() {
    let temp = tempfile::tempdir().unwrap();
    let spec = temp.path().join("spec.json");
    std::fs::write(
        temp.path().join("arbitrary-unported-script.ts"),
        "export default () => {};",
    )
    .unwrap();
    std::fs::write(&spec, r#"{"spec_version":1,"core":{"session_runtime":"worker","multi_session":true,"extensions":["arbitrary-unported-script.ts"]}}"#).unwrap();
    let args = ["ensure", "--launch-spec", spec.to_str().unwrap()].map(str::to_owned);
    let parsed = parse_host_args(&args).unwrap();
    let Err(error) =
        maho_cli::cli::host_request::build_host_request(&parsed, "unused.sock", temp.path())
    else {
        panic!("a launch request must not claim an unported extension");
    };
    assert_eq!(error.exit_code(), 1);
    assert_eq!(error.payload()["reason"], "host_error");
    assert!(
        error.payload()["detail"]
            .as_str()
            .unwrap()
            .contains("has no native port")
    );
}
#[test]
fn host_options_parse_and_last_policy_wins() {
    let args = [
        "ensure",
        "--json",
        "--policy",
        "never",
        "--socket",
        "--literal",
        "--force",
    ]
    .map(str::to_owned);
    let parsed = parse_host_args(&args).unwrap();
    assert_eq!(parsed.subcommand, HostSubcommand::Ensure);
    assert_eq!(parsed.policy, "never");
    assert_eq!(parsed.socket.as_deref(), Some("--literal"));
    assert!(parsed.force);
}
#[test]
fn host_missing_or_invalid_policy_is_error() {
    assert!(parse_host_args(&[]).is_err());
    assert!(
        parse_host_args(&[
            "ensure".to_owned(),
            "--policy".to_owned(),
            "invalid".to_owned()
        ])
        .is_err()
    );
}
// senpi v2026.10.27-2 host-command-args.ts: presenter flags (ensure/handoff only) and --rpc-key-root.
#[test]
fn presenter_flags_parse_only_for_ensure_and_handoff() {
    for sub in ["ensure", "handoff"] {
        let args = [
            sub,
            "--mcp-presenter-fd",
            "4",
            "--mcp-presenter-enforcement",
        ]
        .map(str::to_owned);
        let parsed = parse_host_args(&args).unwrap();
        assert_eq!(parsed.presenter_bootstrap_fd, Some(4));
        assert!(parsed.mcp_presenter_enforcement);
    }
    assert!(
        parse_host_args(&["status", "--mcp-presenter-enforcement"].map(str::to_owned)).is_err()
    );
    assert!(parse_host_args(&["ensure", "--mcp-presenter-fd", "3"].map(str::to_owned)).is_err());
    let parsed = parse_host_args(&["ensure".to_owned()]).unwrap();
    assert_eq!(parsed.presenter_bootstrap_fd, None);
    assert!(!parsed.mcp_presenter_enforcement);
}
#[test]
fn rpc_key_root_resolves_to_an_absolute_path_on_every_subcommand() {
    let parsed = parse_host_args(&["status", "--rpc-key-root", "keys"].map(str::to_owned)).unwrap();
    let root = parsed.key_root.expect("key root");
    assert!(std::path::Path::new(&root).is_absolute());
    assert!(root.ends_with("keys"));
    assert!(parse_host_args(&["status", "--rpc-key-root", "--json"].map(str::to_owned)).is_err());
    assert!(
        parse_host_args(&["status".to_owned()])
            .unwrap()
            .key_root
            .is_none()
    );
}
// senpi `runHostCommand`: request options ride the runner instead of being refused.
#[test]
fn request_options_carry_the_key_root_and_enforcement_to_the_runner() {
    let plain = parse_host_args(&["ensure".to_owned()]).unwrap();
    let options = maho_cli::cli::host_request::request_options(&plain);
    assert!(options.key_root.is_none() && !options.mcp_presenter_enforcement);
    assert!(options.presenter_bootstrap.is_none());
    let named = parse_host_args(
        &[
            "ensure",
            "--mcp-presenter-enforcement",
            "--rpc-key-root",
            "keys",
        ]
        .map(str::to_owned),
    )
    .unwrap();
    let options = maho_cli::cli::host_request::request_options(&named);
    assert!(options.mcp_presenter_enforcement);
    let root = options.key_root.expect("key root");
    assert!(root.is_absolute() && root.ends_with("keys"));
}
