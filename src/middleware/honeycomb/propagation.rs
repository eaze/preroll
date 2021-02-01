// This file is copied from https://github.com/nlopes/beeline-rust
//
// MIT License from that repo:

// Copyright (c) 2019 Norberto Lopes
//
// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:
//
// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

/// assumes a header of the form:
///
/// VERSION;PAYLOAD

/// VERSION=1
/// =========
/// PAYLOAD is a list of comma-separated params (k=v pairs), with no spaces.  recognized
/// keys + value types:
///
///  trace_id=${traceId}    - traceId is an opaque ascii string which shall not include ','
///  parent_id=${spanId}    - spanId is an opaque ascii string which shall not include ','
///  dataset=${datasetId}   - datasetId is the slug for the honeycomb dataset to which downstream spans should be sent; shall not include ','
///  context=${contextBlob} - contextBlob is a base64 encoded json object.
///
/// ex: X-Honeycomb-Trace: 1;trace_id=weofijwoeifj,parent_id=owefjoweifj,context=SGVsbG8gV29ybGQ=
use super::errors::{BeelineError, Result};
use libhoney::Value;

pub const PROPAGATION_HTTP_HEADER: &str = "X-Honeycomb-Trace";
pub const PROPAGATION_VERSION: usize = 1;

/// Propagation contains all the information about a payload header
///  trace_id=${traceId}    - traceId is an opaque ascii string which shall not include ','
///  parent_id=${spanId}    - spanId is an opaque ascii string which shall not include ','
///  dataset=${datasetId}   - datasetId is the slug for the honeycomb dataset to which downstream spans should be sent; shall not include ','
///  context=${contextBlob} - contextBlob is a base64 encoded json object.
///
/// ex: X-Honeycomb-Trace: 1;trace_id=weofijwoeifj,parent_id=owefjoweifj,context=SGVsbG8gV29ybGQ=
#[derive(Debug, PartialEq)]
pub struct Propagation {
    pub trace_id: String,
    pub parent_id: String,
    pub dataset: String,
    pub trace_context: Value,
}

impl Propagation {
    pub fn unmarshal_trace_context(header: &str) -> Result<Self> {
        let ver: Vec<&str> = header.splitn(2, ';').collect();
        if ver[0] == "1" {
            return Propagation::unmarshal_trace_context_v1(ver[1]);
        }

        Err(BeelineError::PropagationError(format!(
            "unrecognized version for trace header {}",
            ver[0]
        )))
    }

    fn unmarshal_trace_context_v1(header: &str) -> Result<Self> {
        let clauses: Vec<&str> = header.split(',').collect();
        let (mut trace_id, mut parent_id, mut dataset, mut context) = (
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
        );

        for clause in clauses.iter() {
            let kv: Vec<&str> = clause.splitn(2, '=').collect();
            match kv[0] {
                "trace_id" => trace_id = kv[1].to_string(),
                "parent_id" => parent_id = kv[1].to_string(),
                "dataset" => dataset = kv[1].to_string(),
                "context" => context = kv[1].to_string(),
                _ => (),
            };
        }

        if trace_id.is_empty() && !parent_id.is_empty() {
            return Err(BeelineError::PropagationError(String::from(
                "parent_id without trace_id",
            )));
        }

        Ok(Propagation {
            trace_id,
            parent_id,
            dataset,
            trace_context: serde_json::from_slice(&base64::decode(&context).map_err(|e| {
                BeelineError::PropagationError(format!(
                    "unable to decode base64 trace context: {}",
                    e
                ))
            })?)
            .map_err(|e| {
                BeelineError::PropagationError(format!("unable to unmarshal trace context: {}", e))
            })?,
        })
    }

    pub fn marshal_trace_context(&self) -> String {
        let dataset = if !self.dataset.is_empty() {
            format!("dataset={},", self.dataset)
        } else {
            String::new()
        };

        format!(
            "{};trace_id={},parent_id={},{}context={}",
            PROPAGATION_VERSION,
            self.trace_id,
            self.parent_id,
            dataset,
            base64::encode(&self.trace_context.to_string())
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_marshal() {
        let mut p = Propagation {
            trace_id: "abcdef123456".to_string(),
            parent_id: "0102030405".to_string(),
            trace_context: json!({
                "userID": 1,
                "errorMsg": "failed to sign on",
                "toRetry":  true,
            }),
            dataset: "".to_string(),
        };
        assert_eq!(
            p.marshal_trace_context(),
            "1;trace_id=abcdef123456,parent_id=0102030405,context=eyJ1c2VySUQiOjEsImVycm9yTXNnIjoiZmFpbGVkIHRvIHNpZ24gb24iLCJ0b1JldHJ5Ijp0cnVlfQ=="
        );

        p.dataset = "dada".to_string();
        assert_eq!(
            p.marshal_trace_context(),
            "1;trace_id=abcdef123456,parent_id=0102030405,dataset=dada,context=eyJ1c2VySUQiOjEsImVycm9yTXNnIjoiZmFpbGVkIHRvIHNpZ24gb24iLCJ0b1JldHJ5Ijp0cnVlfQ=="
        );
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_unmarshal_with_dataset() {
        let p = Propagation {
            trace_id: "weofijwoeifj".to_string(),
            parent_id: "owefjoweifj".to_string(),
            dataset: "dada".to_string(),
            trace_context: json!({"key": "value"}),
        };
        assert_eq!(
            p,
            Propagation::unmarshal_trace_context(&p.marshal_trace_context()).unwrap()
        );
    }
}
