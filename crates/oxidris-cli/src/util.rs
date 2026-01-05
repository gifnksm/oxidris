use std::{
    fs::File,
    io::{self, BufWriter, StdoutLock, Write as _},
    path::PathBuf,
};

use anyhow::Context;
use oxidris_evaluator::board_feature::{self, BoxedBoardFeature};

use crate::{
    analysis::{
        BoardFeatureNormalizationParamCollection, FeatureBuilder, RawBoardSample,
        RawFeatureStatistics,
    },
    model::session::SessionData,
};

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

pub fn build_feature_from_session(
    sessions: &[SessionData],
) -> anyhow::Result<Vec<BoxedBoardFeature>> {
    let sources = board_feature::all_board_feature_sources();

    eprintln!("Computing feature raw values for all boards...");
    let raw_samples = RawBoardSample::from_sessions(&sources, sessions);
    eprintln!("Extracted {} raw samples", raw_samples.len());

    eprintln!("Computing raw feature statistics...");
    let raw_stats = RawFeatureStatistics::from_samples(&sources, &raw_samples);
    eprintln!("Raw feature statistics computed");

    eprintln!("Computing feature normalization parameters...");
    let norm_params = BoardFeatureNormalizationParamCollection::from_stats(&sources, &raw_stats);
    eprintln!("Normalization parameters computed");

    eprintln!("Building feature builder...");
    let feature_builder = FeatureBuilder::new(norm_params);
    let features = feature_builder.build_all_features()?;
    eprintln!("Feature builder built");

    Ok(features)
}
