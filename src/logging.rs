use bevy::log::{BoxedFmtLayer, tracing, tracing_subscriber};
use bevy::prelude::App;
use std::fmt;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields, FormattedFields, format::Writer, time::FormatTime};
use tracing_subscriber::registry::LookupSpan;

/// Replaces Bevy's terminal formatter without changing event targets or filters.
pub(crate) fn fmt_layer(_: &mut App) -> Option<BoxedFmtLayer> {
  Some(Box::new(
    tracing_subscriber::fmt::layer()
      .event_format(LogFormatter)
      .with_writer(std::io::stderr),
  ))
}

// Fixed character width keeps message starts aligned. Overflow is removed from the left.
const TARGET_WIDTH: usize = 40;

/// Custom log formatter that drops the date, abbreviates the module path, aligns the log message start, and dims the
/// log message text colour for DEBUG and below.
#[derive(Debug)]
struct LogFormatter;

fn abbreviated_target(target: &str) -> String {
  let mut abbreviated = String::with_capacity(target.len());
  let mut segments = target.split("::").peekable();
  while let Some(segment) = segments.next() {
    if segments.peek().is_none() {
      abbreviated.push_str(segment);
    } else {
      match segment {
        "procedural_generation_2" => abbreviated.push_str("pg2"),
        _ => abbreviated.extend(segment.chars().take(1)),
      }
      abbreviated.push_str("::");
    }
  }
  abbreviated
}

impl<S, N> FormatEvent<S, N> for LogFormatter
where
  S: tracing::Subscriber + for<'lookup> LookupSpan<'lookup>,
  N: for<'writer> FormatFields<'writer> + 'static,
{
  fn format_event(&self, ctx: &FmtContext<'_, S, N>, mut writer: Writer<'_>, event: &tracing::Event<'_>) -> fmt::Result {
    // Reuse tracing's UTC clock and fractional precision, dropping only the date
    let mut timestamp = String::new();
    tracing_subscriber::fmt::time::SystemTime.format_time(&mut Writer::new(&mut timestamp))?;
    let (_, time) = timestamp.split_once('T').ok_or(fmt::Error)?;
    let metadata = event.metadata();
    let target = abbreviated_target(metadata.target());
    let excess = target.chars().count().saturating_sub(TARGET_WIDTH);
    let start = target.char_indices().nth(excess).map_or(target.len(), |(index, _)| index);
    let target = &target[start..];
    let (dimmed, reset, level_colour) = if writer.has_ansi_escapes() {
      let level_colour = match *metadata.level() {
        tracing::Level::TRACE => "\x1b[35m",
        tracing::Level::DEBUG => "\x1b[34m",
        tracing::Level::INFO => "\x1b[32m",
        tracing::Level::WARN => "\x1b[33m",
        tracing::Level::ERROR => "\x1b[31m",
      };
      ("\x1b[2m", "\x1b[0m", level_colour)
    } else {
      ("", "", "")
    };
    write!(
      writer,
      "{dimmed}{time}{reset} {level_colour}{:>5}{reset} {dimmed}{target:<TARGET_WIDTH$}:{reset} ",
      metadata.level()
    )?;
    let dim_message = matches!(*metadata.level(), tracing::Level::TRACE | tracing::Level::DEBUG);
    if dim_message {
      writer.write_str(dimmed)?;
    }
    ctx.format_fields(writer.by_ref(), event)?;
    if dim_message {
      writer.write_str(reset)?;
    }
    // Keep span context after the message so nested spans cannot shift its starting column
    if let Some(scope) = ctx.event_scope() {
      let bold = if writer.has_ansi_escapes() { "\x1b[1m" } else { "" };
      writer.write_str(" [")?;
      for (index, span) in scope.from_root().enumerate() {
        if index > 0 {
          write!(writer, "{dimmed}:{reset}")?;
        }
        write!(writer, "{bold}{}{reset}", span.metadata().name())?;
        let extensions = span.extensions();
        if let Some(fields) = extensions.get::<FormattedFields<N>>().filter(|fields| !fields.is_empty()) {
          write!(writer, "{bold}{{{reset}{fields}{bold}}}{reset}")?;
        }
      }
      writer.write_str("]")?;
    }
    writeln!(writer)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::io::{self, Write};
  use std::sync::{Arc, Mutex};
  use tracing_subscriber::prelude::*;

  #[derive(Clone, Default)]
  struct Buffer(Arc<Mutex<Vec<u8>>>);

  impl Write for Buffer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
      self.0.lock().unwrap().extend_from_slice(bytes);
      Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
      Ok(())
    }
  }

  fn capture(ansi: bool, filter: &str, emit: impl FnOnce()) -> String {
    let output = Buffer::default();
    let writer = output.clone();
    let subscriber = tracing_subscriber::registry()
      .with(tracing_subscriber::EnvFilter::new(filter))
      .with(
        tracing_subscriber::fmt::layer()
          .event_format(LogFormatter)
          .with_ansi(ansi)
          .with_writer(move || writer.clone()),
      );

    tracing::subscriber::with_default(subscriber, emit);

    String::from_utf8(output.0.lock().unwrap().clone()).unwrap()
  }

  #[test]
  fn fmt_layer_builds_a_bevy_formatter_override() {
    assert!(fmt_layer(&mut App::new()).is_some());
  }

  #[test]
  fn fmt_layer_preserves_original_module_targets_for_filtering() {
    let output = capture(
      false,
      "off,procedural_generation_2::generation::object::model::object_grid=debug",
      || {
        tracing::debug!(target: "procedural_generation_2::generation::object::model::object_grid", "Visible");
        tracing::trace!(target: "procedural_generation_2::generation::object::model::object_grid", "Too verbose");
        tracing::debug!(target: "procedural_generation_2::generation::object::model::cell", "Other module");
        tracing::debug!(target: "pg2::g::o::m::object_grid", "Abbreviated target");
      },
    );

    assert_eq!(output.lines().count(), 1);
    assert_eq!(output[23..63].trim_end(), "pg2::g::o::m::object_grid");
    assert_eq!(&output[65..], "Visible\n");
  }

  #[test]
  fn fmt_layer_preserves_structured_fields_and_span_context_without_shifting_messages() {
    let output = capture(false, "trace", || {
      let generation = tracing::info_span!("generation", chunk = 5);
      let _generation = generation.enter();
      let attempt = tracing::debug_span!("attempt", retry = 2);
      let _attempt = attempt.enter();
      tracing::debug!(target: "short", updates = 895, "Validated");
    });

    assert_eq!(
      &output[65..],
      "Validated updates=895 [generation{chunk=5}:attempt{retry=2}]\n"
    );
  }

  #[test]
  fn fmt_layer_dims_only_trace_and_debug_messages() {
    let output = capture(true, "trace", || {
      tracing::trace!(target: "short", "Trace {}", "message");
      tracing::debug!(target: "short", "Debug {}", "message");
      tracing::info!(target: "short", "Info message");
      tracing::warn!(target: "short", "Warn message");
      tracing::error!(target: "short", "Error message");
    });

    assert_eq!(output.lines().count(), 5);
    for (line, expected) in output.lines().zip([
      "\x1b[2mTrace message\x1b[0m",
      "\x1b[2mDebug message\x1b[0m",
      "Info message",
      "Warn message",
      "Error message",
    ]) {
      let (_, message) = line.split_once(":\x1b[0m ").unwrap();
      assert_eq!(message, expected);
    }
  }

  #[test]
  fn fmt_layer_uses_standard_level_colours_only_when_ansi_is_enabled() {
    fn emit_levels() {
      tracing::trace!(target: "short", "Trace message");
      tracing::debug!(target: "short", "Debug message");
      tracing::info!(target: "short", "Info message");
      tracing::warn!(target: "short", "Warn message");
      tracing::error!(target: "short", "Error message");
    }

    let coloured = capture(true, "trace", emit_levels);
    let plain = capture(false, "trace", emit_levels);

    for (line, expected) in coloured.lines().zip([
      "\x1b[35mTRACE\x1b[0m",
      "\x1b[34mDEBUG\x1b[0m",
      "\x1b[32m INFO\x1b[0m",
      "\x1b[33m WARN\x1b[0m",
      "\x1b[31mERROR\x1b[0m",
    ]) {
      assert!(line.contains(expected), "Missing standard level colour in {line:?}");
      assert!(line.starts_with("\x1b[2m"), "Timestamp should be dimmed");
      assert!(line.contains("\x1b[2mshort"), "Target should be dimmed");
    }
    assert_eq!(coloured.lines().count(), 5);
    assert_eq!(plain.lines().count(), 5);
    assert!(!plain.contains('\x1b'));
    for (line, message) in plain.lines().zip([
      "Trace message",
      "Debug message",
      "Info message",
      "Warn message",
      "Error message",
    ]) {
      assert_eq!(&line[65..], message);
    }
  }

  #[test]
  fn fmt_layer_truncates_targets_from_the_left_at_character_boundaries() {
    let output = capture(false, "trace", || {
      tracing::info!(target: "parent::prefix_0123456789012345678901234567890123456789", "Long leaf");
      tracing::info!(target: "parent::é123456789012345678901234567890123456789", "Unicode leaf");
    });

    let lines: Vec<_> = output.lines().collect();
    assert_eq!(&lines[0][23..], "0123456789012345678901234567890123456789: Long leaf");
    assert_eq!(&lines[1][23..], "é123456789012345678901234567890123456789: Unicode leaf");
    for line in lines {
      assert_eq!(
        line.chars().position(|character| character == 'L' || character == 'U'),
        Some(65)
      );
    }
  }

  #[test]
  fn fmt_layer_emits_precise_time_and_abbreviated_target_without_date() {
    let output = capture(false, "trace", || {
      tracing::debug!(
        target: "procedural_generation_2::generation::object::model::object_grid",
        "Validated object grid cg(5, 4)"
      );
    });

    let line = output.trim_end();
    let time = &line[..16];
    assert_eq!(time.as_bytes()[2], b':');
    assert_eq!(time.as_bytes()[5], b':');
    assert_eq!(time.as_bytes()[8], b'.');
    assert_eq!(time.as_bytes()[15], b'Z');
    assert!(time.replace([':', '.', 'Z'], "").bytes().all(|byte| byte.is_ascii_digit()));
    assert_eq!(&line[17..22], "DEBUG");
    assert_eq!(line[23..63].trim_end(), "pg2::g::o::m::object_grid");
    assert_eq!(&line[63..], ": Validated object grid cg(5, 4)");
  }
}
