use std::{
    fs::File,
    io::{self, BufWriter, Write as _},
    iter,
    path::PathBuf,
};

use anyhow::Context as _;
use oxidris_ai::board_feature::ALL_BOARD_FEATURES;

use crate::{
    analysis,
    data::{self, FeatureStatistics},
    util,
};

#[derive(Default, Debug, Clone, clap::Args)]
pub(crate) struct GenerateBoardFeatureStatsArg {
    /// Boards data file path
    boards_file: PathBuf,
    /// Output file path
    #[arg(long)]
    output: Option<PathBuf>,
}

pub fn run(arg: &GenerateBoardFeatureStatsArg) -> anyhow::Result<()> {
    let GenerateBoardFeatureStatsArg {
        boards_file,
        output,
    } = arg;

    eprintln!("Loading boards from {}...", boards_file.display());
    let boards = data::load_board(boards_file)?;
    eprintln!("Loaded {} boards", boards.len());

    eprintln!("Computing featuress for all boards...");
    let boards_features = analysis::compute_all_features(&boards);
    eprintln!("Features computed");

    eprintln!("Computing statistics");
    let statistics = analysis::coimpute_statistics(&boards_features);
    eprintln!("Statistics computed");

    let writer = if let Some(output_path) = output {
        let file = File::create(output_path)
            .with_context(|| format!("Failed to create output file: {}", output_path.display()))?;
        Box::new(file) as Box<dyn io::Write>
    } else {
        Box::new(io::stdout().lock()) as Box<dyn io::Write>
    };
    let mut writer = BufWriter::new(writer);
    dump_source(&mut writer, &statistics).with_context(|| {
        format!(
            "Failed to write source code to {}",
            output
                .as_ref()
                .map_or_else(|| "stdout".to_string(), |p| p.display().to_string())
        )
    })?;
    writer.flush().with_context(|| {
        format!(
            "Failed to write source code to {}",
            output
                .as_ref()
                .map_or_else(|| "stdout".to_string(), |p| p.display().to_string())
        )
    })?;

    Ok(())
}

fn dump_source(writer: &mut dyn io::Write, statistics: &[FeatureStatistics]) -> anyhow::Result<()> {
    for (f, stats) in iter::zip(ALL_BOARD_FEATURES.as_array(), statistics) {
        writeln!(
            writer,
            "impl {} {{",
            f.type_name().replace("oxidris_ai::", "crate::")
        )?;
        for (kind, stats) in [
            ("RAW", &stats.raw),
            ("TRANSFORMED", &stats.transformed),
            ("NORMALIZED", &stats.normalized),
        ] {
            for percentile in [1u8, 5, 25, 50, 75, 95, 99] {
                let value = stats.get_percentile(f32::from(percentile)).unwrap();
                let s = util::format_f32(value);
                writeln!(writer, "    pub const {kind}_P{percentile:02}: f32 = {s};",)?;
            }
        }
        writeln!(writer, "}}")?;
        writeln!(writer)?;
    }

    Ok(())
}
