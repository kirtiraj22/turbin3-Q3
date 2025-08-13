use anchor_lang::prelude::*;
use crate::state::*;

#[derive(Accounts)]
pub struct HealMonsters<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"player", signer.key().as_ref()],
        bump = player_account.bump
    )]
    pub player_account: Account<'info, PlayerAccount>,
}

impl<'info> HealMonsters<'info> {
    pub fn heal_monsters(&mut self) -> Result<()> {
        for monster in &mut self.player_account.monsters {
            monster.heal_to_full();
            monster.status = None;
        }

        msg!("All monsters healed to full HP!");
        Ok(())
    }
}