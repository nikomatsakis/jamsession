mod harness;

use harness::{TestDaemon, TestDaemonConfig};

#[tokio::test]
async fn smoke_list_sessions_empty() {
    let daemon = TestDaemon::start(TestDaemonConfig::default()).await;

    let result = daemon
        .execute_client(
            r#"
        let sessions = list_sessions();
        sessions.len()
    "#,
        )
        .await;

    assert_eq!(result, "0");
}

#[tokio::test]
async fn smoke_basic_session_prompt_response() {
    let daemon = TestDaemon::start(TestDaemonConfig {
        agent_script: r#"
            let prompt = receive_prompt();
            say("echo: " + prompt);
        "#
        .into(),
        ..Default::default()
    })
    .await;

    let result = daemon
        .execute_client(
            r#"
        let s = start_session();
        s.prompt("hello")
    "#,
        )
        .await;

    assert_eq!(result, "echo: hello");
}
