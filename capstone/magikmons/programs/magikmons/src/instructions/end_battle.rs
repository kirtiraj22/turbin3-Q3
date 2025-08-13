use anchor_lang::prelude::*;
use crate::{error::CustomError, state::*};

#[derive(Accounts)]
pub struct EndBattle<'info> {
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
        close = signer
    )]
    pub battle_state: Account<'info, BattleState>,

    #[account(
        seeds = [b"config"],
        bump = game_config.config_bump
    )]
    pub game_config: Account<'info, GameConfig>,
}

impl<'info> EndBattle<'info> {
    pub fn end_battle(&mut self) -> Result<()> {
        require!(
            self.battle_state.status != BattleStatus::Active,
            CustomError::BattleStillOngoing
        );

        let npc_config = self.game_config.get_npc_config(self.battle_state.npc_id)
            .ok_or(CustomError::InvalidNpcId)?;

        let active_lineup = self.player_account.active_lineup.clone();

        for (i, lineup_index) in active_lineup.iter().enumerate() {
            if let (Some(battle_monster), Some(player_monster)) = (
                self.battle_state.player_monsters.get(i),
                self.player_account.monsters.get_mut(*lineup_index as usize)
            ) {
                player_monster.current_hp = battle_monster.current_hp;
                player_monster.level = battle_monster.level;
                player_monster.current_xp = battle_monster.current_xp;
                player_monster.max_xp = battle_monster.max_xp;
                player_monster.max_hp = battle_monster.max_hp;
                player_monster.moves = battle_monster.moves.clone();
                player_monster.status = battle_monster.status.clone();
            }
        }

        if self.battle_state.status == BattleStatus::PlayerWon {
            if let Some(defeated) = self.player_account.defeated_npcs.get_mut(self.battle_state.npc_id as usize) {
                *defeated = true;
            }

            let player_xp = match npc_config.opponent_type {
                OpponentType::Trainer => 50,
                OpponentType::GymLeader => 200,
            };

            let leveled_up = self.player_account.gain_player_xp(player_xp);
            if leveled_up {
                msg!("Player leveled up!");
            }

            if npc_config.opponent_type == OpponentType::GymLeader {
                let city_name = match npc_config.city {
                    CityName::TurbineTown => "turbine",
                    CityName::SurfpoolCity => "surfcity",
                    CityName::SolCity => "solcity",
                    CityName::SuperCity => "supercity",
                };
                
                if !self.player_account.has_badge(city_name) {
                    self.player_account.badges.push(city_name.to_string());
                    msg!("Earned {} gym badge!", city_name);
                }
            }

            self.player_account.battles_won += 1;
            msg!("Battle won! Player gained {} XP", player_xp);
        } else {
            for monster in &mut self.player_account.monsters {
                monster.heal_to_full();
                monster.status = None;
            }
            msg!("Battle lost, but monsters are healed for retry!");
        }

        self.player_account.total_battles += 1;
        Ok(())
    }
}