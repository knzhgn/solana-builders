use anchor_lang::prelude::*;

use crate::{
    error::ProfileError,
    state::{UserProfile, PROFILE_SEED},
};

#[derive(Accounts)]
pub struct DeleteProfile<'info> {
    /// close = authority: Anchor обнулит данные, переведёт все lamports (rent) владельцу
    /// и передаст аккаунт System Program.
    #[account(
        mut,
        seeds = [PROFILE_SEED, authority.key().as_ref()],
        bump = profile.bump,
        has_one = authority @ ProfileError::Unauthorized,
        close = authority
    )]
    pub profile: Account<'info, UserProfile>,

    /// mut: сюда возвращается rent.
    #[account(mut)]
    pub authority: Signer<'info>,
}

pub fn handle_delete_profile(_ctx: Context<DeleteProfile>) -> Result<()> {
    Ok(())
}
