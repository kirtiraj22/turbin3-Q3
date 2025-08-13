use anchor_lang::prelude::*;

#[error_code]
pub enum CustomError {
    #[msg("Battle has already ended")]
    BattleEnded,
    
    #[msg("Invalid NPC ID")]
    InvalidNpcId,
    
    #[msg("Already defeated")]
    AlreadyDefeated,
    
    #[msg("Name too long")]
    NameTooLong,
    
    #[msg("Battle is still ongoing")]
    BattleStillOngoing,

    #[msg("Monster fainted!")]
    MonsterFainted,

    #[msg("Must defeat trainers first")]
    MustDefeatTrainersFirst,

    #[msg("Already has badge for this city")]
    AlreadyHasBadge,

    #[msg("Insufficient items")]
    InsufficientItems,

    #[msg("Invalid action")]
    InvalidAction,

    #[msg("No alive monsters")]
    NoAliveMonsters,

    #[msg("No active monster")]
    NoActiveMonster,

    #[msg("Invalid monster index")]
    InvalidMonsterIndex,

    #[msg("Monster already at full HP")]
    MonsterAlreadyFullHP,

    #[msg("Already in this city")]
    AlreadyInCity,
}