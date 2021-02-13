use tide::{Middleware, Next, Request};

#[cfg(feature = "test")]
use uuid::Uuid;

use super::extension_types::RequestId;

/// Attach a RequestId UUID to every request.
#[derive(Debug, Default, Clone)]
pub struct RequestIdMiddleware {
    _priv: (),
}

impl RequestIdMiddleware {
    /// Create a new instance of `RequestIdMiddleware`.
    #[must_use]
    pub fn new() -> Self {
        Self { _priv: () }
    }

    /// Attach a UUID to every request.
    async fn handle<'a, State: Clone + Send + Sync + 'static>(
        &'a self,
        mut req: Request<State>,
        next: Next<'a, State>,
    ) -> tide::Result {
        if req.ext::<RequestId>().is_some() {
            return Ok(next.run(req).await);
        }

        let request_id: RequestId;
        #[cfg(not(feature = "test"))]
        if let Some(header) = req.header("X-Request-Id") {
            request_id = match header.last().as_str().parse() {
                Ok(id) => id,
                Err(e) => {
                    log::warn!("Invalid X-Request-Id: \"{}\" - Error: {}", header, e);
                    RequestId::new()
                }
            };
        } else {
            request_id = RequestId::new();
        }
        #[cfg(feature = "test")]
        {
            request_id = Uuid::nil().into()
        }

        req.set_ext(request_id.clone());

        let mut res = next.run(req).await;

        res.insert_header("X-Request-Id", request_id.as_str());

        Ok(res)
    }
}

#[tide::utils::async_trait]
impl<State: Clone + Send + Sync + 'static> Middleware<State> for RequestIdMiddleware {
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> tide::Result {
        self.handle(req, next).await
    }
}
