#[derive(Debug, Clone, Default)]
pub struct MemorySnapshot {
    pub player_name: String,
    pub player_health: u32,
    pub position: (f32, f32, f32),
    pub target_guid: Option<u64>,
}
