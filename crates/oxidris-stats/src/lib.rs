//! Statistical analysis utilities for the Oxidris project.
//!
//! This crate provides a collection of statistical tools including:
//!
//! - **Descriptive statistics**: Calculate mean, median, variance, standard deviation, etc.
//! - **Percentiles**: Compute and store percentile values for datasets
//! - **Histogram generation**: Create frequency distributions with percentile-based binning
//! - **Comprehensive statistics**: Combined descriptive statistics, percentiles, and histograms
//! - **Survival analysis**: Kaplan-Meier estimator for time-to-event data with censoring
//!
//! # Modules
//!
//! - [`descriptive`]: Descriptive statistics for summarizing datasets
//! - [`percentiles`]: Percentile computation and storage
//! - [`histogram`]: Histogram construction for visualizing data distributions
//! - [`comprehensive`]: Comprehensive statistical analysis combining multiple measures
//! - [`survival`]: Kaplan-Meier survival curves for analyzing time-to-event data
//!
//! # Examples
//!
//! ## Computing descriptive statistics
//!
//! ```
//! use oxidris_stats::descriptive::DescriptiveStats;
//!
//! let values = [1.0, 2.0, 3.0, 4.0, 5.0];
//! let stats = DescriptiveStats::new(values).unwrap();
//! assert_eq!(stats.mean, 3.0);
//! ```
//!
//! ## Computing percentiles
//!
//! ```
//! use oxidris_stats::percentiles::Percentiles;
//!
//! let values = [1.0, 2.0, 3.0, 4.0, 5.0];
//! let percentiles = Percentiles::new(values, &[25.0, 50.0, 75.0]);
//! assert_eq!(percentiles.get(50.0), Some(3.0));
//! ```
//!
//! ## Computing comprehensive statistics
//!
//! ```
//! use oxidris_stats::comprehensive::ComprehensiveStats;
//!
//! let values = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
//! let stats = ComprehensiveStats::new(values, &[25.0, 50.0, 75.0], 5, None, None, None).unwrap();
//! assert_eq!(stats.percentiles.get(50.0), Some(6.0));
//! ```
//!
//! ## Creating a histogram
//!
//! ```
//! use oxidris_stats::histogram::Histogram;
//!
//! let values = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
//! let histogram = Histogram::new(values, 5, None, None, None);
//! ```
//!
//! ## Analyzing survival data
//!
//! ```
//! use oxidris_stats::survival::KaplanMeierCurve;
//!
//! // Data: (time, is_censored)
//! let data = vec![
//!     (10, false), // Event occurred at time 10
//!     (20, true),  // Censored at time 20
//!     (30, false), // Event occurred at time 30
//! ];
//! let curve = KaplanMeierCurve::from_data(data);
//! ```

pub mod comprehensive;
pub mod descriptive;
pub mod histogram;
pub mod percentiles;
pub mod survival;
