use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Monster{
    pub monster_id: String,
    pub level: u8,
    pub current_hp: u32,
    pub max_hp: u32,
    pub current_xp: u32,
    pub max_xp: u32,
    pub moves: Vec<String>,
    pub status: Option<MonsterStatus>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct MonsterStatus {
    pub status_type: String,
    pub expires_in: u8,
}

impl Monster {
    pub const MAX_MOVES: usize = 4;
    pub const MAX_MONSTER_ID_LEN: usize = 8;

    pub const LEN: usize = 
    4 + Self::MAX_MONSTER_ID_LEN + 
    1 + // level
    4 + 4 +     // current and max hp
    4 + 4 +     // current and max xp
    4 + (Self::MAX_MOVES * 16) +    // vec(4) + max moves size(16 chars per move) 
    1 + 1 + 16;     // status (Option + type + expires_in + name)

    pub fn create_starter(monster_id: String) -> Self {
        Self {
            monster_id,
            level: 1,
            current_hp: 50,
            max_hp: 50,
            current_xp: 0,
            max_xp: 100,
            moves: vec!["damage1".to_string()],
            status: None
        }
    }

    pub fn create_npc_monster(monster_id: String, level: u8) -> Self {
        let base_hp = 20 + (level as u32 * 10);
        let moves = match level {
            1 => vec!["damage1".to_string()],
            2 => vec!["damage1".to_string(), "sleepyStatus".to_string()],
            _ => vec!["damage1".to_string(), "sleepyStatus".to_string(), "paralysisStatus".to_string()],
        };

        Self {
            monster_id,
            level,
            current_hp: base_hp,
            max_hp: base_hp,
            current_xp: 0,
            max_xp: 100,
            moves,
            status: None,
        }
    }

    pub fn is_fainted(&self) -> bool {
        self.current_hp == 0
    }

    pub fn heal_to_full(&mut self) {
        self.current_hp = self.max_hp;
    }

    pub fn take_damage(&mut self, damage: u32) {
        self.current_hp = self.current_hp.saturating_sub(damage);
    }

    pub fn heal(&mut self, amount: u32) {
        self.current_hp = (self.current_hp + amount).min(self.max_hp);
    }

    pub fn gain_xp(&mut self, xp: u32) -> bool {
        self.current_xp += xp;
        if self.current_xp >= self.max_xp {
            self.level_up();
            return true;
        }
        false
    }

    fn level_up(&mut self) {
        self.level += 1;
        self.max_hp += 20;
        self.current_hp = self.max_hp; 
        self.current_xp = 0;
        self.max_xp += 50;

        match self.level {
            2 => {
                if !self.moves.contains(&"sleepyStatus".to_string()) {
                    self.moves.push("sleepyStatus".to_string());
                }
            },
            3 => {
                if !self.moves.contains(&"paralysisStatus".to_string()) {
                    self.moves.push("paralysisStatus".to_string());
                }
            },
            _ => {}
        }
    }
}