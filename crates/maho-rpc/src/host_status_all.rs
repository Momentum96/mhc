//! `host status --all`: one status row per endpoint an agent directory holds state for (senpi `host-status-all.ts`).
//!
//! Enumeration is the daemon directory (`list_host_endpoints`) and every row is the single-socket report read
//! with `prune: false`, so this answer removes NOTHING. A directory that names no socket still gets a row
//! (`socket: null`) built from the directory alone. Every row carries `endpoint_kind` and the liveness verdict as
//! `alive` plus `reason` when it is not routable; a `tui` row also carries `owner`. A host row also carries
//! `stale_old_engine`: whether `host gc` would stop it as a stale older-engine leftover (`host_stale_engine`,
//! senpi #3268), judged with observing reads only and only for a routable `rpc_host` endpoint.
use crate::host_endpoints::{
    EndpointKind, HostEndpointEntry, endpoint_probe_timeout_ms, judge_endpoint_liveness,
    list_host_endpoints,
};
use crate::host_status::{HostSessionCounts, generation_status_rows, probe_host_status};
use futures::{StreamExt, stream};
use serde_json::{Value, json};
use std::path::Path;

/// Endpoints read at once, so hundreds of hung hosts never exhaust descriptors.
pub const STATUS_ALL_MAX_IN_FLIGHT: usize = 64;

pub async fn read_all_host_status(
    agent_dir: &Path,
    include_workers: bool,
    timeout_ms: Option<u64>,
) -> std::io::Result<Vec<Value>> {
    read_all_host_status_with(
        agent_dir,
        include_workers,
        timeout_ms,
        &crate::host_gc::StaleEngineRequest::default(),
    )
    .await
}
/// `read_all_host_status` with the stale-engine judge's options (senpi `StatusAllOptions.staleEngine`); the
/// default judges against this build.
pub async fn read_all_host_status_with(
    agent_dir: &Path,
    include_workers: bool,
    timeout_ms: Option<u64>,
    stale_engine: &crate::host_gc::StaleEngineRequest,
) -> std::io::Result<Vec<Value>> {
    let endpoints = list_host_endpoints(agent_dir);
    let mut rows: Vec<_> = stream::iter(endpoints.iter().enumerate())
        .map(|(index, endpoint)| async move {
            (
                index,
                endpoint_status(
                    endpoint,
                    agent_dir,
                    include_workers,
                    timeout_ms,
                    stale_engine,
                )
                .await,
            )
        })
        .buffer_unordered(STATUS_ALL_MAX_IN_FLIGHT)
        .collect()
        .await;
    rows.sort_unstable_by_key(|(index, _)| *index);
    rows.into_iter().map(|(_, row)| row).collect()
}
async fn endpoint_status(
    endpoint: &HostEndpointEntry,
    agent_dir: &Path,
    include_workers: bool,
    timeout_ms: Option<u64>,
    stale_engine: &crate::host_gc::StaleEngineRequest,
) -> std::io::Result<Value> {
    let paths = crate::host_daemon_paths::host_daemon_directory_paths(&endpoint.dir);
    let (mut report, answered, listing) = match &endpoint.socket {
        Some(socket) => {
            let probe = probe_host_status(
                socket,
                agent_dir,
                include_workers,
                false,
                endpoint_probe_timeout_ms(endpoint.endpoint_kind, timeout_ms),
            )
            .await?;
            (probe.report, probe.answered, probe.listing)
        }
        None => {
            let generations = crate::host_generations::read_generation_rows_with(
                &paths,
                crate::host_generations::GenerationRowOptions { include_dead: true },
            )?;
            // Without the socket no generation can be proven to serve it, so nothing is elected or offered.
            let pointer = generations.iter().find(|row| row.current).map_or_else(
                || crate::host_pointer_election::HostPointerState::missing(Vec::new()),
                |row| {
                    crate::host_pointer_election::HostPointerState::current(row.instance_id.clone())
                },
            );
            let claims_live = crate::host_status_rows::read_claim_rows(&paths, &generations)
                .iter()
                .filter(|claim| claim.live)
                .count();
            let crashes = crate::host_crash_record::read_host_crash_records(&paths.dir)
                .iter()
                .filter(|record| crate::host_crash_record::is_host_crash(record))
                .count();
            (
                json!({
                    "reachable": false, "lifecycle": null, "socket": null, "pid": null,
                    "processStartTime": null, "instanceId": null, "generation": null,
                    "engineVersion": null, "capabilities": [], "launchProfile": null,
                    "runtimeBuildId": null, "handover": null, "sessions": HostSessionCounts::default(),
                    "zombies": null, "rss_mb": null, "host_rss_mb": null, "host_footprint_mb": null,
                    "open_fds": null, "memory_pressure": null, "path_identity_resolver": null,
                    "env_keys": [], "generations": generation_status_rows(&generations), "pointer": pointer,
                    "crashes": crashes, "shard": null, "session_rows": [], "claims_live": claims_live, "claims": [],
                }),
                None,
                Vec::new(),
            )
        }
    };
    let verdict = judge_endpoint_liveness(
        &paths,
        answered.as_ref().map(|host| host.instance_id.as_deref()),
    );
    let tui = endpoint.endpoint_kind == EndpointKind::Tui;
    let owner = if tui {
        let row = listing.first();
        json!({
            "pid": report["generations"].as_array().and_then(|rows| rows.iter().find(|row| row["current"] == true)).and_then(|row| row["pid"].as_u64()),
            "cwd": row.and_then(|row| row.cwd.as_deref()),
            "session": row.map(|row| json!({"id":row.id,"path":row.session_path,"name":row.name})),
        })
    } else {
        Value::Null
    };
    report["dir"] = json!(endpoint.dir);
    report["identity"] = endpoint.identity.into();
    report["endpoint_kind"] = endpoint.endpoint_kind.as_str().into();
    report["alive"] = (verdict == crate::host_endpoints::EndpointLiveness::Routable).into();
    report["reason"] = if verdict == crate::host_endpoints::EndpointLiveness::Routable {
        Value::Null
    } else {
        verdict.as_str().into()
    };
    report["owner"] = owner;
    let stale = match &endpoint.socket {
        Some(socket)
            if endpoint.endpoint_kind == EndpointKind::RpcHost
                && verdict == crate::host_endpoints::EndpointLiveness::Routable =>
        {
            matches!(
                stale_engine.judge(socket, agent_dir).await,
                crate::host_stale_engine::StaleEngineVerdict::Stale { .. }
            )
        }
        _ => false,
    };
    report["stale_old_engine"] = stale.into();
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn an_absent_host_row_is_dead_and_a_nameless_directory_still_gets_a_row() {
        let temp = tempfile::tempdir().unwrap();
        assert!(
            read_all_host_status(temp.path(), false, None)
                .await
                .unwrap()
                .is_empty()
        );
        let socket = temp.path().join("gone.sock").to_string_lossy().into_owned();
        let paths = crate::host_daemon_paths::create_host_daemon_paths(&socket, temp.path());
        crate::host_daemon_paths::create_daemon_directories(&paths).unwrap();
        std::fs::write(&paths.endpoint_file, json!({"socket":socket}).to_string()).unwrap();
        std::fs::create_dir_all(temp.path().join("rpc-host-daemon").join("0123456789abcdef"))
            .unwrap();
        let rows = read_all_host_status(temp.path(), false, Some(200))
            .await
            .unwrap();
        assert_eq!(rows.len(), 2);
        let named = rows
            .iter()
            .find(|row| row["socket"] == json!(socket))
            .unwrap();
        assert_eq!(
            (
                named["alive"].clone(),
                named["reason"].clone(),
                named["endpoint_kind"].clone()
            ),
            (json!(false), json!("dead"), json!("rpc_host"))
        );
        let nameless = rows.iter().find(|row| row["socket"].is_null()).unwrap();
        assert_eq!(
            (nameless["identity"].clone(), nameless["reachable"].clone()),
            (json!("unknown"), json!(false))
        );
        for field in [
            "lifecycle",
            "runtimeBuildId",
            "handover",
            "crashes",
            "shard",
            "session_rows",
            "claims",
        ] {
            assert!(nameless.get(field).is_some(), "{field}");
        }
        assert!(paths.dir.exists(), "status --all must remove nothing");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn answering_tui_without_instance_stays_unroutable_but_names_owner_session() {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
        let temp = tempfile::tempdir().unwrap();
        let socket = temp.path().join("t.sock");
        let listener = tokio::net::UnixListener::bind(&socket).unwrap();
        let socket = socket.to_str().unwrap();
        let paths = crate::host_daemon_paths::create_host_daemon_paths(socket, temp.path());
        crate::host_daemon_paths::create_daemon_directories(&paths).unwrap();
        std::fs::write(
            &paths.endpoint_file,
            json!({"socket":socket,"endpoint_kind":"tui"}).to_string(),
        )
        .unwrap();
        std::fs::write(&paths.pointer_file, r#"{"instance_id":"recorded"}"#).unwrap();
        let generation = crate::host_daemon_paths::generation_paths(&paths, "recorded");
        std::fs::create_dir_all(&generation.dir).unwrap();
        std::fs::write(
            &generation.pid_file,
            json!({"pid":std::process::id(),"generation":1}).to_string(),
        )
        .unwrap();

        let serve = tokio::spawn(async move {
            loop {
                let (socket, _) = listener.accept().await.unwrap();
                let mut socket = tokio::io::BufReader::new(socket);
                let mut line = String::new();
                // Each observing connection can ask for protocol info before its actual request.
                while socket.read_line(&mut line).await.unwrap() != 0 {
                    let request: Value = serde_json::from_str(&line).unwrap();
                    assert_eq!(request["observe"], true);
                    let listing = request["type"] == "list_sessions";
                    let data = match request["type"].as_str() {
                        Some("get_protocol_info") => {
                            json!({"protocolVersion":1, "serverVersion":"test", "capabilities":[]})
                        }
                        Some("list_sessions") => {
                            json!({"sessions":[{"sessionId":"mine","sessionPath":"/session.jsonl","name":"Title","cwd":"/project"}]})
                        }
                        command => panic!("unexpected status probe: {command:?}"),
                    };
                    socket
                        .write_all(
                            format!(
                                "{}\n",
                                json!({"id":request["id"],"success":true,"data":data})
                            )
                            .as_bytes(),
                        )
                        .await
                        .unwrap();
                    if listing {
                        return;
                    }
                    line.clear();
                }
            }
        });
        let statuses = read_all_host_status(temp.path(), false, Some(1500))
            .await
            .unwrap();
        let row = &statuses[0];
        assert_eq!(row["reachable"], true);
        assert_eq!(
            row["instanceId"], "recorded",
            "the report may fall back to disk"
        );
        assert_eq!(
            row["alive"], false,
            "routing requires the socket's instance"
        );
        assert_eq!(row["reason"], "live_unresponsive");
        assert_eq!(row["session_rows"], json!([]));
        assert_eq!(
            row["owner"],
            json!({"pid":std::process::id(),"cwd":"/project","session":{"id":"mine","path":"/session.jsonl","name":"Title"}})
        );
        serve.await.unwrap();
    }
}
