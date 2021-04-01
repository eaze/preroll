use eaze_tracing_honeycomb::{register_dist_tracing_root, SpanId, TraceId};
use tide::{Middleware, Next, Request};
use tracing::instrument;

use super::extension_types::RequestId;
use super::honeycomb::propagation::{Propagation, PROPAGATION_HTTP_HEADER};

/// Set up tracing for every request.
#[derive(Debug, Default, Clone)]
pub struct TraceMiddleware {
    _priv: (),
}

impl TraceMiddleware {
    /// Create a new instance of `TraceMiddleware`.
    #[must_use]
    pub fn new() -> Self {
        Self { _priv: () }
    }

    /// Set up tracing for every request.
    #[instrument(skip(req, next))]
    async fn handle<'a, State: Clone + Send + Sync + 'static>(
        &'a self,
        mut req: Request<State>,
        next: Next<'a, State>,
    ) -> tide::Result {
        if req.ext::<TraceId>().is_some() {
            return Ok(next.run(req).await);
        }

        let trace_id: TraceId;
        let mut parent_span: Option<SpanId> = None;
        let mut propagation: Option<Propagation> = None;
        if let Some(header) = req.header(PROPAGATION_HTTP_HEADER) {
            match Propagation::unmarshal_trace_context(header.as_str()) {
                Ok(prop) => {
                    trace_id = prop.trace_id.clone().into();

                    if !prop.parent_id.is_empty() {
                        match prop.parent_id.parse::<SpanId>() {
                            Ok(span_id) => parent_span = Some(span_id),
                            Err(e) => {
                                log::warn!(
                                    "Error parsing parent span id from X-Honeycomb-Trace: {:?}",
                                    e
                                )
                            }
                        }
                    }
                    propagation = Some(prop);
                }
                Err(e) => {
                    log::warn!(
                        "{} could not be un-marshaled: {}",
                        PROPAGATION_HTTP_HEADER,
                        e
                    );
                    if let Some(req_id) = req.ext::<RequestId>() {
                        trace_id = req_id.as_str().into();
                    } else {
                        trace_id = TraceId::new();
                    }
                }
            };
        } else if let Some(req_id) = req.ext::<RequestId>() {
            trace_id = req_id.as_str().into();
        } else {
            trace_id = TraceId::new();
        }

        req.set_ext(trace_id.clone());

        if let Err(error) = register_dist_tracing_root(trace_id, parent_span) {
            log::error!("Failed to set honeycomb trace root: {:?}", error);
        }

        match eaze_tracing_honeycomb::current_dist_trace_ctx() {
            Ok((trace_id, span_id)) => {
                log::debug!("current_dist_trace_ctx: ({}, {})", trace_id, span_id)
            }
            Err(error) => log::error!("Failed to get current_dist_trace_ctx: {:?}", error),
        }

        tracing::info!(
            method = req.method().as_ref(),
            host = req.host().unwrap_or(""),
            path = req.url().path(),
            query = req.url().query().unwrap_or(""),
            frag = req.url().fragment().unwrap_or(""),
            // Consider enabling when http_types::Version has an `as_ref<&'static str>()`.
            // http_version = req.version().map(|v| v.as_ref()).unwrap_or(""),
            "HTTP Request Info"
        );

        let mut res = next.run(req).await;

        tracing::info!(
            status = res.status() as u16,
            body_size = res
                .len()
                .map(|v| v.to_string())
                .as_deref()
                .unwrap_or("chunked"),
            "HTTP Response Info"
        );

        if let Some(prop) = propagation {
            res.insert_header("X-Honeycomb-Trace", prop.marshal_trace_context());
        }

        Ok(res)
    }
}

#[tide::utils::async_trait]
impl<State: Clone + Send + Sync + 'static> Middleware<State> for TraceMiddleware {
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> tide::Result {
        self.handle(req, next).await
    }
}
