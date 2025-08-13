use anchor_lang::prelude::*;
use crate::{error::CustomError, state::*};

#[derive(Accounts)]
#[instruction(name: String)]
pub struct CreatePlayer<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init,
        payer = signer,
        seeds = [b"player", signer.key().as_ref()],
        bump,
        space = PlayerAccount::LEN
    )]
    pub player_account: Account<'info, PlayerAccount>,

    pub system_program: Program<'info, System>,
}

impl<'info> CreatePlayer<'info> {
    pub fn create_player(&mut self, name: String, bumps: &CreatePlayerBumps) -> Result<()> {
        require!(name.len() <= PlayerAccount::MAX_NAME_LEN, CustomError::NameTooLong);

        let starter_monster = Monster::create_starter("f001".to_string());

        self.player_account.set_inner(PlayerAccount {
            owner: self.signer.key(),
            name,
            current_city: CityName::TurbineTown,
            level: 1,
            current_xp: 0,
            max_xp: 100,
            defeated_npcs: vec![false; 20], 
            monsters: vec![starter_monster],
            active_lineup: vec![0],
            items: vec![PlayerItem {
                action_id: "item_recoverHp".to_string(),
                quantity: 3,
            }],
            badges: vec![],
            total_battles: 0,
            battles_won: 0,
            bump: bumps.player_account,
        });

        msg!("Player created with starter monster!");
        Ok(())
    }
}