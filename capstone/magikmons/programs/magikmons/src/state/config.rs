use anchor_lang::prelude::*;
use crate::state::enums::*;

#[account]
pub struct GameConfig {
    pub authority: Pubkey,
    pub treasury: Pubkey,
    pub npc_configs: Vec<NPCConfig>,
    pub total_cities: u8,
    pub config_bump: u8,
}

impl GameConfig {
    pub const MAX_NPCS: usize = 20;
    pub const LEN: usize = 8 + 
        32 + 32 + // authority + treasury
        4 + (Self::MAX_NPCS * NPCConfig::LEN) +  // npc_configs (estimated size)
        1 + 1; // total_cities + config_bump

    pub fn get_npc_config(&self, npc_id: u8) -> Option<&NPCConfig> {
        self.npc_configs.get(npc_id as usize)
    }
}