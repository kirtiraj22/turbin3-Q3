use anchor_lang::prelude::*;
use crate::{error::CustomError, state::*};

#[derive(Accounts)]
pub struct TravelToCity<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"player", signer.key().as_ref()],
        bump = player_account.bump
    )]
    pub player_account: Account<'info, PlayerAccount>,
}

impl<'info> TravelToCity<'info> {
    pub fn travel_to_city(&mut self, destination: CityName) -> Result<()> {
        require!(
            self.player_account.current_city != destination,
            CustomError::AlreadyInCity
        );

        let destination_name = destination.clone();
        self.player_account.current_city = destination;
        msg!("Traveled to {:?}!", destination_name);
        Ok(())
    }
}
