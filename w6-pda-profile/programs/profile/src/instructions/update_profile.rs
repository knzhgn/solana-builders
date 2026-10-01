use anchor_lang::prelude::*;

use crate::{
    error::ProfileError,
    state::{UserProfile, PROFILE_SEED},
};

#[derive(Accounts)]
pub struct UpdateProfile<'info> {
    /// seeds + bump = profile.bump: адрес должен быть PDA именно этого authority.
    /// has_one: поле profile.authority в данных тоже равно authority (вторая защита).
    #[account(
        mut,
        seeds = [PROFILE_SEED, authority.key().as_ref()],
        bump = profile.bump,
        has_one = authority @ ProfileError::Unauthorized
    )]
    pub profile: Account<'info, UserProfile>,

    pub authority: Signer<'info>,
}

pub fn handle_update_profile(
    ctx: Context<UpdateProfile>,
    username: String,
    bio: String,
) -> Result<()> {
    super::validate(&username, &bio)?;
    let profile = &mut ctx.accounts.profile;
    profile.username = username;
    profile.bio = bio;
    profile.updated_at = Clock::get()?.unix_timestamp;
    Ok(())
}
