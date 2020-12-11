use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{fmt, io, process};

// Note: Rust-Analyzer sometimes has trouble with this use:: statement.
use env_logger::fmt::{Color, Formatter, Style, StyledValue};
use lazy_static::lazy_static;
use log::{kv, Level};

// Literally copied from rustc.
// Currently "experimental" because the name is undecided.
// Docs: https://doc.rust-lang.org/std/any/fn.type_name_of_val.html
// Tracking issue: https://github.com/rust-lang/rust/issues/66359
pub fn type_name_of_val<T: ?Sized>(_val: &T) -> &'static str {
    std::any::type_name::<T>()
}

// Modified from the pretty_env_logger crate
pub fn log_format_pretty(f: &mut Formatter, record: &log::Record<'_>) -> io::Result<()> {
    let target = record.target();
    if target.starts_with("tracing::span") {
        // Ignore tracing spans.
        return Ok(());
    }

    let max_width = max_target_width(target);

    let mut style = f.style();
    let level = colored_level(&mut style, record.level());

    let mut style = f.style();
    let target = style.set_bold(true).value(Padded {
        value: target,
        width: max_width,
    });

    write!(f, "{} {} | {}", level, target, record.args(),)?;
    format_kv_pairs(f, record).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    writeln!(f)
}

struct Padded<T> {
    value: T,
    width: usize,
}

impl<T: fmt::Display> fmt::Display for Padded<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{: <width$}", self.value, width = self.width)
    }
}

static MAX_MODULE_WIDTH: AtomicUsize = AtomicUsize::new(0);

fn max_target_width(target: &str) -> usize {
    let max_width = MAX_MODULE_WIDTH.load(Ordering::Relaxed);
    if max_width < target.len() {
        MAX_MODULE_WIDTH.store(target.len(), Ordering::Relaxed);
        target.len()
    } else {
        max_width
    }
}

fn colored_level<'a>(style: &'a mut Style, level: Level) -> StyledValue<'a, &'static str> {
    match level {
        Level::Trace => style.set_color(Color::Magenta).value("TRACE"),
        Level::Debug => style.set_color(Color::Blue).value("DEBUG"),
        Level::Info => style.set_color(Color::Green).value("INFO "),
        Level::Warn => style.set_color(Color::Yellow).value("WARN "),
        Level::Error => style.set_color(Color::Red).value("ERROR"),
    }
}

// Heavily modified from the femme crate
fn format_kv_pairs(f: &mut Formatter, record: &log::Record<'_>) -> Result<(), kv::Error> {
    struct Visitor<'f> {
        f: &'f mut Formatter,
    }

    impl<'kvs, 'f> kv::Visitor<'kvs> for Visitor<'f> {
        fn visit_pair(
            &mut self,
            key: kv::Key<'kvs>,
            val: kv::Value<'kvs>,
        ) -> Result<(), kv::Error> {
            let mut style = self.f.style();
            let key = style.set_bold(true).value(key);

            write!(self.f, "\n  {} {}", key, val)?;
            Ok(())
        }
    }

    let mut visitor = Visitor { f };
    record.key_values().visit(&mut visitor)?;
    Ok(())
}

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
        "\"time\":\"{}\"",
        chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
    )?;
    write!(f, ",\"hostname\":\"{}\"", *HOSTNAME)?;
    write!(f, ",\"pid\":{}", process::id())?;
    write!(
        f,
        ",\"level\":\"{}\"",
        format!("{}", record.level()).to_lowercase()
    )?;
    write!(f, ",\"target\":\"{}\"", target)?;
    write!(f, ",\"message\":")?;
    write_json_str(f, &record.args().to_string())?;

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

    let mut visitor = Visitor { writer: f };
    record
        .key_values()
        .visit(&mut visitor)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    writeln!(f, "}}")
}

// until log kv Value impl serde::Serialize
fn write_json_str<W: Write>(writer: &mut W, raw: &str) -> io::Result<()> {
    serde_json::to_writer(writer, raw)?;
    Ok(())
}

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
