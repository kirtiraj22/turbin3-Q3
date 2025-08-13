use anchor_lang::prelude::*;
use crate::state::*;

#[derive(Accounts)]
pub struct AddNpcToConfig<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"config"],
        bump = game_config.config_bump,
        has_one = authority
    )]
    pub game_config: Account<'info, GameConfig>,
}

impl<'info> AddNpcToConfig<'info> {
    pub fn add_npc_to_config(&mut self, npc_config: NPCConfig) -> Result<()> {
        require!(
            self.game_config.npc_configs.len() < GameConfig::MAX_NPCS,
            anchor_lang::error::ErrorCode::AccountDidNotSerialize
        );

        self.game_config.npc_configs.push(npc_config);
        msg!("NPC added to game config");
        Ok(())
    }
}