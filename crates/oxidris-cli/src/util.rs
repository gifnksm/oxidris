use std::{
    fs::File,
    io::{self, BufWriter, StdoutLock, Write as _},
    path::{Path, PathBuf},
};

use anyhow::Context;
use oxidris_analysis::{
    feature_builder::FeatureBuilder,
    normalization::BoardFeatureNormalizationParamCollection,
    sample::RawBoardSample,
    session::{SessionCollection, SessionData},
    statistics::RawFeatureStatistics,
    survival::SurvivalStatsMap,
};
use oxidris_evaluator::board_feature::{self, BoxedBoardFeature};

use crate::schema::ai_model::AiModel;

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
        self.flush()
            .with_context(|| format!("Failed to flush output to {}", self.display_path()))?;
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

pub fn read_json_file<T, P>(file_kind: &str, path: P) -> anyhow::Result<T>
where
    T: serde::de::DeserializeOwned,
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let file = File::open(path)
        .with_context(|| format!("Failed to open {} file: {}", file_kind, path.display()))?;

    let reader = io::BufReader::new(file);
    let value = serde_json::from_reader(reader).with_context(|| {
        format!(
            "Failed to parse {} JSON file: {}",
            file_kind,
            path.display()
        )
    })?;

    Ok(value)
}

/// Read board session data from a JSON file
///
/// # Arguments
///
/// * `path` - Path to the boards JSON file
///
/// # Returns
///
/// Deserialized session collection
///
/// # Errors
///
/// Returns error if file cannot be opened or parsed
pub fn read_boards_file<P>(path: P) -> anyhow::Result<SessionCollection>
where
    P: AsRef<Path>,
{
    read_json_file("boards", path)
}

/// Read AI model configuration from a JSON file
///
/// # Arguments
///
/// * `path` - Path to the AI model JSON file
///
/// # Returns
///
/// Deserialized AI model configuration
///
/// # Errors
///
/// Returns error if file cannot be opened or parsed
pub fn read_ai_model_file<P>(path: P) -> anyhow::Result<AiModel>
where
    P: AsRef<Path>,
{
    read_json_file("AI model", path)
}

/// Feature set selection for feature building
///
/// Determines which types of features to construct from session data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureSet {
    /// All features including raw and table transforms
    All,
    /// Only features with Kaplan-Meier normalization
    Km,
    /// Only raw-transform features (penalty/risk)
    Raw,
}

/// Build board features from session data with computed normalization parameters
///
/// This function performs the complete feature engineering pipeline:
///
/// 1. Extract raw feature values from all boards in sessions
/// 2. Compute raw feature statistics (percentiles)
/// 3. Compute survival statistics grouped by feature values (Kaplan-Meier)
/// 4. Generate normalization parameters from statistics
/// 5. Build feature set (raw only or all including table transforms)
///
/// # Arguments
///
/// * `feature_set` - Which features to build (All or Raw only)
/// * `sessions` - Session data containing board states and placements
///
/// # Returns
///
/// Vector of boxed board features with runtime normalization
///
/// # Errors
///
/// Returns error if normalization parameters cannot be computed or
/// features cannot be constructed
///
/// # Performance
///
/// This function processes all boards in all sessions multiple times:
/// - Once for raw value extraction
/// - Once for survival analysis
///
/// For large datasets, this may take significant time and memory.
pub fn build_feature_from_session(
    feature_set: FeatureSet,
    sessions: &[SessionData],
) -> anyhow::Result<Vec<BoxedBoardFeature>> {
    let sources = board_feature::source::all_board_feature_sources();

    eprintln!("Computing feature raw values for all boards...");
    let raw_samples = RawBoardSample::from_sessions(&sources, sessions);
    eprintln!("Extracted {} raw samples", raw_samples.len());

    eprintln!("Computing raw feature statistics...");
    let raw_stats = RawFeatureStatistics::from_samples(&sources, &raw_samples);
    eprintln!("Raw feature statistics computed");

    eprintln!("Computing feature survival statistics...");
    let survival_stats = SurvivalStatsMap::collect_all_by_feature_value(sessions, &sources);
    eprintln!("Feature survival statistics computed");

    eprintln!("Computing feature normalization parameters...");
    let norm_params =
        BoardFeatureNormalizationParamCollection::from_stats(&sources, &raw_stats, &survival_stats);
    eprintln!("Normalization parameters computed");

    eprintln!("Building feature builder...");
    let feature_builder = FeatureBuilder::new(norm_params);
    let features = match feature_set {
        FeatureSet::All => feature_builder.build_all_features()?,
        FeatureSet::Km => feature_builder.build_km_features()?,
        FeatureSet::Raw => feature_builder.build_raw_features()?,
    };
    eprintln!("Built {} features", features.len());

    Ok(features)
}
