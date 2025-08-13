use anchor_lang::prelude::*;
use crate::state::monster::Monster;
use crate::state::enums::*;

#[account]
pub struct PlayerAccount {
    pub owner: Pubkey,
    pub name: String,
    pub current_city: CityName,
    pub level: u8,
    pub current_xp: u32,
    pub max_xp: u32,
    pub defeated_npcs: Vec<bool>, 
    pub monsters: Vec<Monster>, 
    pub active_lineup: Vec<u8>, 
    pub items: Vec<PlayerItem>,
    pub badges: Vec<String>, 
    pub total_battles: u32,
    pub battles_won: u32,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PlayerItem {
    pub action_id: String,
    pub quantity: u32,
}

impl PlayerAccount {
    pub const MAX_NAME_LEN: usize = 32;
    pub const MAX_NPCS: usize = 20;
    pub const MAX_MONSTERS: usize = 6;
    pub const MAX_LINEUP: usize = 3;
    pub const MAX_ITEMS: usize = 10;
    pub const MAX_BADGES: usize = 10;

    pub const LEN: usize = 8 + 
        32 + // owner
        4 + Self::MAX_NAME_LEN + // name
        1 + // current_city
        1 + 4 + 4 + // level, current_xp, max_xp
        4 + Self::MAX_NPCS + // defeated_npcs
        4 + (Self::MAX_MONSTERS * Monster::LEN) + // monsters
        4 + (Self::MAX_LINEUP * 1) + // active_lineup
        4 + (Self::MAX_ITEMS * (16 + 4)) + // items (action_id + quantity)
        4 + (Self::MAX_BADGES * 16) + // badges
        4 + 4 + // total_battles + battles_won
        1; // bump

    pub fn get_active_monsters(&self) -> Vec<&Monster> {
        self.active_lineup.iter()
            .filter_map(|&index| self.monsters.get(index as usize))
            .collect()
    }

    pub fn get_first_alive_monster_index(&self) -> Option<u8> {
        for &index in &self.active_lineup {
            if let Some(monster) = self.monsters.get(index as usize) {
                if !monster.is_fainted() {
                    return Some(index);
                }
            }
        }
        None
    }

    pub fn has_alive_monsters(&self) -> bool {
        self.get_first_alive_monster_index().is_some()
    }

    pub fn gain_player_xp(&mut self, xp: u32) -> bool {
        self.current_xp += xp;
        if self.current_xp >= self.max_xp {
            self.level += 1;
            self.current_xp = 0;
            self.max_xp += 100;
            self.add_item("item_recoverHp".to_string(), 2);
            return true;
        }
        false
    }

    pub fn add_item(&mut self, action_id: String, quantity: u32) {
        if let Some(existing) = self.items.iter_mut().find(|item| item.action_id == action_id) {
            existing.quantity += quantity;
        } else if self.items.len() < Self::MAX_ITEMS {
            self.items.push(PlayerItem { action_id, quantity });
        }
    }

    pub fn use_item(&mut self, action_id: &str) -> bool {
        if let Some(item) = self.items.iter_mut().find(|item| item.action_id == action_id) {
            if item.quantity > 0 {
                item.quantity -= 1;
                return true;
            }
        }
        false
    }

    pub fn has_badge(&self, city: &str) -> bool {
        self.badges.iter().any(|badge| badge == city)
    }

    pub fn can_challenge_gym_leader(&self, city: &CityName, npc_configs: &[NPCConfig]) -> bool {
        let city_trainers: Vec<usize> = npc_configs.iter()
            .enumerate()
            .filter(|(_, npc)| npc.city == *city && npc.opponent_type == OpponentType::Trainer)
            .map(|(i, _)| i)
            .collect();

        city_trainers.iter().all(|&index| self.defeated_npcs.get(index).copied().unwrap_or(false))
    }
}