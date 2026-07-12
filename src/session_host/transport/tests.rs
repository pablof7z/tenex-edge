use super::*;

#[test]
fn transport_kind_strings() {
    assert_eq!(TransportKind::Pty.as_str(), "pty");
    assert_eq!(TransportKind::Acp.as_str(), "acp");
}

#[test]
fn select_transport_is_pty_in_phase1() {
    let t = select_transport("claude");
    assert_eq!(t.kind(), TransportKind::Pty);
}

#[test]
fn pty_transport_reports_pty_kind() {
    assert_eq!(PtyTransport.kind(), TransportKind::Pty);
}

#[test]
fn acp_transport_reports_acp_kind() {
    assert_eq!(AcpTransport.kind(), TransportKind::Acp);
}

#[tokio::test]
async fn pty_resume_without_token_errors() {
    let spec = LaunchSpec {
        slug: "claude".into(),
        root: "chan".into(),
        abs_path: "/tmp".into(),
        group: None,
        ephemeral: false,
        base_command: vec!["claude".into()],
    };
    let resume = ResumeSpec {
        native_id: String::new(),
    };
    let err = PtyTransport.resume(&spec, &resume).await.unwrap_err();
    assert!(err.to_string().contains("not resumable"));
}

#[tokio::test]
async fn acp_is_live_false_for_unknown_endpoint() {
    let ep = EndpointRef {
        kind: TransportKind::Acp,
        endpoint_id: "te-acp-nope".into(),
    };
    assert!(!AcpTransport.is_live(&ep));
    // kill of an unregistered endpoint is a no-op, not an error.
    assert!(AcpTransport.kill(&ep).await.is_ok());
}
