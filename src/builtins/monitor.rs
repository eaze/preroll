use std::env;
use std::sync::Arc;
use std::time::Instant;

use once_cell::sync::OnceCell;
use serde::Serialize;
use tide::{Body, Server};

use crate::utils::HOSTNAME;

static SERVICE_NAME: OnceCell<&'static str> = OnceCell::new();
static START_TIME: OnceCell<Instant> = OnceCell::new();

pub fn setup_monitor<State>(service_name: &'static str, server: &mut Server<Arc<State>>)
where
    State: Send + Sync + 'static,
{
    SERVICE_NAME.set(service_name).ok();
    START_TIME.set(Instant::now()).ok();

    server.at("/monitor/ping").get(|_| async {
        Ok(*SERVICE_NAME
            .get()
            .unwrap_or(&"service name not initialized"))
    });

    server.at("/monitor/status").get(|_| async {
        let status = Status {
            git: env::var("GIT_COMMIT")
                .unwrap_or_else(|_| "No GIT_COMMIT environment variable.".to_string()),
            hostname: &*HOSTNAME,
            service: *SERVICE_NAME
                .get()
                .unwrap_or(&"service name not initialized"),
            uptime: START_TIME
                .get()
                .map(|start| start.elapsed().as_secs_f64())
                .unwrap_or(f64::NEG_INFINITY),
        };

        Body::from_json(&status)
    });
}

#[derive(Serialize)]
struct Status<'host> {
    git: String,
    hostname: &'host str,
    service: &'static str,
    uptime: f64,
}

// TODO(Jeremiah):
//
// Add more status fields, similar to Boltzmann.js:
//
// {
//     "downstream": {
//         "postgresReachability": {
//             "error": null,
//             "latency": 2,
//             "status": "healthy"
//         },
//         "redisReachability": {
//             "error": null,
//             "latency": 2,
//             "status": "healthy"
//         }
//     },
//     "memory": {
//         "rss": 87212032
//     },
//     "stats": {
//         "requestCount": 63425,
//         "statuses": {
//             "200": 50024,
//             "202": 7963,
//             "204": 5404,
//             "500": 34
//         }
//     },
// }
