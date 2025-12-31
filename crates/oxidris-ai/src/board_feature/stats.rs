impl crate::board_feature::HolesPenalty {
    pub const RAW_P01: f32 = 0.0;
    pub const RAW_P05: f32 = 0.0;
    pub const RAW_P25: f32 = 0.0;
    pub const RAW_P50: f32 = 1.0;
    pub const RAW_P75: f32 = 2.0;
    pub const RAW_P95: f32 = 6.0;
    pub const RAW_P99: f32 = 9.0;
    pub const TRANSFORMED_P01: f32 = 0.0;
    pub const TRANSFORMED_P05: f32 = 0.0;
    pub const TRANSFORMED_P25: f32 = 0.0;
    pub const TRANSFORMED_P50: f32 = 1.0;
    pub const TRANSFORMED_P75: f32 = 2.0;
    pub const TRANSFORMED_P95: f32 = 6.0;
    pub const TRANSFORMED_P99: f32 = 9.0;
    pub const NORMALIZED_P01: f32 = 0.0;
    pub const NORMALIZED_P05: f32 = 0.0;
    pub const NORMALIZED_P25: f32 = 0.666_666_6;
    pub const NORMALIZED_P50: f32 = 0.833_333_3;
    pub const NORMALIZED_P75: f32 = 1.0;
    pub const NORMALIZED_P95: f32 = 1.0;
    pub const NORMALIZED_P99: f32 = 1.0;
}

impl crate::board_feature::HoleDepthPenalty {
    pub const RAW_P01: f32 = 0.0;
    pub const RAW_P05: f32 = 0.0;
    pub const RAW_P25: f32 = 0.0;
    pub const RAW_P50: f32 = 2.0;
    pub const RAW_P75: f32 = 9.0;
    pub const RAW_P95: f32 = 32.0;
    pub const RAW_P99: f32 = 60.0;
    pub const TRANSFORMED_P01: f32 = 0.0;
    pub const TRANSFORMED_P05: f32 = 0.0;
    pub const TRANSFORMED_P25: f32 = 0.0;
    pub const TRANSFORMED_P50: f32 = 2.0;
    pub const TRANSFORMED_P75: f32 = 9.0;
    pub const TRANSFORMED_P95: f32 = 32.0;
    pub const TRANSFORMED_P99: f32 = 60.0;
    pub const NORMALIZED_P01: f32 = 0.0;
    pub const NORMALIZED_P05: f32 = 0.0;
    pub const NORMALIZED_P25: f32 = 0.718_75;
    pub const NORMALIZED_P50: f32 = 0.937_5;
    pub const NORMALIZED_P75: f32 = 1.0;
    pub const NORMALIZED_P95: f32 = 1.0;
    pub const NORMALIZED_P99: f32 = 1.0;
}

impl crate::board_feature::RowTransitionsPenalty {
    pub const RAW_P01: f32 = 3.0;
    pub const RAW_P05: f32 = 4.0;
    pub const RAW_P25: f32 = 7.0;
    pub const RAW_P50: f32 = 11.0;
    pub const RAW_P75: f32 = 18.0;
    pub const RAW_P95: f32 = 32.0;
    pub const RAW_P99: f32 = 43.0;
    pub const TRANSFORMED_P01: f32 = 3.0;
    pub const TRANSFORMED_P05: f32 = 4.0;
    pub const TRANSFORMED_P25: f32 = 7.0;
    pub const TRANSFORMED_P50: f32 = 11.0;
    pub const TRANSFORMED_P75: f32 = 18.0;
    pub const TRANSFORMED_P95: f32 = 32.0;
    pub const TRANSFORMED_P99: f32 = 43.0;
    pub const NORMALIZED_P01: f32 = 0.0;
    pub const NORMALIZED_P05: f32 = 0.0;
    pub const NORMALIZED_P25: f32 = 0.5;
    pub const NORMALIZED_P50: f32 = 0.75;
    pub const NORMALIZED_P75: f32 = 0.892_857_13;
    pub const NORMALIZED_P95: f32 = 1.0;
    pub const NORMALIZED_P99: f32 = 1.0;
}

impl crate::board_feature::ColumnTransitionsPenalty {
    pub const RAW_P01: f32 = 3.0;
    pub const RAW_P05: f32 = 6.0;
    pub const RAW_P25: f32 = 9.0;
    pub const RAW_P50: f32 = 11.0;
    pub const RAW_P75: f32 = 13.0;
    pub const RAW_P95: f32 = 19.0;
    pub const RAW_P99: f32 = 25.0;
    pub const TRANSFORMED_P01: f32 = 3.0;
    pub const TRANSFORMED_P05: f32 = 6.0;
    pub const TRANSFORMED_P25: f32 = 9.0;
    pub const TRANSFORMED_P50: f32 = 11.0;
    pub const TRANSFORMED_P75: f32 = 13.0;
    pub const TRANSFORMED_P95: f32 = 19.0;
    pub const TRANSFORMED_P99: f32 = 25.0;
    pub const NORMALIZED_P01: f32 = 0.0;
    pub const NORMALIZED_P05: f32 = 0.0;
    pub const NORMALIZED_P25: f32 = 0.461_538_43;
    pub const NORMALIZED_P50: f32 = 0.615_384_6;
    pub const NORMALIZED_P75: f32 = 0.769_230_8;
    pub const NORMALIZED_P95: f32 = 1.0;
    pub const NORMALIZED_P99: f32 = 1.0;
}

impl crate::board_feature::SurfaceBumpinessPenalty {
    pub const RAW_P01: f32 = 2.0;
    pub const RAW_P05: f32 = 4.0;
    pub const RAW_P25: f32 = 6.0;
    pub const RAW_P50: f32 = 9.0;
    pub const RAW_P75: f32 = 14.0;
    pub const RAW_P95: f32 = 28.0;
    pub const RAW_P99: f32 = 39.0;
    pub const TRANSFORMED_P01: f32 = 2.0;
    pub const TRANSFORMED_P05: f32 = 4.0;
    pub const TRANSFORMED_P25: f32 = 6.0;
    pub const TRANSFORMED_P50: f32 = 9.0;
    pub const TRANSFORMED_P75: f32 = 14.0;
    pub const TRANSFORMED_P95: f32 = 28.0;
    pub const TRANSFORMED_P99: f32 = 39.0;
    pub const NORMALIZED_P01: f32 = 0.0;
    pub const NORMALIZED_P05: f32 = 0.0;
    pub const NORMALIZED_P25: f32 = 0.583_333_4;
    pub const NORMALIZED_P50: f32 = 0.791_666_7;
    pub const NORMALIZED_P75: f32 = 0.916_666_7;
    pub const NORMALIZED_P95: f32 = 1.0;
    pub const NORMALIZED_P99: f32 = 1.0;
}

impl crate::board_feature::SurfaceRoughnessPenalty {
    pub const RAW_P01: f32 = 3.0;
    pub const RAW_P05: f32 = 5.0;
    pub const RAW_P25: f32 = 8.0;
    pub const RAW_P50: f32 = 12.0;
    pub const RAW_P75: f32 = 18.0;
    pub const RAW_P95: f32 = 37.0;
    pub const RAW_P99: f32 = 56.0;
    pub const TRANSFORMED_P01: f32 = 3.0;
    pub const TRANSFORMED_P05: f32 = 5.0;
    pub const TRANSFORMED_P25: f32 = 8.0;
    pub const TRANSFORMED_P50: f32 = 12.0;
    pub const TRANSFORMED_P75: f32 = 18.0;
    pub const TRANSFORMED_P95: f32 = 37.0;
    pub const TRANSFORMED_P99: f32 = 56.0;
    pub const NORMALIZED_P01: f32 = 0.0;
    pub const NORMALIZED_P05: f32 = 0.0;
    pub const NORMALIZED_P25: f32 = 0.593_75;
    pub const NORMALIZED_P50: f32 = 0.781_25;
    pub const NORMALIZED_P75: f32 = 0.906_25;
    pub const NORMALIZED_P95: f32 = 1.0;
    pub const NORMALIZED_P99: f32 = 1.0;
}

impl crate::board_feature::WellDepthPenalty {
    pub const RAW_P01: f32 = 0.0;
    pub const RAW_P05: f32 = 0.0;
    pub const RAW_P25: f32 = 0.0;
    pub const RAW_P50: f32 = 1.0;
    pub const RAW_P75: f32 = 5.0;
    pub const RAW_P95: f32 = 13.0;
    pub const RAW_P99: f32 = 20.0;
    pub const TRANSFORMED_P01: f32 = 0.0;
    pub const TRANSFORMED_P05: f32 = 0.0;
    pub const TRANSFORMED_P25: f32 = 0.0;
    pub const TRANSFORMED_P50: f32 = 1.0;
    pub const TRANSFORMED_P75: f32 = 5.0;
    pub const TRANSFORMED_P95: f32 = 13.0;
    pub const TRANSFORMED_P99: f32 = 20.0;
    pub const NORMALIZED_P01: f32 = 0.0;
    pub const NORMALIZED_P05: f32 = 0.0;
    pub const NORMALIZED_P25: f32 = 0.615_384_6;
    pub const NORMALIZED_P50: f32 = 0.923_076_9;
    pub const NORMALIZED_P75: f32 = 1.0;
    pub const NORMALIZED_P95: f32 = 1.0;
    pub const NORMALIZED_P99: f32 = 1.0;
}

impl crate::board_feature::DeepWellRisk {
    pub const RAW_P01: f32 = 0.0;
    pub const RAW_P05: f32 = 0.0;
    pub const RAW_P25: f32 = 0.0;
    pub const RAW_P50: f32 = 1.0;
    pub const RAW_P75: f32 = 5.0;
    pub const RAW_P95: f32 = 13.0;
    pub const RAW_P99: f32 = 20.0;
    pub const TRANSFORMED_P01: f32 = 0.0;
    pub const TRANSFORMED_P05: f32 = 0.0;
    pub const TRANSFORMED_P25: f32 = 0.0;
    pub const TRANSFORMED_P50: f32 = 1.0;
    pub const TRANSFORMED_P75: f32 = 5.0;
    pub const TRANSFORMED_P95: f32 = 13.0;
    pub const TRANSFORMED_P99: f32 = 20.0;
    pub const NORMALIZED_P01: f32 = 0.0;
    pub const NORMALIZED_P05: f32 = 0.0;
    pub const NORMALIZED_P25: f32 = 1.0;
    pub const NORMALIZED_P50: f32 = 1.0;
    pub const NORMALIZED_P75: f32 = 1.0;
    pub const NORMALIZED_P95: f32 = 1.0;
    pub const NORMALIZED_P99: f32 = 1.0;
}

impl crate::board_feature::TopOutRisk {
    pub const RAW_P01: f32 = 1.0;
    pub const RAW_P05: f32 = 2.0;
    pub const RAW_P25: f32 = 3.0;
    pub const RAW_P50: f32 = 4.0;
    pub const RAW_P75: f32 = 8.0;
    pub const RAW_P95: f32 = 12.0;
    pub const RAW_P99: f32 = 16.0;
    pub const TRANSFORMED_P01: f32 = 1.0;
    pub const TRANSFORMED_P05: f32 = 2.0;
    pub const TRANSFORMED_P25: f32 = 3.0;
    pub const TRANSFORMED_P50: f32 = 4.0;
    pub const TRANSFORMED_P75: f32 = 8.0;
    pub const TRANSFORMED_P95: f32 = 12.0;
    pub const TRANSFORMED_P99: f32 = 16.0;
    pub const NORMALIZED_P01: f32 = 0.0;
    pub const NORMALIZED_P05: f32 = 0.0;
    pub const NORMALIZED_P25: f32 = 1.0;
    pub const NORMALIZED_P50: f32 = 1.0;
    pub const NORMALIZED_P75: f32 = 1.0;
    pub const NORMALIZED_P95: f32 = 1.0;
    pub const NORMALIZED_P99: f32 = 1.0;
}

impl crate::board_feature::TotalHeightPenalty {
    pub const RAW_P01: f32 = 4.0;
    pub const RAW_P05: f32 = 10.0;
    pub const RAW_P25: f32 = 15.0;
    pub const RAW_P50: f32 = 27.0;
    pub const RAW_P75: f32 = 58.0;
    pub const RAW_P95: f32 = 95.0;
    pub const RAW_P99: f32 = 125.0;
    pub const TRANSFORMED_P01: f32 = 4.0;
    pub const TRANSFORMED_P05: f32 = 10.0;
    pub const TRANSFORMED_P25: f32 = 15.0;
    pub const TRANSFORMED_P50: f32 = 27.0;
    pub const TRANSFORMED_P75: f32 = 58.0;
    pub const TRANSFORMED_P95: f32 = 95.0;
    pub const TRANSFORMED_P99: f32 = 125.0;
    pub const NORMALIZED_P01: f32 = 0.0;
    pub const NORMALIZED_P05: f32 = 0.0;
    pub const NORMALIZED_P25: f32 = 0.435_294_1;
    pub const NORMALIZED_P50: f32 = 0.8;
    pub const NORMALIZED_P75: f32 = 0.941_176_5;
    pub const NORMALIZED_P95: f32 = 1.0;
    pub const NORMALIZED_P99: f32 = 1.0;
}

impl crate::board_feature::LineClearBonus {
    pub const RAW_P01: f32 = 0.0;
    pub const RAW_P05: f32 = 0.0;
    pub const RAW_P25: f32 = 0.0;
    pub const RAW_P50: f32 = 0.0;
    pub const RAW_P75: f32 = 0.0;
    pub const RAW_P95: f32 = 1.0;
    pub const RAW_P99: f32 = 3.0;
    pub const TRANSFORMED_P01: f32 = 0.0;
    pub const TRANSFORMED_P05: f32 = 0.0;
    pub const TRANSFORMED_P25: f32 = 0.0;
    pub const TRANSFORMED_P50: f32 = 0.0;
    pub const TRANSFORMED_P75: f32 = 0.0;
    pub const TRANSFORMED_P95: f32 = 0.0;
    pub const TRANSFORMED_P99: f32 = 2.0;
    pub const NORMALIZED_P01: f32 = 0.0;
    pub const NORMALIZED_P05: f32 = 0.0;
    pub const NORMALIZED_P25: f32 = 0.0;
    pub const NORMALIZED_P50: f32 = 0.0;
    pub const NORMALIZED_P75: f32 = 0.0;
    pub const NORMALIZED_P95: f32 = 0.0;
    pub const NORMALIZED_P99: f32 = 0.333_333_34;
}

impl crate::board_feature::IWellReward {
    pub const RAW_P01: f32 = 0.0;
    pub const RAW_P05: f32 = 0.0;
    pub const RAW_P25: f32 = 1.0;
    pub const RAW_P50: f32 = 2.0;
    pub const RAW_P75: f32 = 4.0;
    pub const RAW_P95: f32 = 8.0;
    pub const RAW_P99: f32 = 11.0;
    pub const TRANSFORMED_P01: f32 = 0.0;
    pub const TRANSFORMED_P05: f32 = 0.0;
    pub const TRANSFORMED_P25: f32 = 0.0;
    pub const TRANSFORMED_P50: f32 = 0.0;
    pub const TRANSFORMED_P75: f32 = 0.0;
    pub const TRANSFORMED_P95: f32 = 1.0;
    pub const TRANSFORMED_P99: f32 = 1.0;
    pub const NORMALIZED_P01: f32 = 0.0;
    pub const NORMALIZED_P05: f32 = 0.0;
    pub const NORMALIZED_P25: f32 = 0.0;
    pub const NORMALIZED_P50: f32 = 0.0;
    pub const NORMALIZED_P75: f32 = 0.0;
    pub const NORMALIZED_P95: f32 = 1.0;
    pub const NORMALIZED_P99: f32 = 1.0;
}
