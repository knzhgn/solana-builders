use anchor_lang::prelude::*;

/// Seed первого уровня; второй seed — адрес кошелька владельца.
pub const PROFILE_SEED: &[u8] = b"profile";
/// Лимиты в БАЙТАХ (так String лежит on-chain): кириллица занимает 2 байта на букву.
pub const MAX_USERNAME_LEN: usize = 32;
pub const MAX_BIO_LEN: usize = 160;

/// Размер на диске = 8 (discriminator) + INIT_SPACE = 8 + 249 = 257 байт.
#[account]
#[derive(InitSpace)]
pub struct UserProfile {
    pub authority: Pubkey, // 32
    #[max_len(32)]
    pub username: String, // 4 (длина) + 32
    #[max_len(160)]
    pub bio: String, // 4 + 160
    pub created_at: i64,   // 8
    pub updated_at: i64,   // 8
    pub bump: u8,          // 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn space_matches_fields() {
        // 32 + (4+32) + (4+160) + 8 + 8 + 1 = 249
        assert_eq!(UserProfile::INIT_SPACE, 249);
    }

    #[test]
    fn pda_is_deterministic_and_off_curve() {
        let owner = Pubkey::new_unique();
        let (a, bump_a) = Pubkey::find_program_address(&[PROFILE_SEED, owner.as_ref()], &crate::ID);
        let (b, bump_b) = Pubkey::find_program_address(&[PROFILE_SEED, owner.as_ref()], &crate::ID);
        assert_eq!((a, bump_a), (b, bump_b));
        assert!(!a.is_on_curve());
        let (c, _) = Pubkey::find_program_address(
            &[PROFILE_SEED, Pubkey::new_unique().as_ref()],
            &crate::ID,
        );
        assert_ne!(a, c);
    }
}
