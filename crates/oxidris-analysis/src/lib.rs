//! Offline data analysis and feature engineering for Tetris AI training
//!
//! This crate provides tools for analyzing gameplay sessions, computing feature
//! statistics, and building runtime-parameterized features for the evaluator system.
//!
//! # Overview
//!
//! The analysis system supports three main workflows:
//!
//! ## Feature Construction Workflow
//!
//! Build data-driven normalized features from training data:
//!
//! 1. **Load Session Data** ([`session::SessionCollection`]): Load recorded gameplay sessions
//! 2. **Extract Raw Samples** ([`sample::RawBoardSample`]): Extract raw feature values
//! 3. **Compute Statistics** ([`statistics::RawFeatureStatistics`]): Analyze distributions
//! 4. **Generate Parameters** ([`normalization::BoardFeatureNormalizationParamCollection`]):
//!    Compute normalization ranges from percentiles
//! 5. **Build Features** ([`feature_builder::FeatureBuilder`]): Construct runtime-parameterized features
//!
//! ## Statistical Analysis Workflow
//!
//! Analyze feature distributions and select representative samples:
//!
//! 1. **Sample Collection** ([`sample::BoardSample`]): Extract and compute feature values
//! 2. **Statistical Analysis** ([`statistics::BoardFeatureStatistics`]): Compute distributions,
//!    percentiles, and histograms
//! 3. **Efficient Indexing** ([`index::BoardIndex`]): Build sorted indices for fast queries
//!
//! ## Survival Analysis Workflow
//!
//! Handle right-censored training data using Kaplan-Meier estimation:
//!
//! 1. **Load Session Data** ([`session::SessionCollection`]): Load gameplay sessions
//! 2. **Group by Feature** ([`survival::SurvivalStatsMap`]): Collect survival times per feature value
//! 3. **Kaplan-Meier Analysis**: Compute unbiased survival curves from censored data
//!
//! # Primary Use Cases
//!
//! - **Feature Engineering**: Build data-driven normalized features for AI training
//! - **Distribution Analysis**: Understand feature characteristics and relationships
//! - **Survival Analysis**: Handle right-censored training data with KM estimation
//! - **Data Quality**: Outlier detection and validation
//! - **Visualization**: Sample selection for training data inspection
//!
//! # Examples
//!
//! ## Building Runtime-Parameterized Features
//!
//! The most common use case - construct features with normalization parameters
//! computed from actual gameplay data:
//!
//! ```no_run
//! use oxidris_analysis::{
//!     feature_builder::FeatureBuilder, normalization::BoardFeatureNormalizationParamCollection,
//!     sample::RawBoardSample, session::SessionData, statistics::RawFeatureStatistics,
//! };
//! use oxidris_evaluator::board_feature;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//!
//! // 1. Load training data (in practice, use serde_json or CLI utility)
//! let sessions: Vec<SessionData> = vec![]; // Load from file
//!
//! // 2. Get feature sources
//! let sources = board_feature::all_board_feature_sources();
//!
//! // 3. Extract raw samples
//! let raw_samples = RawBoardSample::from_sessions(&sources, &sessions);
//!
//! // 4. Compute statistics
//! let raw_stats = RawFeatureStatistics::from_samples(&sources, &raw_samples);
//!
//! // 5. Generate normalization parameters
//! let norm_params = BoardFeatureNormalizationParamCollection::from_stats(&sources, &raw_stats);
//!
//! // 6. Build features with runtime parameters
//! let builder = FeatureBuilder::new(norm_params);
//! let features = builder.build_all_features()?;
//!
//! println!("Built {} features", features.len());
//! # Ok(())
//! # }
//! ```
//!
//! ## Statistical Analysis
//!
//! Analyze feature distributions for understanding or visualization:
//!
//! ```no_run
//! use oxidris_analysis::{
//!     index::BoardIndex, sample::BoardSample, session::SessionData,
//!     statistics::BoardFeatureStatistics,
//! };
//! # use oxidris_evaluator::board_feature::BoxedBoardFeature;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let features: Vec<BoxedBoardFeature> = vec![];
//!
//! // Load sessions and extract samples (in practice, load from file)
//! let sessions: Vec<SessionData> = vec![]; // Load from file
//! let samples = BoardSample::from_sessions(&features, &sessions);
//!
//! // Compute statistics
//! let stats = BoardFeatureStatistics::from_samples(&features, &samples);
//! println!("Feature 0 raw mean: {}", stats[0].raw.stats.mean);
//! println!(
//!     "Feature 0 normalized P95: {}",
//!     stats[0].normalized.percentiles.get(95.0).unwrap()
//! );
//!
//! // Build index for queries
//! let index = BoardIndex::from_samples(&features, &samples);
//! let top_10 = index.get_boards_in_rank_range(0, 0, 10);
//! println!("Top 10 boards for feature 0: {:?}", top_10);
//! # Ok(())
//! # }
//! ```
//!
//! ## Survival Analysis
//!
//! Compute survival statistics with Kaplan-Meier estimation for censored data:
//!
//! ```no_run
//! use oxidris_analysis::{session::SessionData, survival::SurvivalStatsMap};
//! use oxidris_evaluator::{
//!     board_feature::{BoardFeatureSource, source::NumHoles},
//!     placement_analysis::PlacementAnalysis,
//! };
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//!
//! // Load sessions (in practice, load from file)
//! let sessions: Vec<SessionData> = vec![]; // Load from file
//! let source = NumHoles;
//!
//! // Group survival times by feature value
//! let stats = SurvivalStatsMap::collect_by_group(&sessions, |_session, board| {
//!     let analysis = PlacementAnalysis::from_board(&board.before_placement, board.placement);
//!     source.extract_raw(&analysis)
//! });
//!
//! // Access KM median survival for each value
//! for (value, stat) in &stats.map {
//!     if let Some(km_median) = stat.median_km {
//!         println!(
//!             "Holes={}: KM median survival = {:.1} turns",
//!             value, km_median
//!         );
//!     }
//! }
//! # Ok(())
//! # }
//! ```

pub mod feature_builder;
pub mod index;
pub mod normalization;
pub mod sample;
pub mod session;
pub mod statistics;
pub mod survival;
