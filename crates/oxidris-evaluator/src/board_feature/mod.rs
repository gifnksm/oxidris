//! Board evaluation features for Tetris.
//!
//! This module provides the feature system for evaluating Tetris board states. Each feature
//! produces a normalized score in \[0.0, 1.0\], where higher is always better (negative features
//! are inverted via [`FeatureSignal::Negative`]).
//!
//! # Feature Architecture
//!
//! The feature system separates concerns into two layers:
//!
//! ## Feature Sources ([`source`] module)
//!
//! Feature sources extract raw measurements from board states and are categorized by what they measure:
//!
//! **Survival Features** - Directly affect game termination:
//! - [`source::NumHoles`] - Count of covered empty cells
//! - [`source::SumOfHoleDepth`] - Cumulative depth of buried holes
//! - [`source::MaxHeight`] - Tallest column height
//! - [`source::CenterColumnMaxHeight`] - Tallest among center 4 columns
//! - [`source::TotalHeight`] - Sum of all column heights
//!
//! **Structure Features** - Affect placement flexibility:
//! - [`source::SurfaceBumpiness`] - Height variation between adjacent columns (first-order)
//! - [`source::SurfaceRoughness`] - Local surface curvature (second-order)
//! - [`source::RowTransitions`] - Horizontal fragmentation
//! - [`source::ColumnTransitions`] - Vertical fragmentation
//! - [`source::SumOfWellDepth`] - Cumulative well depth
//!
//! **Score Features** - Directly contribute to game score:
//! - [`source::NumClearedLines`] - Lines cleared by placement
//! - [`source::EdgeIWellDepth`] - I-piece well quality at board edges
//!
//! ## Feature Types
//!
//! Feature sources are wrapped by feature types that provide transformation and normalization:
//!
//! - [`transform::RawTransform`] - Linear transformation with percentile-based normalization
//! - [`transform::LineClearBonus`] - Discrete bonus mapping for line clears
//! - [`transform::IWellReward`] - Triangular reward for optimal I-well depth
//!
//! # Feature Processing Pipeline
//!
//! Each feature processes board states through three steps:
//!
//! 1. **Extract Raw** - [`BoardFeatureSource::extract_raw()`] extracts the raw measurement
//! 2. **Transform** - [`BoardFeature::transform()`] converts raw value to meaningful representation
//! 3. **Normalize** - [`BoardFeature::normalize()`] scales to \[0.0, 1.0\]
//!
//! See [`BoardFeature::compute_feature_value()`] for the complete pipeline.
//!
//! # Trait Overview
//!
//! - [`BoardFeatureSource`] - Extracts raw values from board states (implemented by source types)
//! - [`BoardFeature`] - Complete feature computation including transform and normalize (implemented by feature types)

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{board_feature::transform::FeatureProcessing, placement_analysis::PlacementAnalysis};

pub use self::source::{BoardFeatureSource, BoxedBoardFeatureSource};

pub mod source;
pub mod transform;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatureSignal {
    Positive,
    Negative,
}

#[derive(Debug, Clone, Copy)]
pub struct BoardFeatureValue {
    pub raw: u32,
    pub transformed: f32,
    pub normalized: f32,
}

pub trait BoardFeature: fmt::Debug + Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn feature_source(&self) -> &dyn BoardFeatureSource;
    fn feature_processing(&self) -> FeatureProcessing;
    fn clone_boxed(&self) -> BoxedBoardFeature;

    #[must_use]
    fn extract_raw(&self, analysis: &PlacementAnalysis) -> u32;

    #[must_use]
    fn transform(&self, raw: u32) -> f32;

    #[must_use]
    fn normalize(&self, transformed: f32) -> f32;

    #[must_use]
    fn compute_feature_value(&self, analysis: &PlacementAnalysis) -> BoardFeatureValue {
        let raw = self.extract_raw(analysis);
        let transformed = self.transform(raw);
        let normalized = self.normalize(transformed);
        BoardFeatureValue {
            raw,
            transformed,
            normalized,
        }
    }
}

pub type BoxedBoardFeature = Box<dyn BoardFeature>;

impl Clone for BoxedBoardFeature {
    fn clone(&self) -> Self {
        self.clone_boxed()
    }
}

impl BoardFeature for BoxedBoardFeature {
    fn id(&self) -> &str {
        self.as_ref().id()
    }

    fn name(&self) -> &str {
        self.as_ref().name()
    }

    fn feature_source(&self) -> &dyn BoardFeatureSource {
        self.as_ref().feature_source()
    }

    fn feature_processing(&self) -> FeatureProcessing {
        self.as_ref().feature_processing()
    }

    fn clone_boxed(&self) -> BoxedBoardFeature {
        self.as_ref().clone_boxed()
    }

    fn extract_raw(&self, analysis: &PlacementAnalysis) -> u32 {
        self.as_ref().extract_raw(analysis)
    }

    fn transform(&self, raw: u32) -> f32 {
        self.as_ref().transform(raw)
    }

    fn normalize(&self, transformed: f32) -> f32 {
        self.as_ref().normalize(transformed)
    }
}
