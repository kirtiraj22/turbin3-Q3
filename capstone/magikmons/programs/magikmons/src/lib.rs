pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("7drP7t7GV2zfuru8dnKwnj9TRUUZ6mA9pDfMgk6yTrpJ");

#[program]
pub mod magikmons {
    use super::*;
    pub fn initialize_game(
        ctx: Context<InitializeGame>,
        treasury: Pubkey,
    ) -> Result<()> {
        ctx.accounts.initialize_game(treasury, &ctx.bumps)
    }
    pub fn create_player(
        ctx: Context<CreatePlayer>,
        name: String,
    ) -> Result<()> {
        ctx.accounts.create_player(name, &ctx.bumps)
    }
    pub fn start_battle(
        ctx: Context<StartBattle>,
        npc_id: u8,
    ) -> Result<()> {
        ctx.accounts.start_battle(npc_id, &ctx.bumps)
    }
    pub fn execute_turn(
        ctx: Context<ExecuteTurn>,
        player_action: PlayerAction,
    ) -> Result<()> {
        ctx.accounts.execute_turn(player_action)
    }
    pub fn end_battle(
        ctx: Context<EndBattle>,
    ) -> Result<()> {
        ctx.accounts.end_battle()
    }
    pub fn travel_to_city(
        ctx: Context<TravelToCity>,
        destination: CityName,
    ) -> Result<()> {
        ctx.accounts.travel_to_city(destination)
    }
    pub fn heal_monsters(
        ctx: Context<HealMonsters>,
    ) -> Result<()> {
        ctx.accounts.heal_monsters()
    }
    pub fn add_npc_to_config(
        ctx: Context<AddNpcToConfig>,
        npc_config: NPCConfig,
    ) -> Result<()> {
        ctx.accounts.add_npc_to_config(npc_config)
    }
}