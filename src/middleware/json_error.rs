use super::extension_types::{CorrelationId, RequestId};
use serde::{Deserialize, Serialize};
use tide::{Body, Middleware, Next, Request, Result};

#[cfg(feature = "honeycomb")]
use eaze_tracing_honeycomb::TraceId;

#[cfg(feature = "test")]
use uuid::Uuid;

/// Transfrom Errors (`Result::Err`) into JSON responses.
///
/// Special care is taken when handling non-4XX errors to not expose internal error messages.
#[derive(Debug, Default, Clone)]
pub struct JsonErrorMiddleware {
    _priv: (),
}

struct JsonErrorMiddlewareHasBeenRun;

/// The structure of an error as formatted by preroll's error handling middleware.
///
/// A service using preroll will always respond with a JSON body in this format if an internal or client error occurs.
///
/// An example of the structure as it would be in JSON:
/// ```text
/// {
///   "status": 422,
///   "title": "Unprocessable Entity",
///   "message": "missing field \"address\"",
///   "request_id": "00000000-0000-0000-0000-000000000000"
///   "correlation_id": null,
/// }
/// ```
#[derive(Debug, Deserialize, Serialize)]
pub struct JsonError {
    /// The http status code. Refer to [httpstatuses.com](https://httpstatuses.com/) for a nice reference.
    pub status: u16,
    /// The 'canonical reason' of the http status code as specified in [rfc7231 section 6.1](https://tools.ietf.org/html/rfc7231#section-6.1),
    /// implemented via [`http_types::StatusCode`](https://docs.rs/http-types/2.9.0/http_types/enum.StatusCode.html).
    pub title: String,
    /// The origin error message for 4XX client errors.
    ///
    /// In case of an 5XX internal server error, this field will be `"Internal Server Error (correlation_id=00000000-0000-0000-0000-000000000000)"`.
    ///
    /// If the original error context is missing, this field will be `"(no additional context)"`.
    pub message: String,
    /// The UUID v4 assigned to the request, possibly from an incoming header.
    pub request_id: RequestId,
    /// The service-unique UUID v4 assigned to the error response for 5XX internal server errors.
    pub correlation_id: Option<String>,
    #[cfg(feature = "honeycomb")]
    #[cfg_attr(feature = "docs", doc(cfg(feature = "honeycomb")))]
    /// If the `honeycomb` feature is enabled, this will be the honeycomb trace id associated with this request.
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

        #[cfg(feature = "honeycomb")]
        let honeycomb_trace_id = req.ext::<TraceId>().cloned();

        let mut res = next.run(req).await;
        let status = res.status();

        if status.is_server_error() {
            #[cfg(not(feature = "test"))]
            let correlation_id = CorrelationId::new();
            #[cfg(feature = "test")]
            let correlation_id: CorrelationId = Uuid::nil().into();

            let body = JsonError {
                title: status.canonical_reason().to_string(),
                message: format!("Internal Server Error (correlation_id={})", correlation_id),
                status: status as u16,
                request_id,
                correlation_id: Some(correlation_id.to_string()),
                #[cfg(feature = "honeycomb")]
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
                    #[cfg(feature = "honeycomb")]
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
                    #[cfg(feature = "honeycomb")]
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
