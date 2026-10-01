use anchor_lang::prelude::*;

use crate::state::{UserProfile, PROFILE_SEED};

#[derive(Accounts)]
pub struct CreateProfile<'info> {
    /// init + seeds + bump: Anchor вычисляет PDA из seeds, находит canonical bump, сверяет
    /// с переданным адресом и создаёт аккаунт через System Program, подписывая за PDA
    /// (invoke_signed). Если по этому адресу аккаунт уже есть, создание падает: один профиль на кошелёк.
    #[account(
        init,
        payer = authority,
        space = UserProfile::DISCRIMINATOR.len() + UserProfile::INIT_SPACE,
        seeds = [PROFILE_SEED, authority.key().as_ref()],
        bump
    )]
    pub profile: Account<'info, UserProfile>,

    /// Платит за аккаунт и становится владельцем профиля.
    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handle_create_profile(
    ctx: Context<CreateProfile>,
    username: String,
    bio: String,
) -> Result<()> {
    super::validate(&username, &bio)?;
    let now = Clock::get()?.unix_timestamp; // время текущего блока, unix-секунды
    let profile = &mut ctx.accounts.profile;
    profile.authority = ctx.accounts.authority.key();
    profile.username = username;
    profile.bio = bio;
    profile.created_at = now;
    profile.updated_at = now;
    // Сохраняем canonical bump: дальше PDA проверяется одним вычислением, без перебора bump.
    profile.bump = ctx.bumps.profile;
    Ok(())
}
