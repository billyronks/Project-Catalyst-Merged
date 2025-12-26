//! MCP Server implementation

use axum::response::sse::{Event, KeepAlive, Sse};
use futures_util::stream::Stream;
use std::convert::Infallible;
use std::time::Duration;
use std::pin::Pin;
use std::task::{Context, Poll};

use brivas_core::Result;
use brivas_mcp_sdk::protocol::{ServerCapabilities, ToolsCapability, ResourcesCapability, PromptsCapability};

use crate::config::McpConfig;
use crate::tools::BrivasTools;
use crate::resources::BrivasResources;
use crate::prompts::BrivasPrompts;

/// MCP Server
pub struct McpServer {
    pub capabilities: ServerCapabilities,
    pub tools: BrivasTools,
    pub resources: BrivasResources,
    pub prompts: BrivasPrompts,
}

impl McpServer {
    pub async fn new(_config: &McpConfig) -> Result<Self> {
        Ok(Self {
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability { list_changed: false }),
                resources: Some(ResourcesCapability { subscribe: true, list_changed: false }),
                prompts: Some(PromptsCapability { list_changed: false }),
            },
            tools: BrivasTools::new(),
            resources: BrivasResources::new(),
            prompts: BrivasPrompts::new(),
        })
    }
}

/// Simple ping stream for SSE
struct PingStream {
    interval: tokio::time::Interval,
}

impl PingStream {
    fn new() -> Self {
        Self {
            interval: tokio::time::interval(Duration::from_secs(30)),
        }
    }
}

impl Stream for PingStream {
    type Item = std::result::Result<Event, Infallible>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.interval).poll_tick(cx) {
            Poll::Ready(_) => {
                let event = Event::default()
                    .data(r#"{"jsonrpc":"2.0","method":"ping"}"#)
                    .event("message");
                Poll::Ready(Some(Ok(event)))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// SSE handler for MCP over HTTP
pub async fn sse_handler() -> Sse<impl Stream<Item = std::result::Result<Event, Infallible>>> {
    Sse::new(PingStream::new()).keep_alive(KeepAlive::default())
}
