use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Transfer, transfer, Mint, Token, TokenAccount, MintTo, mint_to}
};
use constant_product_curve::ConstantProduct;

use crate::{ error::AmmError, state::Config};

#[derive(Accounts)]
pub struct Deposit<'info>{
    #[account(mut)]    
    pub depositor:Signer<'info>,

    pub mint_x: Account<'info, Mint>,
    pub mint_y: Account<'info, Mint>,

    #[account(
        has_one = mint_x,
        has_one = mint_y,
        seeds = [b"config", config.config_id.to_le_bytes().as_ref()],
        bump = config.config_bump
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [b"lp", config.key().as_ref()],
        bump = config.lp_bump
    )]
    pub mint_lp: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = config,
    )]
    pub vault_x: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = config,
    )]
    pub vault_y: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = depositor,
    )]
    depositor_ata_x: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = depositor
    )]
    depositor_ata_y: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = depositor,
        associated_token::mint = mint_lp,
        associated_token::authority = depositor,
        associated_token::token_program = token_program
    )]
    depositor_ata_lp: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>
}

impl<'info> Deposit<'info>{
    pub fn deposit(&mut self, amount: u64, max_x: u64, max_y: u64) -> Result<()> {
        require!(self.config.locked == false, AmmError::PoolLocked);
        require!(amount != 0, AmmError::InvalidAmount);

        let (x, y) = match self.mint_lp.supply == 0 && self.vault_x.amount == 0 && self.vault_y.amount == 0 {
            true => (max_y, max_y),
            false => {
                let required_amount = ConstantProduct::xy_deposit_amounts_from_l(self.vault_x.amount, self.vault_y.amount, self.mint_lp.supply, amount, 6).unwrap();
                (required_amount.x, required_amount.y)
            }
        };

        require!(x <= max_x && y <= max_y, AmmError::SlippageExceeded);

        // deposit x and y tokens in respective vaults(provide liquidity)
        self.deposit_tokens(true, x)?;
        self.deposit_tokens(false, y)?;

        // mint lp tokens(as a proof) for providing liquidity
        self.mint_lp_tokens(amount)?;

        Ok(())
    }

    pub fn deposit_tokens(&mut self, is_x: bool, amount: u64) -> Result<()>{
        let (from, to) = match is_x {
            true => (self.depositor_ata_x.to_account_info(), self.vault_x.to_account_info()),
            false => (self.depositor_ata_y.to_account_info(), self.vault_y.to_account_info())
        };

        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = Transfer { 
            from, 
            to, 
            authority: self.depositor.to_account_info()
        };
        
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        transfer(cpi_ctx, amount)?;
        Ok(())
    }

    pub fn mint_lp_tokens(&mut self, amount: u64) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = MintTo {
            mint: self.mint_lp.to_account_info(),
            to: self.depositor_ata_lp.to_account_info(),
            authority: self.config.to_account_info()
        };

        let seeds = &[
            &b"config"[..],
            &self.config.config_id.to_le_bytes(),
            &[self.config.config_bump]
        ];

        let signer_seeds = &[
            &seeds[..]
        ];

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        
        mint_to(cpi_ctx, amount)?;

        Ok(())
    }
}