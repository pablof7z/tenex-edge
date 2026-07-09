use super::protocol::{error, notification, result, Message, INVALID_REQUEST, METHOD_NOT_FOUND};
use anyhow::{Context, Result};
use axum::extract::State;
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tokio::task::JoinHandle;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

#[derive(Clone)]
pub(super) struct HttpState {
    subscriptions: HttpSubscriptions,
}

#[derive(Clone)]
pub(super) struct HttpSubscriptions {
    tasks: Arc<Mutex<HashMap<String, JoinHandle<()>>>>,
    tx: broadcast::Sender<Value>,
}

impl Default for HttpSubscriptions {
    fn default() -> Self {
        let (tx, _) = broadcast::channel(128);
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            tx,
        }
    }
}

pub(super) async fn serve(args: super::McpArgs) -> Result<()> {
    let addr = SocketAddr::new(args.host.parse::<IpAddr>()?, args.port);
    let path = normalize_path(&args.path)?;
    let state = HttpState {
        subscriptions: HttpSubscriptions::default(),
    };
    let app = Router::new()
        .route("/", get(root_health))
        .route(&path, post(post_mcp).get(get_mcp).options(options_mcp))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let local = listener.local_addr()?;
    eprintln!(
        "[tenex-edge] MCP HTTP listening on http://{}:{}{}",
        local.ip(),
        local.port(),
        path
    );
    eprintln!(
        "[tenex-edge] ChatGPT requires HTTPS; tunnel this endpoint and use https://<host>{path}"
    );
    axum::serve(listener, app)
        .await
        .context("serving MCP HTTP endpoint")
}

async fn post_mcp(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(value): Json<Value>,
) -> Response {
    let message = match serde_json::from_value::<Message>(value) {
        Ok(message) => message,
        Err(err) => {
            log_http_event("post_parse_error", &headers, None, &Value::Null);
            return json_response(
                StatusCode::BAD_REQUEST,
                error(
                    Value::Null,
                    super::protocol::PARSE_ERROR,
                    format!("{err:#}"),
                ),
            );
        }
    };
    log_http_event("post", &headers, message.method.as_deref(), &message.params);
    let Some(method) = message.method.as_deref() else {
        return StatusCode::ACCEPTED.into_response();
    };
    if message.is_notification() {
        return StatusCode::ACCEPTED.into_response();
    }
    let Some(id) = message.id.clone() else {
        return json_response(
            StatusCode::BAD_REQUEST,
            error(Value::Null, INVALID_REQUEST, "request id is required"),
        );
    };
    let response = dispatch_http(&state, method, &message.params, id).await;
    json_response(StatusCode::OK, response)
}

async fn root_health(headers: HeaderMap) -> impl IntoResponse {
    log_http_event("root", &headers, None, &Value::Null);
    Json(json!({ "ok": true, "name": "tenex-edge MCP", "mcp": "/mcp" }))
}

async fn get_mcp(State(state): State<HttpState>, headers: HeaderMap) -> impl IntoResponse {
    log_http_event("sse_get", &headers, None, &Value::Null);
    let rx = state.subscriptions.tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|value| match value {
        Ok(value) => Some(Ok::<_, Infallible>(
            Event::default().data(value.to_string()),
        )),
        Err(_) => None,
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn options_mcp(headers: HeaderMap) -> Response {
    log_http_event("options", &headers, None, &Value::Null);
    (StatusCode::NO_CONTENT, cors_headers()).into_response()
}

async fn dispatch_http(state: &HttpState, method: &str, params: &Value, id: Value) -> Value {
    match method {
        "initialize" => result(id, super::initialize(params)),
        "ping" => result(id, json!({})),
        "tools/list" => result(id, super::tools::list()),
        "tools/call" => match super::tools::call(params).await {
            Ok(value) => result(id, value),
            Err(err) => error(id, super::protocol::INVALID_PARAMS, format!("{err:#}")),
        },
        "resources/list" => result(id, super::resources::list()),
        "resources/templates/list" => result(id, super::resources::templates()),
        "resources/read" => match super::resources::read(params).await {
            Ok(value) => result(id, value),
            Err(err) => error(id, super::protocol::INVALID_PARAMS, format!("{err:#}")),
        },
        "resources/subscribe" => match state.subscriptions.subscribe(params).await {
            Ok(()) => result(id, json!({})),
            Err(err) => error(id, super::protocol::INVALID_PARAMS, format!("{err:#}")),
        },
        "resources/unsubscribe" => match state.subscriptions.unsubscribe(params).await {
            Ok(()) => result(id, json!({})),
            Err(err) => error(id, super::protocol::INVALID_PARAMS, format!("{err:#}")),
        },
        other => error(id, METHOD_NOT_FOUND, format!("unknown method: {other}")),
    }
}

impl HttpSubscriptions {
    async fn subscribe(&self, params: &Value) -> Result<()> {
        let uri = super::protocol::required_string(params, "uri")?;
        let project = super::resources::subscription_project(&uri)?;
        let mut tasks = self.tasks.lock().await;
        if tasks.contains_key(&uri) {
            return Ok(());
        }
        tasks.insert(
            uri.clone(),
            tokio::spawn(run_subscription(uri, project, self.tx.clone())),
        );
        Ok(())
    }

    async fn unsubscribe(&self, params: &Value) -> Result<()> {
        let uri = super::protocol::required_string(params, "uri")?;
        if let Some(task) = self.tasks.lock().await.remove(&uri) {
            task.abort();
        }
        Ok(())
    }
}

async fn run_subscription(uri: String, project: Option<String>, tx: broadcast::Sender<Value>) {
    let params = json!({ "project": project, "backfill": 0 });
    let note_uri = uri.clone();
    let stream_result = async {
        let mut client = crate::daemon::client::Client::connect_or_spawn().await?;
        client
            .stream("tail", params, move |item| {
                if super::resources::event_updates_status_resource(&item) {
                    let note = notification(
                        "notifications/resources/updated",
                        json!({ "uri": note_uri.clone() }),
                    );
                    let _ = tx.send(note);
                }
            })
            .await
    }
    .await;
    if let Err(err) = stream_result {
        eprintln!("[tenex-edge mcp] HTTP subscription ended: {err:#}");
    }
}

fn normalize_path(path: &str) -> Result<String> {
    let path = path.trim();
    if path.is_empty() {
        anyhow::bail!("MCP HTTP path must not be empty");
    }
    Ok(if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    })
}

fn json_response(status: StatusCode, body: Value) -> Response {
    (status, cors_headers(), Json(body)).into_response()
}

fn log_http_event(kind: &str, headers: &HeaderMap, method: Option<&str>, params: &Value) {
    let mut event = serde_json::Map::new();
    event.insert("ts".into(), json!(crate::util::now_secs()));
    event.insert("kind".into(), json!(kind));
    if let Some(method) = method {
        event.insert("mcp_method".into(), json!(method));
    }
    add_param_shape(&mut event, params);
    event.insert("headers".into(), json!(selected_headers(headers)));
    eprintln!("[tenex-edge mcp access] {}", Value::Object(event));
}

fn add_param_shape(event: &mut serde_json::Map<String, Value>, params: &Value) {
    if let Some(tool) = params.get("name").and_then(Value::as_str) {
        event.insert("tool".into(), json!(tool));
    }
    if let Some(uri) = params.get("uri").and_then(Value::as_str) {
        event.insert("resource_uri".into(), json!(uri));
    }
    if let Some(protocol) = params.get("protocolVersion").and_then(Value::as_str) {
        event.insert("protocol_version".into(), json!(protocol));
    }
    if let Some(client) = params.get("clientInfo") {
        event.insert("client_info".into(), client.clone());
    }
    if let Some(arguments) = params.get("arguments").and_then(Value::as_object) {
        let mut keys = arguments.keys().cloned().collect::<Vec<_>>();
        keys.sort();
        event.insert("argument_keys".into(), json!(keys));
        if let Some(session) = arguments.get("session").and_then(Value::as_str) {
            event.insert("argument_session".into(), json!(session));
        }
        if let Some(channel) = arguments.get("channel").and_then(Value::as_str) {
            event.insert("argument_channel".into(), json!(channel));
        }
        if let Some(project) = arguments.get("project").and_then(Value::as_str) {
            event.insert("argument_project".into(), json!(project));
        }
    }
}

fn selected_headers(headers: &HeaderMap) -> serde_json::Map<String, Value> {
    let mut out = serde_json::Map::new();
    for (name, value) in headers {
        let key = name.as_str().to_ascii_lowercase();
        if !should_log_header(&key) {
            continue;
        }
        let value = value.to_str().unwrap_or("<non-utf8>");
        out.insert(key, json!(value));
    }
    out
}

fn should_log_header(key: &str) -> bool {
    !matches!(
        key,
        "authorization" | "cookie" | "set-cookie" | "proxy-authorization"
    )
}

fn cors_headers() -> [(header::HeaderName, HeaderValue); 3] {
    [
        (
            header::ACCESS_CONTROL_ALLOW_ORIGIN,
            HeaderValue::from_static("*"),
        ),
        (
            header::ACCESS_CONTROL_ALLOW_HEADERS,
            HeaderValue::from_static("content-type, accept, authorization, mcp-protocol-version"),
        ),
        (
            header::ACCESS_CONTROL_ALLOW_METHODS,
            HeaderValue::from_static("GET, POST, OPTIONS"),
        ),
    ]
}
