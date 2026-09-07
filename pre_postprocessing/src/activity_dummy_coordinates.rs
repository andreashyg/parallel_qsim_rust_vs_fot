use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;

/// Adds `x="0" y="0"` to event lines of type `actstart`/`actend` when coordinates are missing.
pub fn add_dummy_coordinates_to_line(line: &str) -> String {
    let is_target_event = line.contains("<event")
        && (line.contains("type=\"actstart\"") || line.contains("type=\"actend\""));
    let has_coordinates = line.contains("x=\"") || line.contains("y=\"");

    if !is_target_event || has_coordinates {
        return line.to_string();
    }

    if let Some(closing_index) = line.rfind("/>") {
        let mut insertion_index = closing_index;
        while insertion_index > 0
            && line
                .as_bytes()
                .get(insertion_index - 1)
                .is_some_and(u8::is_ascii_whitespace)
        {
            insertion_index -= 1;
        }

        let mut out = String::with_capacity(line.len() + 14);
        out.push_str(&line[..insertion_index]);
        out.push_str(" x=\"0\" y=\"0\"");
        out.push_str(&line[insertion_index..]);
        out
    } else {
        line.to_string()
    }
}

/// Rewrites the XML-like input line by line and returns the number of modified lines.
pub fn add_dummy_coordinates_to_file(input: &Path, output: &Path) -> std::io::Result<usize> {
    let reader = open_reader(input)?;
    let mut writer = open_writer(output)?;
    let mut changed_lines = 0usize;

    for line in reader.lines() {
        let line = line?;
        let updated = add_dummy_coordinates_to_line(&line);
        if updated != line {
            changed_lines += 1;
        }
        writeln!(writer, "{updated}")?;
    }

    writer.flush()?;
    writer.finish()?;
    Ok(changed_lines)
}

fn open_reader(path: &Path) -> std::io::Result<Box<dyn BufRead>> {
    let file = File::open(path)?;
    match path.extension().and_then(|e| e.to_str()) {
        Some("gz") => Ok(Box::new(BufReader::new(GzDecoder::new(file)))),
        _ => Ok(Box::new(BufReader::new(file))),
    }
}

fn open_writer(path: &Path) -> std::io::Result<OutputWriter> {
    let file = File::create(path)?;
    let base = BufWriter::new(file);
    match path.extension().and_then(|e| e.to_str()) {
        Some("gz") => Ok(OutputWriter::Gz(GzEncoder::new(base, Compression::fast()))),
        _ => Ok(OutputWriter::Plain(base)),
    }
}

enum OutputWriter {
    Plain(BufWriter<File>),
    Gz(GzEncoder<BufWriter<File>>),
}

impl Write for OutputWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Plain(w) => w.write(buf),
            Self::Gz(w) => w.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Plain(w) => w.flush(),
            Self::Gz(w) => w.flush(),
        }
    }
}

impl OutputWriter {
    fn finish(self) -> std::io::Result<()> {
        match self {
            Self::Plain(mut w) => w.flush(),
            Self::Gz(mut w) => {
                w.try_finish()?;
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::add_dummy_coordinates_to_line;

    #[test]
    fn adds_coordinates_for_actend() {
        let line = r#"<event time="0.0" type="actend" person="0" link="0_1" actType="dummy"  />"#;
        let expected = r#"<event time="0.0" type="actend" person="0" link="0_1" actType="dummy" x="0" y="0"  />"#;
        assert_eq!(expected, add_dummy_coordinates_to_line(line));
    }

    #[test]
    fn adds_coordinates_for_actstart() {
        let line = r#"<event time="1.0" type="actstart" person="1" link="2_3" actType="home" />"#;
        let expected = r#"<event time="1.0" type="actstart" person="1" link="2_3" actType="home" x="0" y="0" />"#;
        assert_eq!(expected, add_dummy_coordinates_to_line(line));
    }

    #[test]
    fn keeps_other_types_unchanged() {
        let line = r#"<event time="2.0" type="entered link" person="1" link="2_3" />"#;
        assert_eq!(line, add_dummy_coordinates_to_line(line));
    }

    #[test]
    fn keeps_existing_coordinates_unchanged() {
        let line = r#"<event time="0.0" type="actend" person="0" link="0_1" x="4" y="5" />"#;
        assert_eq!(line, add_dummy_coordinates_to_line(line));
    }
}
