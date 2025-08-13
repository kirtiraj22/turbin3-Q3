use anchor_lang::prelude::*;
use crate::state::*;

#[derive(Accounts)]
pub struct InitializeGame<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        seeds = [b"config"],
        bump,
        space = GameConfig::LEN
    )]
    pub game_config: Account<'info, GameConfig>,

    pub system_program: Program<'info, System>,
}

impl<'info> InitializeGame<'info> {
    pub fn initialize_game(&mut self, treasury: Pubkey, bumps: &InitializeGameBumps) -> Result<()> {
        let default_npcs = vec![
            NPCConfig {
                city: CityName::TurbineTown,
                opponent_type: OpponentType::Trainer,
                name: "Natty Node Nate".to_string(),
                monsters: vec!["f001".to_string()],
                monster_levels: vec![1],
            },
            NPCConfig {
                city: CityName::TurbineTown,
                opponent_type: OpponentType::Trainer,
                name: "Liquidity Lord Andre".to_string(),
                monsters: vec!["w001".to_string(), "f002".to_string()],
                monster_levels: vec![2, 2],
            },
            NPCConfig {
                city: CityName::TurbineTown,
                opponent_type: OpponentType::GymLeader,
                name: "Devnet Whale Jeff".to_string(),
                monsters: vec!["l001".to_string(), "f003".to_string(), "w002".to_string()],
                monster_levels: vec![3, 3, 4],
            },
        ];

        self.game_config.set_inner(GameConfig {
            authority: self.authority.key(),
            treasury,
            npc_configs: default_npcs,
            total_cities: 1,
            config_bump: bumps.game_config,
        });

        msg!("Game initialized with TurbineTown city NPCs");
        Ok(())
    }
}