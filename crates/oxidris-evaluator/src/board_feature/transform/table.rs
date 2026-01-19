//! Table-based feature transformation using lookup tables
//!
//! This module provides [`TableTransform`], a feature transformation that maps
//! raw feature values to pre-computed values via a lookup table. This is primarily
//! used for Kaplan-Meier survival-based transformations where each feature value
//! maps to its median survival time.
//!
//! # Overview
//!
//! Table transforms work in three steps:
//!
//! 1. **Extract**: Get raw feature value from board state
//! 2. **Transform**: Look up pre-computed value in table (e.g., KM median survival)
//! 3. **Normalize**: Scale to [0, 1] range using pre-computed min/max
//!
//! # Example
//!
//! ```no_run
//! use oxidris_evaluator::board_feature::{
//!     source::NumHoles,
//!     transform::{TableTransform, TableTransformParam},
//! };
//!
//! // Suppose holes=0..5 have median survivals [100.0, 80.0, 60.0, 40.0, 20.0]
//! let param = TableTransformParam::new(
//!     0,                                   // feature_min_value (holes start at 0)
//!     20.0,                                // normalize_min (worst survival)
//!     100.0,                               // normalize_max (best survival)
//!     vec![100.0, 80.0, 60.0, 40.0, 20.0], // median survival for each hole count
//! );
//!
//! let feature = TableTransform::new(
//!     "num_holes_table_km".to_string(),
//!     "Number of Holes (Table KM)".to_string(),
//!     NumHoles,
//!     param,
//! );
//! ```

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{
    board_feature::{
        BoardFeature, BoardFeatureSource, BoxedBoardFeature, transform::FeatureProcessing,
    },
    placement_analysis::PlacementAnalysis,
};

/// Table-based feature transformation using pre-computed lookup tables
///
/// This transformation maps raw feature values to pre-computed transformed values
/// via a lookup table. The primary use case is Kaplan-Meier survival analysis,
/// where each feature value maps to its median survival time.
///
/// # Type Parameters
///
/// * `S` - The underlying feature source type (e.g., [`NumHoles`](crate::board_feature::source::NumHoles))
///
/// # Transform Behavior
///
/// Given a raw feature value `v`:
///
/// 1. Clamp to table range: `v_clamped = clamp(v, min_value, max_value)`
/// 2. Look up in table: `transformed = table[v_clamped - min_value]`
/// 3. Normalize: `normalized = (transformed - norm_min) / (norm_max - norm_min)`
///
/// # Example
///
/// ```no_run
/// use oxidris_evaluator::board_feature::{
///     source::MaxHeight,
///     transform::{TableTransform, TableTransformParam},
/// };
///
/// // Height 0-20 with decreasing survival times
/// let survivals = (0..=20).rev().map(|h| h as f32 * 10.0).collect();
/// let param = TableTransformParam::new(0, 0.0, 200.0, survivals);
///
/// let feature = TableTransform::new(
///     "max_height_table_km".to_string(),
///     "Max Height (Table KM)".to_string(),
///     MaxHeight,
///     param,
/// );
/// ```
#[derive(Debug, Clone)]
pub struct TableTransform<S> {
    /// Unique feature identifier (e.g., "`num_holes_table_km`")
    id: String,
    /// Human-readable feature name
    name: String,
    /// Underlying feature source that extracts raw values
    source: S,
    /// Table transformation parameters (table, ranges)
    param: TableTransformParam,
}

/// Parameters for table-based feature transformation
///
/// Contains the lookup table and normalization ranges for transforming
/// raw feature values into normalized scores.
///
/// # Invariants
///
/// * `table` must not be empty
/// * Table covers range `[feature_min_value, feature_min_value + table.len() - 1]`
/// * Raw values outside this range are clamped to table boundaries
///
/// # Example
///
/// ```no_run
/// use oxidris_evaluator::board_feature::transform::TableTransformParam;
///
/// // Map holes 0-4 to survival times [100, 75, 50, 25, 10]
/// let param = TableTransformParam::new(
///     0,     // min value (0 holes)
///     10.0,  // normalize min (worst survival)
///     100.0, // normalize max (best survival)
///     vec![100.0, 75.0, 50.0, 25.0, 10.0],
/// );
/// ```
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TableTransformParam {
    /// Minimum raw feature value (table start index)
    feature_min_value: u32,
    /// Minimum transformed value (for normalization)
    normalize_min: f32,
    /// Maximum transformed value (for normalization)
    normalize_max: f32,
    /// Lookup table mapping feature values to transformed values
    table: Vec<f32>,
}

impl<S> TableTransform<S> {
    /// Create a new table-based feature transformation
    ///
    /// # Arguments
    ///
    /// * `id` - Unique feature identifier
    /// * `name` - Human-readable feature name
    /// * `source` - Underlying feature source
    /// * `param` - Table transformation parameters
    ///
    /// # Example
    ///
    /// ```no_run
    /// use oxidris_evaluator::board_feature::{
    ///     source::NumHoles,
    ///     transform::{TableTransform, TableTransformParam},
    /// };
    ///
    /// let param = TableTransformParam::new(0, 10.0, 100.0, vec![100.0, 50.0, 10.0]);
    /// let feature = TableTransform::new(
    ///     "num_holes_table_km".to_string(),
    ///     "Holes (Table KM)".to_string(),
    ///     NumHoles,
    ///     param,
    /// );
    /// ```
    #[must_use]
    pub fn new(id: String, name: String, source: S, param: TableTransformParam) -> Self {
        Self {
            id,
            name,
            source,
            param,
        }
    }
}

impl TableTransformParam {
    /// Create new table transformation parameters
    ///
    /// # Arguments
    ///
    /// * `feature_min_value` - Minimum raw feature value (table start)
    /// * `normalize_min` - Minimum transformed value for normalization
    /// * `normalize_max` - Maximum transformed value for normalization
    /// * `table` - Lookup table of transformed values
    ///
    /// # Panics
    ///
    /// Panics if `table` is empty.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use oxidris_evaluator::board_feature::transform::TableTransformParam;
    ///
    /// // Holes 2-6 with survival times [80, 60, 40, 20, 10]
    /// let param = TableTransformParam::new(
    ///     2,    // starts at 2 holes
    ///     10.0, // worst survival
    ///     80.0, // best survival
    ///     vec![80.0, 60.0, 40.0, 20.0, 10.0],
    /// );
    /// ```
    #[must_use]
    pub fn new(
        feature_min_value: u32,
        normalize_min: f32,
        normalize_max: f32,
        table: Vec<f32>,
    ) -> Self {
        assert!(!table.is_empty(), "Table must not be empty");
        Self {
            feature_min_value,
            normalize_min,
            normalize_max,
            table,
        }
    }
}

impl<S> BoardFeature for TableTransform<S>
where
    S: BoardFeatureSource + Clone + fmt::Debug + Send + Sync + 'static,
{
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn feature_source(&self) -> &dyn BoardFeatureSource {
        &self.source
    }

    fn feature_processing(&self) -> FeatureProcessing {
        FeatureProcessing::TableTransform(self.param.clone())
    }

    fn clone_boxed(&self) -> BoxedBoardFeature {
        Box::new(self.clone())
    }

    fn extract_raw(&self, analysis: &PlacementAnalysis) -> u32 {
        self.source.extract_raw(analysis)
    }

    fn transform(&self, raw: u32) -> f32 {
        // Table covers [feature_min_value, feature_min_value + len - 1]
        // Safe because: table is non-empty (checked in constructor)
        let table_len = self.param.table.len();
        #[expect(clippy::cast_possible_truncation)]
        let max = self.param.feature_min_value + (table_len - 1) as u32;

        // Clamp raw value to table range
        let clamped = raw.clamp(self.param.feature_min_value, max);

        // Look up in table (safe because clamped is within table bounds)
        let index = (clamped - self.param.feature_min_value) as usize;
        self.param.table[index]
    }

    fn normalize(&self, transformed: f32) -> f32 {
        // Handle edge case where all values are identical (no variance)
        let range = self.param.normalize_max - self.param.normalize_min;
        if range.abs() < f32::EPSILON {
            // All values are the same - return middle of [0, 1] range
            return 0.5;
        }

        ((transformed - self.param.normalize_min) / range).clamp(0.0, 1.0)
    }
}
