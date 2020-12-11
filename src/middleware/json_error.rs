use super::extension_types::{CorrelationId, RequestId};
use serde::{Deserialize, Serialize};
use tide::{Body, Middleware, Next, Request, Result};
use tracing_honeycomb::TraceId;

/// Transfrom Errors (`Result::Err`) into JSON responses.
///
/// Special care is taken when handling non-4XX errors to not expose internal error messages.
#[derive(Debug, Default, Clone)]
pub struct JsonErrorMiddleware {
    _priv: (),
}

struct JsonErrorMiddlewareHasBeenRun;

#[derive(Debug, Deserialize, Serialize)]
pub struct JsonError {
    pub title: String,
    pub message: String,
    pub status: u16,
    pub request_id: RequestId,
    pub correlation_id: Option<String>,
    pub honeycomb_trace_id: Option<String>,
}

impl JsonErrorMiddleware {
    /// Create a new instance of `JsonErrorMiddleware`.
    #[must_use]
    pub fn new() -> Self {
        Self { _priv: () }
    }

    /// Log a request and a response.
    async fn handle<'a, State: Clone + Send + Sync + 'static>(
        &'a self,
        mut req: Request<State>,
        next: Next<'a, State>,
    ) -> Result {
        if req.ext::<JsonErrorMiddlewareHasBeenRun>().is_some() {
            return Ok(next.run(req).await);
        }
        req.set_ext(JsonErrorMiddlewareHasBeenRun);

        let request_id = req
            .ext::<RequestId>()
            .expect("RequestIdMiddleware must be installed before JsonErrorMiddleware.")
            .clone();

        let honeycomb_trace_id = req.ext::<TraceId>().cloned();

        let mut res = next.run(req).await;
        let status = res.status();

        if status.is_server_error() {
            let correlation_id = CorrelationId::new();

            let body = JsonError {
                title: status.canonical_reason().to_string(),
                message: format!("Internal Server Error (correlation_id={})", correlation_id),
                status: status as u16,
                request_id,
                correlation_id: Some(correlation_id.to_string()),
                honeycomb_trace_id: honeycomb_trace_id.map(|v| v.to_string()),
            };
            res.set_body(Body::from_json(&body)?);

            res.insert_header("X-Correlation-Id", correlation_id.as_str());

            // Set the Correlation Id on the Response so we can use it from the LogMiddleware.
            res.insert_ext(correlation_id);

            return Ok(res);
        }

        // We could also downcast for specific errors to get more precise infomation.
        // Example for async_std::io::Error:
        //
        // if let Some(err) = res.downcast_error::<async_std::io::Error>() {
        //     if let ErrorKind::NotFound = err.kind() {
        //         let msg = format!("Error: {:?}", err);
        //         res.set_status(StatusCode::NotFound);
        //
        //         res.set_body(msg);
        //     }
        // }
        // Ok(res)

        if status.is_client_error() {
            if let Some(error) = res.error() {
                let body = JsonError {
                    title: status.canonical_reason().to_string(),
                    message: format!("{:?}", error),
                    status: status as u16,
                    request_id,
                    correlation_id: None,
                    honeycomb_trace_id: honeycomb_trace_id.map(|v| v.to_string()),
                };
                res.set_body(Body::from_json(&body)?);
            } else {
                let body = JsonError {
                    title: status.canonical_reason().to_string(),
                    message: "(no additional context)".to_string(),
                    status: status as u16,
                    request_id,
                    correlation_id: None,
                    honeycomb_trace_id: honeycomb_trace_id.map(|v| v.to_string()),
                };
                res.set_body(Body::from_json(&body)?);
            }

            return Ok(res);
        }

        Ok(res)
    }
}

#[tide::utils::async_trait]
impl<State: Clone + Send + Sync + 'static> Middleware<State> for JsonErrorMiddleware {
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> Result {
        self.handle(req, next).await
    }
}
