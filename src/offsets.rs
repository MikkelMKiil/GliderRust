#[derive(Debug, Clone, Copy)]
pub struct OffsetTable {
    pub player_guid: usize,
    pub player_xyz: usize,
    pub target_guid: usize,
}

pub const WOTLK_3_3_5A: OffsetTable = OffsetTable {
    player_guid: 0x00C79D18,
    player_xyz: 0x00798,
    target_guid: 0x00BD07B0,
};
