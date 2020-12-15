use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{fmt, io};

// Note: Rust-Analyzer sometimes has trouble with this use:: statement.
use env_logger::fmt::{Color, Formatter, Style, StyledValue};
use log::{kv, Level};

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
