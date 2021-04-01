use std::io::Write;
use std::{io, process};

use lazy_static::lazy_static;
use log::kv;

lazy_static! {
    static ref HOSTNAME: String = gethostname::gethostname().to_string_lossy().to_string();
}

// Modified from the json_env_logger crate
pub fn log_format_json<F>(f: &mut F, record: &log::Record<'_>) -> io::Result<()>
where
    F: Write,
{
    let target = record.target();
    if target.starts_with("tracing::span") {
        // Ignore tracing spans.
        return Ok(());
    }

    write!(f, "{{")?;
    write!(
        f,
        "\"level\":\"{}\"",
        format!("{}", record.level()).to_lowercase()
    )?;
    write!(f, ",\"pid\":{}", process::id())?;
    write!(f, ",\"message\":")?;
    write_json_str(f, &record.args().to_string())?;

    let mut visitor = Visitor { writer: f };
    record
        .key_values()
        .visit(&mut visitor)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    write!(f, ",\"target\":\"{}\"", target)?;
    write!(f, ",\"hostname\":\"{}\"", *HOSTNAME)?;
    write!(
        f,
        ",\"time\":\"{}\"",
        chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
    )?;

    struct Visitor<'w, W: Write> {
        writer: &'w mut W,
    }

    impl<'kvs, 'w, W: Write> kv::Visitor<'kvs> for Visitor<'w, W> {
        fn visit_pair(
            &mut self,
            key: kv::Key<'kvs>,
            val: kv::Value<'kvs>,
        ) -> Result<(), kv::Error> {
            write!(self.writer, ",\"{}\":\"{}\"", key, val)?;
            Ok(())
        }
    }

    writeln!(f, "}}")
}

// until log kv Value impl serde::Serialize
fn write_json_str<W: Write>(writer: &mut W, raw: &str) -> io::Result<()> {
    serde_json::to_writer(writer, raw)?;
    Ok(())
}

// Modified from the json_env_logger crate
#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn writes_records_as_json() -> Result<(), Box<dyn Error>> {
        let mut kvs = std::collections::HashMap::new();
        kvs.insert("a", "b");
        let record = log::Record::builder()
            .args(format_args!("hello"))
            .key_values(&kvs)
            .level(log::Level::Info)
            .build();
        let mut buf = Vec::new();
        log_format_json(&mut buf, &record)?;
        let output = std::str::from_utf8(&buf)?;
        println!("{}", output);
        assert!(serde_json::from_str::<serde_json::Value>(output).is_ok());
        Ok(())
    }

    #[test]
    fn escapes_json_strings() -> Result<(), Box<dyn Error>> {
        let mut buf = Vec::new();
        write_json_str(
            &mut buf, r#""
	"#,
        )?;
        assert_eq!("\"\\\"\\n\\t\"", std::str::from_utf8(&buf)?);
        Ok(())
    }
}
