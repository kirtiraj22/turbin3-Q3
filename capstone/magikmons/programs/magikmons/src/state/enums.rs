use anchor_lang::prelude::*;

#[derive(Debug)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum CityName {
    TurbineTown,
    SurfpoolCity,
    SolCity,
    SuperCity,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum OpponentType {
    Trainer,
    GymLeader,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum BattleStatus {
    Active,
    PlayerWon,
    PlayerLost,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct NPCConfig {
    pub city: CityName,
    pub opponent_type: OpponentType,
    pub name: String,
    pub monsters: Vec<String>, 
    pub monster_levels: Vec<u8>, 
}

impl NPCConfig {
    pub const MAX_NAME_LEN: usize = 32;
    pub const MAX_MONSTERS: usize = 6;
    pub const MAX_MONSTER_ID_LEN: usize = 8;

    pub const LEN: usize = 
        1 + // city enum
        1 + // opponent_type enum
        4 + Self::MAX_NAME_LEN + // name
        4 + (Self::MAX_MONSTERS * (4 + Self::MAX_MONSTER_ID_LEN)) + // monsters
        4 + Self::MAX_MONSTERS; // monster_levels
}