use anchor_lang::prelude::*;
use crate::state::*;

#[account]
pub struct BattleState {
    pub player: Pubkey,
    pub npc_id: u8,
    pub player_monsters: Vec<Monster>, 
    pub npc_monsters: Vec<Monster>, 
    pub active_player_monster: u8, 
    pub active_npc_monster: u8,
    pub current_turn: u8,
    pub status: BattleStatus,
    pub battle_seed: u64,
    pub turn_events: Vec<String>, 
    pub bump: u8,
}

impl BattleState {
    pub const MAX_TEAM_SIZE: usize = 3;
    pub const MAX_EVENTS: usize = 50;

    pub const LEN: usize = 8 +
        32 + // player
        1 + // npc_id
        4 + (Self::MAX_TEAM_SIZE * Monster::LEN * 2) + // both teams
        1 + 1 + // active monster indices
        1 + // current_turn
        1 + // status
        8 + // battle_seed
        4 + (Self::MAX_EVENTS * 64) + // turn_events
        1; // bump

    pub fn get_active_player_monster(&self) -> Option<&Monster> {
        self.player_monsters.get(self.active_player_monster as usize)
    }

    pub fn get_active_npc_monster(&self) -> Option<&Monster> {
        self.npc_monsters.get(self.active_npc_monster as usize)
    }

    pub fn get_active_player_monster_mut(&mut self) -> Option<&mut Monster> {
        self.player_monsters.get_mut(self.active_player_monster as usize)
    }

    pub fn get_active_npc_monster_mut(&mut self) -> Option<&mut Monster> {
        self.npc_monsters.get_mut(self.active_npc_monster as usize)
    }

    pub fn find_next_alive_monster(&self, team: &[Monster], current: u8) -> Option<u8> {
        for (i, monster) in team.iter().enumerate() {
            if i != current as usize && !monster.is_fainted() {
                return Some(i as u8);
            }
        }
        None
    }

    pub fn has_alive_monsters(&self, team: &[Monster]) -> bool {
        team.iter().any(|m| !m.is_fainted())
    }

    pub fn add_event(&mut self, event: String) {
        if self.turn_events.len() < Self::MAX_EVENTS {
            self.turn_events.push(event);
        }
    }
}