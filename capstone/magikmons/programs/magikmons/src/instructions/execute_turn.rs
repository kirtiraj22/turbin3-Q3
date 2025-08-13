use anchor_lang::prelude::*;
use crate::{error::CustomError, state::*};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub enum PlayerAction {
    Attack(String), 
    UseItem { action_id: String, target_monster: u8 },
    SwapMonster(u8), 
}

#[derive(Accounts)]
pub struct ExecuteTurn<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"player", signer.key().as_ref()],
        bump = player_account.bump
    )]
    pub player_account: Account<'info, PlayerAccount>,

    #[account(
        mut,
        seeds = [b"battle", signer.key().as_ref(), &[battle_state.npc_id]],
        bump = battle_state.bump,
    )]
    pub battle_state: Account<'info, BattleState>,
}

impl<'info> ExecuteTurn<'info> {
    pub fn execute_turn(&mut self, player_action: PlayerAction) -> Result<()> {
        require!(self.battle_state.status == BattleStatus::Active, CustomError::BattleEnded);

        match player_action {
            PlayerAction::Attack(action_id) => {
                self.execute_attack(&action_id, true)?;
            }
            PlayerAction::UseItem { action_id, target_monster } => {
                self.use_item(&action_id, target_monster)?;
            }
            PlayerAction::SwapMonster(monster_index) => {
                self.swap_monster(monster_index)?;
            }
        }

        if self.is_monster_fainted(false) {
            if !self.switch_to_next_monster(false)? {
                self.battle_state.status = BattleStatus::PlayerWon;
                self.battle_state.add_event("Player wins the battle!".to_string());
                return Ok(());
            }
        }

        if self.battle_state.status == BattleStatus::Active {
            let npc_action = self.choose_npc_action();
            self.execute_npc_action(&npc_action)?;

            if self.is_monster_fainted(true) {
                if !self.switch_to_next_monster(true)? {
                    self.battle_state.status = BattleStatus::PlayerLost;
                    self.battle_state.add_event("Player loses the battle!".to_string());
                    return Ok(());
                }
            }
        }

        self.battle_state.current_turn += 1;
        Ok(())
    }

    fn execute_attack(&mut self, action_id: &str, is_player: bool) -> Result<()> {
        let (attacker_level, attacker_id) = if is_player {
            let attacker = self.battle_state.get_active_player_monster()
                .ok_or(CustomError::NoActiveMonster)?;
            (attacker.level, attacker.monster_id.clone())
        } else {
            let attacker = self.battle_state.get_active_npc_monster()
                .ok_or(CustomError::NoActiveMonster)?;
            (attacker.level, attacker.monster_id.clone())
        };

        let (damage, status_effect, event_message) = self.calculate_action_effects(action_id, attacker_level, &attacker_id)?;

        if is_player {
            if let Some(target) = self.battle_state.get_active_npc_monster_mut() {
                Self::apply_effects_to_monster(target, damage, status_effect);
            }
            self.handle_monster_defeat(is_player)?;
        } else {
            if let Some(target) = self.battle_state.get_active_player_monster_mut() {
                Self::apply_effects_to_monster(target, damage, status_effect);
            }
        }

        self.battle_state.add_event(event_message);
        Ok(())
    }

    fn execute_npc_action(&mut self, action_id: &str) -> Result<()> {
        if action_id.starts_with("heal") || action_id == "item_recoverHp" {
            if let Some(npc_monster) = self.battle_state.get_active_npc_monster_mut() {
                let heal_amount = 25;
                npc_monster.heal(heal_amount);
                let monster_id = npc_monster.monster_id.clone();
                self.battle_state.add_event(format!("{} recovers {} HP!", monster_id, heal_amount));
            }
        } else {
            self.execute_attack(action_id, false)?;
        }
        Ok(())
    }

    fn apply_effects_to_monster(monster: &mut Monster, damage: u32, status_effect: Option<MonsterStatus>) {
        if damage > 0 {
            monster.take_damage(damage);
        }
        if let Some(status) = status_effect {
            monster.status = Some(status);
        }
    }

    fn handle_monster_defeat(&mut self, is_player_attacking: bool) -> Result<()> {
        if is_player_attacking {
            let (target_fainted, target_level, target_id) = if let Some(target) = self.battle_state.get_active_npc_monster() {
                (target.is_fainted(), target.level, target.monster_id.clone())
            } else {
                return Ok(());
            };

            if target_fainted {
                if let Some(attacker) = self.battle_state.get_active_player_monster_mut() {
                    let xp_gained = target_level as u32 * 20;
                    let attacker_id = attacker.monster_id.clone();
                    if attacker.gain_xp(xp_gained) {
                        self.battle_state.add_event(format!("{} leveled up!", attacker_id));
                    }
                }
                self.battle_state.add_event(format!("{} is ruined!", target_id));
            }
        }
        Ok(())
    }

    fn is_monster_fainted(&self, is_player: bool) -> bool {
        if is_player {
            self.battle_state.get_active_player_monster()
                .map(|m| m.is_fainted())
                .unwrap_or(true)
        } else {
            self.battle_state.get_active_npc_monster()
                .map(|m| m.is_fainted())
                .unwrap_or(true)
        }
    }

    fn switch_to_next_monster(&mut self, is_player: bool) -> Result<bool> {
        if is_player {
            let player_monsters_clone = self.battle_state.player_monsters.clone();
            let current_index = self.battle_state.active_player_monster;
            
            if let Some(next_index) = self.battle_state.find_next_alive_monster(&player_monsters_clone, current_index) {
                self.battle_state.active_player_monster = next_index;
                self.battle_state.add_event("Next monster enters battle!".to_string());
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            let npc_monsters_clone = self.battle_state.npc_monsters.clone();
            let current_index = self.battle_state.active_npc_monster;
            
            if let Some(next_index) = self.battle_state.find_next_alive_monster(&npc_monsters_clone, current_index) {
                self.battle_state.active_npc_monster = next_index;
                let next_monster_id = npc_monsters_clone[next_index as usize].monster_id.clone();
                self.battle_state.add_event(format!("{} appears!", next_monster_id));
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }

    fn calculate_action_effects(&self, action_id: &str, attacker_level: u8, attacker_id: &str) -> Result<(u32, Option<MonsterStatus>, String)> {
        match action_id {
            "damage1" => {
                let damage = self.calculate_damage(10, attacker_level);
                let message = format!("{} uses Tackle for {} damage!", attacker_id, damage);
                Ok((damage, None, message))
            }
            "sleepyStatus" => {
                let status = MonsterStatus {
                    status_type: "sleepy".to_string(),
                    expires_in: 3,
                };
                let message = format!("{} applies sleepy status!", attacker_id);
                Ok((0, Some(status), message))
            }
            "paralysisStatus" => {
                let status = MonsterStatus {
                    status_type: "paralysis".to_string(),
                    expires_in: 3,
                };
                let message = format!("{} applies paralysis!", attacker_id);
                Ok((0, Some(status), message))
            }
            "heal" | "recover" => {
                let message = format!("{} recovers HP!", attacker_id);
                Ok((0, None, message))
            }
            _ => Err(CustomError::InvalidAction.into()),
        }
    }

    fn use_item(&mut self, action_id: &str, target_monster: u8) -> Result<()> {
        require!(
            (target_monster as usize) < self.battle_state.player_monsters.len(),
            CustomError::InvalidMonsterIndex
        );

        require!(
            self.player_account.use_item(action_id),
            CustomError::InsufficientItems
        );

        let monster_id = self.battle_state.player_monsters[target_monster as usize].monster_id.clone();

        if let Some(monster) = self.battle_state.player_monsters.get_mut(target_monster as usize) {
            match action_id {
                "item_recoverHp" => {
                    let heal_amount = 25;
                    monster.heal(heal_amount);
                    self.battle_state.add_event(format!("{} recovers {} HP!", monster_id, heal_amount));
                }
                _ => return Err(CustomError::InvalidAction.into()),
            }
        }

        Ok(())
    }

    fn swap_monster(&mut self, monster_index: u8) -> Result<()> {
        require!(
            (monster_index as usize) < self.battle_state.player_monsters.len(),
            CustomError::InvalidMonsterIndex
        );

        let monster = &self.battle_state.player_monsters[monster_index as usize];
        require!(!monster.is_fainted(), CustomError::MonsterFainted);
        
        require!(
            monster_index != self.battle_state.active_player_monster,
            CustomError::InvalidAction
        );

        let monster_id = monster.monster_id.clone();
        self.battle_state.active_player_monster = monster_index;
        self.battle_state.add_event(format!("Go get 'em, {}!", monster_id));

        Ok(())
    }

    fn choose_npc_action(&self) -> String {
        let npc_monster = self.battle_state.get_active_npc_monster()
            .expect("No active NPC monster");
        
        if npc_monster.current_hp < npc_monster.max_hp / 3 {
            for move_id in &npc_monster.moves {
                if move_id.contains("heal") || move_id.contains("recover") || move_id == "item_recoverHp" {
                    return move_id.clone();
                }
            }
        }
        
        let attack_moves: Vec<_> = npc_monster.moves.iter()
            .filter(|m| !m.contains("heal") && !m.contains("recover") && *m != "item_recoverHp")
            .collect();
        
        if !attack_moves.is_empty() {
            let seed = self.battle_state.battle_seed + self.battle_state.current_turn as u64;
            let move_index = (seed % attack_moves.len() as u64) as usize;
            attack_moves[move_index].clone()
        } else {
            npc_monster.moves[0].clone()
        }
    }

    fn calculate_damage(&self, base_damage: u32, level: u8) -> u32 {
        let level_multiplier = 1.0 + (level as f32 * 0.1);
        ((base_damage as f32) * level_multiplier) as u32
    }
}