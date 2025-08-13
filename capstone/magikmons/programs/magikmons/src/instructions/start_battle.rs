use anchor_lang::prelude::*;
use crate::error::CustomError;
use crate::state::*;

#[derive(Accounts)]
#[instruction(npc_id: u8)]
pub struct StartBattle<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"player", signer.key().as_ref()],
        bump = player_account.bump
    )]
    pub player_account: Account<'info, PlayerAccount>,

    #[account(
        seeds = [b"config"],
        bump = game_config.config_bump
    )]
    pub game_config: Account<'info, GameConfig>,

    #[account(
        init,
        payer = signer,
        seeds = [b"battle", signer.key().as_ref(), &[npc_id]],
        bump,
        space = BattleState::LEN
    )]
    pub battle_state: Account<'info, BattleState>,

    pub system_program: Program<'info, System>,
}

impl<'info> StartBattle<'info> {
    pub fn start_battle(&mut self, npc_id: u8, bumps: &StartBattleBumps) -> Result<()> {
        require!(
            self.player_account.has_alive_monsters(),
            CustomError::NoAliveMonsters
        );

        let npc_config = self.game_config.get_npc_config(npc_id)
            .ok_or(CustomError::InvalidNpcId)?;

        require!(
            !self.player_account.defeated_npcs.get(npc_id as usize).copied().unwrap_or(false),
            CustomError::AlreadyDefeated
        );

        if npc_config.opponent_type == OpponentType::GymLeader {
            require!(
                self.player_account.can_challenge_gym_leader(&npc_config.city, &self.game_config.npc_configs),
                CustomError::MustDefeatTrainersFirst
            );
            
            let city_name = match npc_config.city {
                CityName::TurbineTown => "turbine",
                CityName::SurfpoolCity => "surfcity",
                CityName::SolCity => "solcity",
                CityName::SuperCity => "supercity",
            };
            
            require!(
                !self.player_account.has_badge(city_name),
                CustomError::AlreadyHasBadge
            );
        }

        let npc_monsters: Vec<Monster> = npc_config.monsters.iter()
            .zip(npc_config.monster_levels.iter())
            .map(|(monster_id, &level)| Monster::create_npc_monster(monster_id.clone(), level))
            .collect();

        let player_monsters: Vec<Monster> = self.player_account.active_lineup.iter()
            .filter_map(|&index| self.player_account.monsters.get(index as usize))
            .cloned()
            .collect();

        require!(!player_monsters.is_empty(), CustomError::NoAliveMonsters);

        let clock = Clock::get()?;
        let battle_seed = clock.unix_timestamp as u64 + clock.slot;

        self.battle_state.set_inner(BattleState {
            player: self.player_account.key(),
            npc_id,
            player_monsters,
            npc_monsters,
            active_player_monster: 0,
            active_npc_monster: 0,
            current_turn: 0,
            status: BattleStatus::Active,
            battle_seed,
            turn_events: vec![format!("{} wants to throw down!", npc_config.name)],
            bump: bumps.battle_state,
        });

        msg!("Battle started against NPC {} in {:?}", npc_config.name, npc_config.city);
        Ok(())
    }
}