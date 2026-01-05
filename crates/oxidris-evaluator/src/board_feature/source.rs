//! Internal feature source types for extracting raw values from board states.
//!
//! These types implement [`BoardFeatureSource`] and are wrapped by feature constants
//! like [`LINEAR_HOLES_PENALTY`](super::LINEAR_HOLES_PENALTY) to provide complete feature computation.

use crate::{board_feature::BoardFeatureSource, placement_analysis::PlacementAnalysis};

#[derive(Debug, Clone)]
pub struct NumHoles;

impl BoardFeatureSource for NumHoles {
    fn id(&self) -> &'static str {
        "num_holes"
    }
    fn name(&self) -> &'static str {
        "Number of Holes"
    }
    fn clone_boxed(&self) -> super::BoxedBoardFeatureSource {
        Box::new(self.clone())
    }
    fn extract_raw(&self, analysis: &PlacementAnalysis) -> u32 {
        analysis.board_analysis().num_holes().into()
    }
}

#[derive(Debug, Clone)]
pub struct SumOfHoleDepth;

impl BoardFeatureSource for SumOfHoleDepth {
    fn id(&self) -> &'static str {
        "sum_of_hole_depth"
    }
    fn name(&self) -> &'static str {
        "Sum of Hole Depth"
    }
    fn clone_boxed(&self) -> super::BoxedBoardFeatureSource {
        Box::new(self.clone())
    }
    fn extract_raw(&self, analysis: &PlacementAnalysis) -> u32 {
        analysis.board_analysis().sum_of_hole_depth()
    }
}

#[derive(Debug, Clone)]
pub struct MaxHeight;

impl BoardFeatureSource for MaxHeight {
    fn id(&self) -> &'static str {
        "max_height"
    }
    fn name(&self) -> &'static str {
        "Max Height"
    }
    fn clone_boxed(&self) -> super::BoxedBoardFeatureSource {
        Box::new(self.clone())
    }
    fn extract_raw(&self, analysis: &PlacementAnalysis) -> u32 {
        analysis.board_analysis().max_height().into()
    }
}

#[derive(Debug, Clone)]
pub struct CenterColumnMaxHeight;

impl BoardFeatureSource for CenterColumnMaxHeight {
    fn id(&self) -> &'static str {
        "center_column_max_height"
    }
    fn name(&self) -> &'static str {
        "Center Column Max Height"
    }
    fn clone_boxed(&self) -> super::BoxedBoardFeatureSource {
        Box::new(self.clone())
    }
    fn extract_raw(&self, analysis: &PlacementAnalysis) -> u32 {
        analysis.board_analysis().center_column_max_height().into()
    }
}

#[derive(Debug, Clone)]
pub struct TotalHeight;

impl BoardFeatureSource for TotalHeight {
    fn id(&self) -> &'static str {
        "total_height"
    }
    fn name(&self) -> &'static str {
        "Total Height"
    }
    fn clone_boxed(&self) -> super::BoxedBoardFeatureSource {
        Box::new(self.clone())
    }
    fn extract_raw(&self, analysis: &PlacementAnalysis) -> u32 {
        analysis.board_analysis().total_height().into()
    }
}

#[derive(Debug, Clone)]
pub struct RowTransitions;

impl BoardFeatureSource for RowTransitions {
    fn id(&self) -> &'static str {
        "row_transitions"
    }
    fn name(&self) -> &'static str {
        "Row Transitions"
    }
    fn clone_boxed(&self) -> super::BoxedBoardFeatureSource {
        Box::new(self.clone())
    }
    fn extract_raw(&self, analysis: &PlacementAnalysis) -> u32 {
        analysis.board_analysis().row_transitions()
    }
}

#[derive(Debug, Clone)]
pub struct ColumnTransitions;

impl BoardFeatureSource for ColumnTransitions {
    fn id(&self) -> &'static str {
        "column_transitions"
    }
    fn name(&self) -> &'static str {
        "Column Transitions"
    }
    fn clone_boxed(&self) -> super::BoxedBoardFeatureSource {
        Box::new(self.clone())
    }
    fn extract_raw(&self, analysis: &PlacementAnalysis) -> u32 {
        analysis.board_analysis().column_transitions()
    }
}

#[derive(Debug, Clone)]
pub struct SurfaceBumpiness;

impl BoardFeatureSource for SurfaceBumpiness {
    fn id(&self) -> &'static str {
        "surface_bumpiness"
    }
    fn name(&self) -> &'static str {
        "Surface Bumpiness"
    }
    fn clone_boxed(&self) -> super::BoxedBoardFeatureSource {
        Box::new(self.clone())
    }
    fn extract_raw(&self, analysis: &PlacementAnalysis) -> u32 {
        analysis.board_analysis().surface_bumpiness()
    }
}

#[derive(Debug, Clone)]
pub struct SurfaceRoughness;

impl BoardFeatureSource for SurfaceRoughness {
    fn id(&self) -> &'static str {
        "surface_roughness"
    }
    fn name(&self) -> &'static str {
        "Surface Roughness"
    }
    fn clone_boxed(&self) -> super::BoxedBoardFeatureSource {
        Box::new(self.clone())
    }
    fn extract_raw(&self, analysis: &PlacementAnalysis) -> u32 {
        analysis.board_analysis().surface_roughness()
    }
}

#[derive(Debug, Clone)]
pub struct SumOfWellDepth;

impl BoardFeatureSource for SumOfWellDepth {
    fn id(&self) -> &'static str {
        "sum_of_well_depth"
    }
    fn name(&self) -> &'static str {
        "Sum of Well Depth"
    }
    fn clone_boxed(&self) -> super::BoxedBoardFeatureSource {
        Box::new(self.clone())
    }
    fn extract_raw(&self, analysis: &PlacementAnalysis) -> u32 {
        analysis.board_analysis().sum_of_deep_well_depth()
    }
}

#[derive(Debug, Clone)]
pub struct NumClearedLines;

impl BoardFeatureSource for NumClearedLines {
    fn id(&self) -> &'static str {
        "num_cleared_lines"
    }
    fn name(&self) -> &'static str {
        "Number of Cleared Lines"
    }
    fn clone_boxed(&self) -> super::BoxedBoardFeatureSource {
        Box::new(self.clone())
    }
    fn extract_raw(&self, analysis: &PlacementAnalysis) -> u32 {
        u32::try_from(analysis.cleared_lines()).unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct EdgeIWellDepth;

impl BoardFeatureSource for EdgeIWellDepth {
    fn id(&self) -> &'static str {
        "edge_i_well_depth"
    }
    fn name(&self) -> &'static str {
        "Edge I Well Depth"
    }
    fn clone_boxed(&self) -> super::BoxedBoardFeatureSource {
        Box::new(self.clone())
    }
    fn extract_raw(&self, analysis: &PlacementAnalysis) -> u32 {
        analysis.board_analysis().edge_i_well_depth().into()
    }
}
