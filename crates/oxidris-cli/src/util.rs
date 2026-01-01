use std::{
    fs::File,
    io::{self, BufWriter, StdoutLock, Write as _},
    path::PathBuf,
};

use anyhow::Context;

#[derive(Debug)]
pub enum Output {
    Stdout {
        writer: StdoutLock<'static>,
    },
    File {
        writer: BufWriter<File>,
        path: PathBuf,
    },
}

impl Output {
    pub fn save_json<T>(value: &T, output_path: Option<PathBuf>) -> anyhow::Result<()>
    where
        T: serde::Serialize,
    {
        let mut output = Output::from_output_path(output_path)?;
        output.write_json(value)
    }

    pub fn from_output_path(output_path: Option<PathBuf>) -> anyhow::Result<Self> {
        match output_path {
            Some(path) => Output::open(path),
            None => Ok(Output::stdout()),
        }
    }

    pub fn stdout() -> Self {
        Output::Stdout {
            writer: io::stdout().lock(),
        }
    }

    pub fn open(path: PathBuf) -> anyhow::Result<Self> {
        let file = File::create(&path)
            .with_context(|| format!("Failed to create output file: {}", path.display()))?;
        Ok(Output::File {
            writer: BufWriter::new(file),
            path,
        })
    }

    pub fn display_path(&self) -> String {
        match self {
            Output::Stdout { .. } => "stdout".to_string(),
            Output::File { path, .. } => path.display().to_string(),
        }
    }

    pub fn write_json<T>(&mut self, value: T) -> anyhow::Result<()>
    where
        T: serde::Serialize,
    {
        serde_json::to_writer_pretty(&mut *self, &value)
            .with_context(|| format!("Failed to write JSON to {}", self.display_path()))?;
        writeln!(&mut *self).with_context(|| {
            format!(
                "Failed to write newline after JSON to {}",
                self.display_path()
            )
        })?;
        Ok(())
    }
}

impl io::Write for Output {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Output::Stdout { writer } => writer.write(buf),
            Output::File { writer, .. } => writer.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Output::Stdout { writer } => writer.flush(),
            Output::File { writer, .. } => writer.flush(),
        }
    }
}

pub fn format_f32(weight: f32) -> String {
    let s = format!("{weight:?}");
    let Some((int_part, frac_part)) = s
        .split_once('.')
        .filter(|(_int_part, frac_part)| frac_part.len() > 3)
    else {
        return s;
    };
    let (parts, tail) = frac_part.as_bytes().as_chunks::<3>();
    let mut result = String::from(int_part);
    result.push('.');
    let mut first = true;
    for part in parts {
        if !first {
            result.push('_');
        }
        result.push_str(str::from_utf8(part).unwrap());
        first = false;
    }
    if !tail.is_empty() {
        if !first {
            result.push('_');
        }
        result.push_str(str::from_utf8(tail).unwrap());
    }
    result
}
