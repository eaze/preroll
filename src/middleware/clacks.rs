use tide::{Middleware, Next, Request};

/// Add a clacks overhead header to every outgoing response.
#[derive(Debug, Default, Clone)]
pub struct ClacksMiddleware {
    _priv: (),
}

impl ClacksMiddleware {
    /// Create a new instance of `ClacksMiddleware`.
    #[must_use]
    pub fn new() -> Self {
        Self { _priv: () }
    }

    /// Ensure immortality.
    async fn handle<'a, State: Clone + Send + Sync + 'static>(
        &'a self,
        req: Request<State>,
        next: Next<'a, State>,
    ) -> tide::Result {
        let mut res = next.run(req).await;
        res.insert_header("X-Clacks-Overhead", "GNU/Terry Pratchett");

        Ok(res)
    }
}

#[tide::utils::async_trait]
impl<State: Clone + Send + Sync + 'static> Middleware<State> for ClacksMiddleware {
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> tide::Result {
        self.handle(req, next).await
    }
}
