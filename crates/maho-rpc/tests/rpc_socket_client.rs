#![cfg(any(unix, windows))]
#[cfg(unix)]
use maho_rpc::rpc_client::ClientFrame;
use maho_rpc::rpc_client::RpcSocketClient;
#[cfg(unix)]
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
#[tokio::test]
async fn authenticated_client_sends_secret_before_correlated_protocol_request() {
    use tokio::io::AsyncBufReadExt;
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("authenticated.sock");
    let secret_path = temp.path().join("secret");
    let secret = maho_rpc::socket_transport::create_socket_secret(&secret_path).unwrap();
    #[cfg(unix)]
    let listener = tokio::net::UnixListener::bind(&path).unwrap();
    #[cfg(windows)]
    let listener = {
        let pipe = maho_rpc::socket_transport::resolve_socket_transport_address(
            path.to_str().unwrap(),
            "win32",
            Some(&secret),
        )
        .unwrap();
        maho_rpc::host_lifecycle::win32_pipe::create_first_instance(&pipe).unwrap()
    };
    let (finish, finished) = tokio::sync::oneshot::channel();
    let host = async {
        #[cfg(unix)]
        let (mut stream, _) = listener.accept().await.unwrap();
        #[cfg(windows)]
        let mut stream = {
            tokio::time::timeout(std::time::Duration::from_secs(3), listener.connect())
                .await
                .expect("RPC client connected to the Windows named pipe")
                .unwrap();
            listener
        };
        maho_rpc::socket_transport::authenticate_socket(&mut stream, &secret)
            .await
            .unwrap();
        let (read, mut write) = tokio::io::split(stream);
        let mut lines = tokio::io::BufReader::new(read).lines();
        let command: serde_json::Value =
            serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
        assert_eq!(command["type"], "get_protocol_info");
        write
            .write_all(
                maho_rpc::jsonl::serialize_json_line(
                    &serde_json::json!({"id":command["id"],"type":"response","success":true}),
                )
                .unwrap()
                .as_bytes(),
            )
            .await
            .unwrap();
        finished.await.unwrap();
    };
    let client = async {
        let mut client = RpcSocketClient::connect_authenticated(&path, &secret_path)
            .await
            .unwrap();
        assert_eq!(
            client
                .request(
                    serde_json::json!({"type":"get_protocol_info"}),
                    false,
                    |_| {},
                    |_| {}
                )
                .await
                .unwrap()["success"],
            true
        );
        finish.send(()).unwrap();
    };
    #[cfg(windows)]
    let deadline = std::time::Duration::from_secs(5);
    #[cfg(unix)]
    let deadline = std::time::Duration::from_secs(2);
    tokio::time::timeout(deadline, async { tokio::join!(client, host) })
        .await
        .expect("authenticated RPC exchange finished");
}
#[cfg(unix)]
#[tokio::test]
async fn refused_open_discards_prelease_events_and_preserves_current_lease() {
    let (client, mut host) = tokio::net::UnixStream::pair().unwrap();
    let mut client = RpcSocketClient::from_stream(client);
    client.frames.session_id = Some("existing".into());
    let serve = async {
        let mut request = vec![0; "{\"type\":\"open_session\",\"id\":\"req_1\"}\n".len()];
        host.read_exact(&mut request).await.unwrap();
        host.write_all(b"{\"type\":\"agent_start\",\"sessionId\":\"refused\"}\n{\"type\":\"response\",\"id\":\"req_1\",\"success\":false,\"error\":\"denied\"}\n").await.unwrap();
    };
    let open = client.open_session(serde_json::json!({}), |_| {
        panic!("refused lease must not replay")
    });
    let (result, ()) = tokio::time::timeout(std::time::Duration::from_secs(2), async {
        tokio::join!(open, serve)
    })
    .await
    .unwrap();
    assert_eq!(result.unwrap_err().to_string(), "denied");
    assert!(!client.frames.pending_open_session);
    assert_eq!(client.frames.session_id.as_deref(), Some("existing"));
    assert!(client.frames.flush_pending_session_events().is_empty());
}
#[cfg(unix)]
#[tokio::test]
async fn failed_socket_write_retires_request_debt() {
    let (client, host) = tokio::net::UnixStream::pair().unwrap();
    drop(host);
    let mut client = RpcSocketClient::from_stream(client);
    assert!(
        client
            .send(serde_json::json!({"type":"get_state"}), false, true)
            .await
            .is_err()
    );
    assert!(client.frames.reject_pending().is_empty());
}
#[cfg(unix)]
#[tokio::test]
async fn opening_replays_only_selected_lease_then_routes_next_command() {
    let (client, mut host) = tokio::net::UnixStream::pair().unwrap();
    let mut client = RpcSocketClient::from_stream(client);
    let serve = async {
        let mut request = vec![0; "{\"type\":\"open_session\",\"id\":\"req_1\"}\n".len()];
        host.read_exact(&mut request).await.unwrap();
        host.write_all(b"{\"type\":\"agent_start\",\"sessionId\":\"other\"}\n{\"type\":\"agent_start\",\"sessionId\":\"selected\"}\n{\"type\":\"response\",\"id\":\"req_1\",\"success\":true,\"data\":{\"sessionId\":\"selected\",\"state\":{}}}\n").await.unwrap();
    };
    let mut events = vec![];
    let open = client.open_session(serde_json::json!({}), |event| events.push(event));
    let (result, ()) = tokio::time::timeout(std::time::Duration::from_secs(2), async {
        tokio::join!(open, serve)
    })
    .await
    .unwrap();
    assert_eq!(result.unwrap()["sessionId"], "selected");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["sessionId"], "selected");
    assert_eq!(
        client
            .send(serde_json::json!({"type":"get_state"}), true, true)
            .await
            .unwrap()["sessionId"],
        "selected"
    );
}
#[cfg(unix)]
#[tokio::test]
async fn response_hook_runs_before_next_frame_from_same_socket_read() {
    let (client, mut host) = tokio::net::UnixStream::pair().unwrap();
    let mut client = RpcSocketClient::from_stream(client);
    let seen = std::cell::Cell::new(false);
    let serve = async {
        let mut request = vec![0; "{\"type\":\"get_state\",\"id\":\"req_1\"}\n".len()];
        host.read_exact(&mut request).await.unwrap();
        let request: serde_json::Value = serde_json::from_slice(&request).unwrap();
        assert_eq!(request["id"], "req_1");
        host.write_all(b"{\"type\":\"response\",\"id\":\"req_1\",\"success\":true}\n{\"type\":\"agent_start\"}\n").await.unwrap();
    };
    let request = client.request(
        serde_json::json!({"type":"get_state"}),
        false,
        |_| panic!("following event overtook response"),
        |_| seen.set(true),
    );
    let (result, ()) = tokio::time::timeout(std::time::Duration::from_secs(2), async {
        tokio::join!(request, serve)
    })
    .await
    .unwrap();
    result.unwrap();
    assert!(seen.get());
    assert!(matches!(
        client.receive().await.unwrap(),
        Some(ClientFrame::Event(_))
    ));
}
#[cfg(unix)]
#[tokio::test]
async fn real_socket_correlates_response_before_following_event() {
    let (client, mut host) = tokio::net::UnixStream::pair().unwrap();
    let mut client = RpcSocketClient::from_stream(client);
    client.frames.session_id = Some("lease".into());
    let command = client
        .send(serde_json::json!({"type":"get_messages"}), true, true)
        .await
        .unwrap();
    let mut bytes = vec![0; command.to_string().len() + 1];
    host.read_exact(&mut bytes).await.unwrap();
    let request: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(request["sessionId"], "lease");
    host.write_all(b"{\"type\":\"response\",\"id\":\"req_1\",\"success\":true}\n{\"type\":\"agent_start\",\"sessionId\":\"lease\"}\n").await.unwrap();
    assert!(matches!(
        client.receive().await.unwrap(),
        Some(ClientFrame::Response(_))
    ));
    assert!(matches!(
        client.receive().await.unwrap(),
        Some(ClientFrame::Event(_))
    ));
    host.shutdown().await.unwrap();
    assert!(client.receive().await.unwrap().is_none());
    assert!(client.frames.reject_pending().is_empty());
}
