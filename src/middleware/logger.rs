use cfg_if::cfg_if;
use kv_log_macro::{error, info, trace, warn};
use tide::http::headers::{REFERER, USER_AGENT};
use tide::{Middleware, Next, Request, Result};

#[cfg(features = "honeycomb")]
use tracing_honeycomb::TraceId;

use super::extension_types::{CorrelationId, RequestId};

/// Log all outgoing responses.
#[derive(Debug, Default, Clone)]
pub struct LogMiddleware {
    _priv: (),
}

struct LogMiddlewareHasBeenRun;

impl LogMiddleware {
    /// Create a new instance of `LogMiddleware`.
    #[must_use]
    pub fn new() -> Self {
        Self { _priv: () }
    }

    /// Log a request and a response.
    async fn log<'a, State: Clone + Send + Sync + 'static>(
        &'a self,
        mut req: Request<State>,
        next: Next<'a, State>,
    ) -> Result {
        if req.ext::<LogMiddlewareHasBeenRun>().is_some() {
            return Ok(next.run(req).await);
        }
        req.set_ext(LogMiddlewareHasBeenRun);

        let request_id = req
            .ext::<RequestId>()
            .expect("RequestIdMiddleware must be installed before LogMiddleware.")
            .clone();

        cfg_if! {
            if #[cfg(features = "honeycomb")] {
                let honeycomb_trace_id = req.ext::<TraceId>().cloned();
            } else {
                let honeycomb_trace_id = Some("disabled");
            }
        }

        let path = req.url().path().to_owned();
        let method = req.method();
        // TODO(Jeremiah): Do we need to check the Forwarded header for the origin IP?
        let ip = req.peer_addr().unwrap_or("(no Peer Address)").to_string();
        let referer = req
            .header(REFERER)
            .map(|hvs| hvs.last().as_str())
            .unwrap_or("(no Referer)")
            .to_string();
        let user_agent = req
            .header(USER_AGENT)
            .map(|hvs| hvs.last().as_str())
            .unwrap_or("(no User-Agent)")
            .to_string();

        trace!("Incoming Request", {
            method: method.as_ref(),
            path: path,
            ip: ip,
            referer: referer,
            user_agent: user_agent,
            body_size: req.len(),
            request_id: request_id,
        });

        let start = std::time::Instant::now();
        let res = next.run(req).await;
        let status = res.status();

        #[cfg(feature = "panic-on-error")]
        #[allow(clippy::unwrap_used)]
        if let Some(error) = res.error() {
            Err::<(), &tide::Error>(error).unwrap();
        }

        if let Some(correlation_id) = res.ext::<CorrelationId>() {
            if let Some(error) = res.error() {
                error!("Internal Error", {
                    status: status as u16,
                    method: method.as_ref(),
                    path: path,
                    ip: ip,
                    referer: referer,
                    user_agent: user_agent,
                    message: format!("{:?}", error),
                    error_type: error.type_name(),
                    correlation_id: correlation_id,
                    request_id: request_id,
                    honeycomb_trace_id: honeycomb_trace_id.map(|v| v.to_string()),
                    elapsed: format!("{:?}", start.elapsed()),
                });
            } else {
                error!("Internal Error", {
                    status: status as u16,
                    method: method.as_ref(),
                    path: path,
                    ip: ip,
                    referer: referer,
                    user_agent: user_agent,
                    correlation_id: correlation_id,
                    request_id: request_id,
                    honeycomb_trace_id: honeycomb_trace_id.map(|v| v.to_string()),
                    elapsed: format!("{:?}", start.elapsed()),
                });
            }
        } else if status.is_server_error() {
            // Programmer error, always expect there to be JsonErrorMiddleware,
            // which will catch internal server errors first and assign them a correlation id.
            error!("Internal Error -- JsonErrorMiddleware must be installed after LogMiddleware");
        } else if status.is_client_error() {
            if let Some(error) = res.error() {
                warn!("Client Error: {}", status.canonical_reason(), {
                    status: status as u16,
                    method: method.as_ref(),
                    path: path,
                    ip: ip,
                    referer: referer,
                    user_agent: user_agent,
                    message: format!("{:?}", error),
                    error_type: error.type_name(),
                    request_id: request_id,
                    honeycomb_trace_id: honeycomb_trace_id.map(|v| v.to_string()),
                    elapsed: format!("{:?}", start.elapsed()),
                });
            } else {
                warn!("Client Error: {}", status.canonical_reason(), {
                    status: status as u16,
                    method: method.as_ref(),
                    path: path,
                    ip: ip,
                    referer: referer,
                    user_agent: user_agent,
                    request_id: request_id,
                    honeycomb_trace_id: honeycomb_trace_id.map(|v| v.to_string()),
                    elapsed: format!("{:?}", start.elapsed()),
                });
            }
        } else {
            info!("{}", status.canonical_reason(), {
                status: status as u16,
                method: method.as_ref(),
                path: path,
                ip: ip,
                referer: referer,
                user_agent: user_agent,
                body_size: res.len(),
                request_id: request_id,
                honeycomb_trace_id: honeycomb_trace_id.map(|v| v.to_string()),
                elapsed: format!("{:?}", start.elapsed()),
            });
        }
        Ok(res)
    }
}

#[tide::utils::async_trait]
impl<State: Clone + Send + Sync + 'static> Middleware<State> for LogMiddleware {
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> Result {
        self.log(req, next).await
    }
}
