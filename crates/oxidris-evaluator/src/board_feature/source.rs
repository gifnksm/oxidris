//! Feature source types for extracting raw values from board states.
//!
//! These types implement [`BoardFeatureSource`] and define how to extract raw measurements
//! from placement analysis. They are wrapped by feature types (e.g., [`RawTransform`](super::transform::RawTransform),
//! [`LineClearBonus`](super::transform::LineClearBonus)) to provide transformation and normalization.

use std::fmt;

use crate::placement_analysis::PlacementAnalysis;

#[must_use]
pub fn all_board_feature_sources() -> Vec<BoxedBoardFeatureSource> {
    vec![
        // survival features
        Box::new(NumHoles),
        Box::new(SumOfHoleDepth),
        Box::new(MaxHeight),
        Box::new(CenterColumnMaxHeight),
        Box::new(TotalHeight),
        // structure features
        Box::new(SurfaceBumpiness),
        Box::new(SurfaceRoughness),
        Box::new(RowTransitions),
        Box::new(ColumnTransitions),
        Box::new(SumOfWellDepth),
        // score features
        Box::new(NumClearedLines),
        Box::new(EdgeIWellDepth),
    ]
}

pub trait BoardFeatureSource: fmt::Debug + Send + Sync {
    #[must_use]
    fn id(&self) -> &str;
    #[must_use]
    fn name(&self) -> &str;
    #[must_use]
    fn type_name(&self) -> &str {
        std::any::type_name::<Self>()
    }
    #[must_use]
    fn clone_boxed(&self) -> BoxedBoardFeatureSource;
    #[must_use]
    fn extract_raw(&self, analysis: &PlacementAnalysis) -> u32;
}

pub type BoxedBoardFeatureSource = Box<dyn BoardFeatureSource>;

impl Clone for BoxedBoardFeatureSource {
    fn clone(&self) -> Self {
        self.clone_boxed()
    }
}

impl BoardFeatureSource for BoxedBoardFeatureSource {
    fn id(&self) -> &str {
        self.as_ref().id()
    }

    fn name(&self) -> &str {
        self.as_ref().name()
    }

    fn clone_boxed(&self) -> BoxedBoardFeatureSource {
        self.as_ref().clone_boxed()
    }

    fn extract_raw(&self, analysis: &PlacementAnalysis) -> u32 {
        self.as_ref().extract_raw(analysis)
    }
}

/// Number of holes (empty cells with at least one occupied cell above them).
///
/// A hole is an empty cell that has at least one occupied cell directly above it in the same column.
/// This metric captures trapped empty spaces that are difficult or impossible to fill without
/// clearing lines first.
///
/// # Raw measurement
///
/// For each column, scan from top to bottom:
/// - Track whether we've encountered at least one occupied cell
/// - Once an occupied cell is found, count all subsequent empty cells as holes
/// - `raw = total count of holes across all columns`
#[derive(Debug, Clone)]
pub struct NumHoles;

impl BoardFeatureSource for NumHoles {
    fn id(&self) -> &'static str {
        "num_holes"
    }
    fn name(&self) -> &'static str {
        "Number of Holes"
    }
    fn clone_boxed(&self) -> BoxedBoardFeatureSource {
        Box::new(self.clone())
    }
    fn extract_raw(&self, analysis: &PlacementAnalysis) -> u32 {
        analysis.board_analysis().num_holes().into()
    }
}

/// Cumulative hole depth (weighted by the number of blocks above each hole).
///
/// This metric measures how deeply holes are buried in the stack. Unlike [`NumHoles`] which counts
/// holes uniformly, this weights each hole by its depth (number of cells above it, including both
/// occupied and empty cells). Deeply buried holes are much more costly to clear.
///
/// # Raw measurement
///
/// For each column, scan top-down tracking a `depth` counter:
/// - When an occupied cell is encountered, increment `depth`
/// - When an empty cell is encountered after at least one occupied cell (`depth > 0`):
///   - Add the current `depth` value to the cumulative sum
///   - Increment `depth` (the hole itself adds to depth for cells below)
/// - `raw = Σ(depth at each hole)` across all columns
#[derive(Debug, Clone)]
pub struct SumOfHoleDepth;

impl BoardFeatureSource for SumOfHoleDepth {
    fn id(&self) -> &'static str {
        "sum_of_hole_depth"
    }
    fn name(&self) -> &'static str {
        "Sum of Hole Depth"
    }
    fn clone_boxed(&self) -> BoxedBoardFeatureSource {
        Box::new(self.clone())
    }
    fn extract_raw(&self, analysis: &PlacementAnalysis) -> u32 {
        analysis.board_analysis().sum_of_hole_depth()
    }
}

/// Maximum column height across the board.
///
/// Measures the height of the tallest column, which directly relates to top-out risk
/// and placement flexibility.
///
/// # Raw measurement
///
/// - `raw = max(column_heights)`: the height of the tallest column (0-20 for standard board)
#[derive(Debug, Clone)]
pub struct MaxHeight;

impl BoardFeatureSource for MaxHeight {
    fn id(&self) -> &'static str {
        "max_height"
    }
    fn name(&self) -> &'static str {
        "Max Height"
    }
    fn clone_boxed(&self) -> BoxedBoardFeatureSource {
        Box::new(self.clone())
    }
    fn extract_raw(&self, analysis: &PlacementAnalysis) -> u32 {
        analysis.board_analysis().max_height().into()
    }
}

/// Maximum height among the center 4 columns (columns 3-6).
///
/// The center columns are strategically critical because they are the most difficult to clear
/// when filled, most pieces naturally gravitate toward the center, and high center columns
/// severely restrict piece placement options.
///
/// # Raw measurement
///
/// - `raw = max(column_heights[3..=6])`: tallest column among the center 4 columns
#[derive(Debug, Clone)]
pub struct CenterColumnMaxHeight;

impl BoardFeatureSource for CenterColumnMaxHeight {
    fn id(&self) -> &'static str {
        "center_column_max_height"
    }
    fn name(&self) -> &'static str {
        "Center Column Max Height"
    }
    fn clone_boxed(&self) -> BoxedBoardFeatureSource {
        Box::new(self.clone())
    }
    fn extract_raw(&self, analysis: &PlacementAnalysis) -> u32 {
        analysis.board_analysis().center_column_max_height().into()
    }
}

/// Sum of all column heights (global stacking pressure).
///
/// Measures cumulative board pressure by summing heights across all columns. This reflects
/// the total "weight" of the board state and overall stacking pressure, complementing
/// localized height metrics.
///
/// # Raw measurement
///
/// - `raw = Σ(column_heights)` across all 10 columns
#[derive(Debug, Clone)]
pub struct TotalHeight;

impl BoardFeatureSource for TotalHeight {
    fn id(&self) -> &'static str {
        "total_height"
    }
    fn name(&self) -> &'static str {
        "Total Height"
    }
    fn clone_boxed(&self) -> BoxedBoardFeatureSource {
        Box::new(self.clone())
    }
    fn extract_raw(&self, analysis: &PlacementAnalysis) -> u32 {
        analysis.board_analysis().total_height().into()
    }
}

/// Horizontal fragmentation (occupancy changes between adjacent cells within rows).
///
/// Counts transitions where adjacent cells differ in occupancy (empty ↔ filled) within each row.
/// High transition counts indicate horizontally fragmented structures with narrow gaps and broken rows.
///
/// # Raw measurement
///
/// - For each row, scan left to right within the playable area
/// - Count transitions where adjacent cells differ in occupancy
/// - Board walls are intentionally ignored to preserve left-right symmetry
/// - `raw = total transitions across all rows`
///
/// Note: Unlike typical implementations that treat walls as filled cells, this ignores walls
/// to avoid artificial bias toward center placement.
#[derive(Debug, Clone)]
pub struct RowTransitions;

impl BoardFeatureSource for RowTransitions {
    fn id(&self) -> &'static str {
        "row_transitions"
    }
    fn name(&self) -> &'static str {
        "Row Transitions"
    }
    fn clone_boxed(&self) -> BoxedBoardFeatureSource {
        Box::new(self.clone())
    }
    fn extract_raw(&self, analysis: &PlacementAnalysis) -> u32 {
        analysis.board_analysis().row_transitions()
    }
}

/// Vertical fragmentation (occupancy changes between cells within columns).
///
/// Counts transitions where adjacent cells differ in occupancy (empty ↔ filled) when scanning
/// top to bottom within each column. High transition counts indicate vertical fragmentation,
/// internal splits, and covered holes.
///
/// # Raw measurement
///
/// - For each column, scan from top to bottom within the playable area
/// - Count transitions where adjacent cells differ in occupancy
/// - `raw = total transitions across all columns`
#[derive(Debug, Clone)]
pub struct ColumnTransitions;

impl BoardFeatureSource for ColumnTransitions {
    fn id(&self) -> &'static str {
        "column_transitions"
    }
    fn name(&self) -> &'static str {
        "Column Transitions"
    }
    fn clone_boxed(&self) -> BoxedBoardFeatureSource {
        Box::new(self.clone())
    }
    fn extract_raw(&self, analysis: &PlacementAnalysis) -> u32 {
        analysis.board_analysis().column_transitions()
    }
}

/// Surface height variation between adjacent columns (first-order differences).
///
/// Measures the absolute height differences between adjacent columns, capturing overall
/// step-like surface patterns and non-flat surfaces that complicate piece placement.
///
/// # Raw measurement
///
/// - For each pair of adjacent columns, compute `|height_right - height_left|`
/// - Sum across all adjacent pairs
/// - `raw = Σ|height differences|`
#[derive(Debug, Clone)]
pub struct SurfaceBumpiness;

impl BoardFeatureSource for SurfaceBumpiness {
    fn id(&self) -> &'static str {
        "surface_bumpiness"
    }
    fn name(&self) -> &'static str {
        "Surface Bumpiness"
    }
    fn clone_boxed(&self) -> BoxedBoardFeatureSource {
        Box::new(self.clone())
    }
    fn extract_raw(&self, analysis: &PlacementAnalysis) -> u32 {
        analysis.board_analysis().surface_bumpiness()
    }
}

/// Local surface curvature (second-order height differences, discrete Laplacian).
///
/// Measures small-scale surface unevenness using second-order differences (curvature).
/// More sensitive to local irregularities than [`SurfaceBumpiness`] while tolerating gradual slopes.
///
/// # Raw measurement
///
/// - For each triplet of adjacent columns, compute the discrete Laplacian:
///   `|(right - mid) - (mid - left)|`
/// - Sum across all triplets
/// - `raw = Σ|second-order differences|`
#[derive(Debug, Clone)]
pub struct SurfaceRoughness;

impl BoardFeatureSource for SurfaceRoughness {
    fn id(&self) -> &'static str {
        "surface_roughness"
    }
    fn name(&self) -> &'static str {
        "Surface Roughness"
    }
    fn clone_boxed(&self) -> BoxedBoardFeatureSource {
        Box::new(self.clone())
    }
    fn extract_raw(&self, analysis: &PlacementAnalysis) -> u32 {
        analysis.board_analysis().surface_roughness()
    }
}

/// Cumulative well depth across columns (measures vertical well commitment).
///
/// A well is a single-column depression flanked by higher columns. This metric measures
/// the total depth of all wells, indicating over-commitment to vertical structures.
/// Shallow wells (depth ≤ 1) are considered safe for controlled play.
///
/// # Raw measurement
///
/// - For each column, calculate well depth (how much lower it is than neighbors)
/// - Only wells deeper than 1 are considered risky (threshold for safe construction)
/// - `raw = Σ(depth - 1)` across columns where `depth > 1`
#[derive(Debug, Clone)]
pub struct SumOfWellDepth;

impl BoardFeatureSource for SumOfWellDepth {
    fn id(&self) -> &'static str {
        "sum_of_well_depth"
    }
    fn name(&self) -> &'static str {
        "Sum of Well Depth"
    }
    fn clone_boxed(&self) -> BoxedBoardFeatureSource {
        Box::new(self.clone())
    }
    fn extract_raw(&self, analysis: &PlacementAnalysis) -> u32 {
        analysis.board_analysis().sum_of_deep_well_depth()
    }
}

/// Number of lines cleared by a placement.
///
/// Counts how many lines were cleared as a result of the current piece placement.
/// This is a per-placement action metric, not a cumulative board state metric.
///
/// # Raw measurement
///
/// - `raw = number of lines cleared` (0-4)
/// - Direct count from the placement result
#[derive(Debug, Clone)]
pub struct NumClearedLines;

impl BoardFeatureSource for NumClearedLines {
    fn id(&self) -> &'static str {
        "num_cleared_lines"
    }
    fn name(&self) -> &'static str {
        "Number of Cleared Lines"
    }
    fn clone_boxed(&self) -> BoxedBoardFeatureSource {
        Box::new(self.clone())
    }
    fn extract_raw(&self, analysis: &PlacementAnalysis) -> u32 {
        u32::try_from(analysis.cleared_lines()).unwrap()
    }
}

/// Depth of I-piece well at the board edges (leftmost or rightmost column).
///
/// Measures the quality of I-piece well setup at board edges. A well depth around 4
/// is optimal for efficient tetris (4-line clear) execution. Too shallow or too deep
/// wells are less useful.
///
/// # Raw measurement
///
/// - Check leftmost (column 0) and rightmost (column 9) columns
/// - Calculate well depth for each edge column
/// - `raw = max(left_well_depth, right_well_depth)`
#[derive(Debug, Clone)]
pub struct EdgeIWellDepth;

impl BoardFeatureSource for EdgeIWellDepth {
    fn id(&self) -> &'static str {
        "edge_i_well_depth"
    }
    fn name(&self) -> &'static str {
        "Edge I Well Depth"
    }
    fn clone_boxed(&self) -> BoxedBoardFeatureSource {
        Box::new(self.clone())
    }
    fn extract_raw(&self, analysis: &PlacementAnalysis) -> u32 {
        analysis.board_analysis().edge_i_well_depth().into()
    }
}
