//! Board feature analysis
//!
//! This module provides types and functions for statistical analysis of
//! board features in Tetris gameplay sessions.
//!
//! # Overview
//!
//! The analysis pipeline consists of three main steps:
//!
//! 1. **Sample Collection** ([`BoardSample`]): Extract and compute feature values
//!    from recorded game sessions
//! 2. **Statistical Analysis** ([`BoardFeatureStatistics`]): Compute distributions,
//!    percentiles, and histograms for each feature
//! 3. **Efficient Indexing** ([`BoardIndex`]): Build sorted indices for fast
//!    percentile and rank-based queries
//!
//! # Typical Workflow
//!
//! ```no_run
//! use oxidris_cli::analysis::{BoardFeatureStatistics, BoardIndex, BoardSample};
//! # let features = todo!();
//! # let sessions = todo!();
//!
//! // 1. Collect samples from session data
//! let samples = BoardSample::from_sessions(&features, &sessions);
//!
//! // 2. Compute statistics
//! let stats = BoardFeatureStatistics::from_samples(&features, &samples);
//!
//! // 3. Build index for efficient queries
//! let index = BoardIndex::from_samples(&features, &samples);
//!
//! // 4. Analyze results
//! println!("Feature 0 mean: {}", stats[0].raw.mean);
//! let top_boards = index.get_boards_in_rank_range(0, 0, 10);
//! ```
//!
//! # Use Cases
//!
//! - Feature engineering and normalization parameter tuning
//! - Distribution analysis for understanding feature characteristics
//! - Sample selection for training data visualization
//! - Outlier detection and data quality validation

pub use self::{feature_builder::*, index::*, normalization::*, sample::*, statistics::*};

mod feature_builder;
mod index;
mod normalization;
mod sample;
mod statistics;
pub mod survival;
