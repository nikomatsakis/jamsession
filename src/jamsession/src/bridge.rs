use std::sync::{Arc, Mutex};

use agent_client_protocol::{Agent, Client, ConnectionTo, Dispatch, HandleDispatchFrom, Handled};
use tokio::sync::Notify;

/// Shared message buffer for recording agent messages (used for session/load replay).
pub type MessageBuffer = Arc<Mutex<Vec<serde_json::Value>>>;

/// Shared signal that fires on every message through the bridge (for quiescence detection).
pub type ActivitySignal = Arc<Notify>;

/// Forwards all dispatches from the client to the agent.
/// Installed on the client's connection after session activation.
pub struct BridgeHandler {
    agent_cx: ConnectionTo<Agent>,
    activity: ActivitySignal,
}

impl BridgeHandler {
    pub fn new(agent_cx: ConnectionTo<Agent>, activity: ActivitySignal) -> Self {
        Self { agent_cx, activity }
    }
}

impl HandleDispatchFrom<Client> for BridgeHandler {
    async fn handle_dispatch_from(
        &mut self,
        message: Dispatch,
        _client_cx: ConnectionTo<Client>,
    ) -> agent_client_protocol::schema::Result<Handled<Dispatch>> {
        self.activity.notify_waiters();
        self.agent_cx.send_proxied_message(message)?;
        Ok(Handled::Yes)
    }

    fn describe_chain(&self) -> impl std::fmt::Debug {
        "BridgeHandler"
    }
}

/// Forwards all dispatches from the agent back to the client.
/// Also records notifications into the session's message buffer for replay.
pub struct ReverseBridgeHandler {
    client_cx: ConnectionTo<Client>,
    buffer: MessageBuffer,
    activity: ActivitySignal,
}

impl ReverseBridgeHandler {
    pub fn new(
        client_cx: ConnectionTo<Client>,
        buffer: MessageBuffer,
        activity: ActivitySignal,
    ) -> Self {
        Self {
            client_cx,
            buffer,
            activity,
        }
    }
}

impl HandleDispatchFrom<Agent> for ReverseBridgeHandler {
    async fn handle_dispatch_from(
        &mut self,
        message: Dispatch,
        _agent_cx: ConnectionTo<Agent>,
    ) -> agent_client_protocol::schema::Result<Handled<Dispatch>> {
        self.activity.notify_waiters();
        // Record notifications into buffer for future session/load replay
        if let Dispatch::Notification(ref notif) = message
            && let Ok(value) = serde_json::to_value(notif)
        {
            self.buffer.lock().unwrap().push(value);
        }
        self.client_cx.send_proxied_message(message)?;
        Ok(Handled::Yes)
    }

    fn describe_chain(&self) -> impl std::fmt::Debug {
        "ReverseBridgeHandler"
    }
}
