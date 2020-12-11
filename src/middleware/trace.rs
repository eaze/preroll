use tide::{Middleware, Next, Request};
use tracing::instrument;
use tracing_honeycomb::{register_dist_tracing_root, SpanId, TraceId};
use uuid::Uuid;

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
            let prop = Propagation::unmarshal_trace_context(header.as_str())?;
            trace_id = Uuid::parse_str(&prop.trace_id)?
                .as_u128()
                .to_string()
                .parse()?;
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
        } else if let Some(req_id) = req.ext::<RequestId>() {
            // Awful hacks around tracing-honeycomb's terrible TraceId api.
            trace_id = req_id.as_str().parse()?;
        } else {
            trace_id = TraceId::generate();
        }

        req.set_ext(trace_id);

        if let Err(error) = register_dist_tracing_root(trace_id, parent_span) {
            log::error!("Failed to set honeycomb trace root: {:?}", error);
        }

        match tracing_honeycomb::current_dist_trace_ctx() {
            Ok((trace_id, span_id)) => {
                log::info!("current_dist_trace_ctx: ({}, {})", trace_id, span_id)
            }
            Err(error) => log::error!("Failed to get current_dist_trace_ctx: {:?}", error),
        }

        let mut res = next.run(req).await;

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
