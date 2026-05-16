use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::time::timeout;

use crate::client::Client;
use crate::error::Error;
use crate::oauth::{build_oauth_init_url, generate_oauth_state, generate_pkce_challenge, BuildOauthInitUrl};
use crate::types::{OauthExchangePayload, OauthExchangeResponse};

const SUCCESS_HTML: &str = "<!doctype html><meta charset=utf-8><title>Sign in complete</title><style>body{font-family:-apple-system,system-ui,sans-serif;background:#08080b;color:#e6e6ec;display:grid;place-items:center;height:100vh;margin:0}</style><h1>You can close this tab.</h1>";

#[derive(Debug, thiserror::Error)]
pub enum LoopbackError {
    #[error("bind callback listener: {0}")]
    Bind(String),
    #[error("read callback: {0}")]
    Read(String),
    #[error("parse callback url: {0}")]
    Parse(String),
    #[error("oauth state mismatch")]
    StateMismatch,
    #[error("oauth callback missing code")]
    MissingCode,
    #[error("oauth callback missing state")]
    MissingState,
    #[error("open browser: {0}")]
    OpenBrowser(String),
    #[error("oauth callback timed out")]
    Timeout,
    #[error("api: {0}")]
    Api(#[from] Error),
}

pub struct LoopbackOutcome {
    pub exchange: OauthExchangeResponse,
}

/// Runs the desktop loopback PKCE OAuth flow end-to-end:
///
/// 1. Binds a transient `127.0.0.1` listener.
/// 2. Generates PKCE + state, builds the provider URL.
/// 3. Calls `open_browser(url)` so the consumer controls how the system browser is launched.
/// 4. Awaits the callback (300 s timeout, configurable).
/// 5. Exchanges the code for an access token via the SDK.
/// 6. Stores the token on the supplied client (`set_customer_token`).
pub async fn loopback_login(
    sdk: &mut Client,
    product: &str,
    provider: &str,
    open_browser: impl Fn(&str) -> Result<(), String>,
    timeout_secs: u64,
) -> Result<LoopbackOutcome, LoopbackError> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| LoopbackError::Bind(e.to_string()))?;
    let port = listener
        .local_addr()
        .map_err(|e| LoopbackError::Bind(e.to_string()))?
        .port();
    let redirect_uri = format!("http://127.0.0.1:{port}/cb");

    let pkce = generate_pkce_challenge();
    let state = generate_oauth_state();

    let auth_url = build_oauth_init_url(BuildOauthInitUrl {
        base_url: sdk.base_url(),
        provider,
        product,
        redirect_uri: &redirect_uri,
        code_challenge: &pkce.challenge,
        code_challenge_method: Some(pkce.method),
        state: Some(&state),
    });

    open_browser(&auth_url).map_err(LoopbackError::OpenBrowser)?;

    let (code, returned_state) = match timeout(
        Duration::from_secs(timeout_secs.max(1)),
        accept_callback(listener),
    )
    .await
    {
        Ok(Ok(pair)) => pair,
        Ok(Err(e)) => return Err(e),
        Err(_) => return Err(LoopbackError::Timeout),
    };

    if returned_state != state {
        return Err(LoopbackError::StateMismatch);
    }

    let exchange = sdk
        .exchange_oauth_code(OauthExchangePayload {
            code: &code,
            code_verifier: &pkce.verifier,
        })
        .await?;

    sdk.set_customer_token(exchange.access_token.clone());

    Ok(LoopbackOutcome { exchange })
}

async fn accept_callback(listener: TcpListener) -> Result<(String, String), LoopbackError> {
    let (mut stream, _) = listener
        .accept()
        .await
        .map_err(|e| LoopbackError::Read(e.to_string()))?;
    let (read_half, mut write_half) = stream.split();
    let mut reader = BufReader::new(read_half);
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .await
        .map_err(|e| LoopbackError::Read(e.to_string()))?;

    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 2 {
        return Err(LoopbackError::Read("malformed request line".to_string()));
    }
    let url = url::Url::parse(&format!("http://127.0.0.1{}", parts[1]))
        .map_err(|e| LoopbackError::Parse(e.to_string()))?;

    let mut code = None;
    let mut state = None;
    for (k, v) in url.query_pairs() {
        match k.as_ref() {
            "code" => code = Some(v.into_owned()),
            "state" => state = Some(v.into_owned()),
            _ => {}
        }
    }

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{}",
        SUCCESS_HTML.len(),
        SUCCESS_HTML
    );
    let _ = write_half.write_all(response.as_bytes()).await;
    let _ = write_half.flush().await;

    Ok((
        code.ok_or(LoopbackError::MissingCode)?,
        state.ok_or(LoopbackError::MissingState)?,
    ))
}
