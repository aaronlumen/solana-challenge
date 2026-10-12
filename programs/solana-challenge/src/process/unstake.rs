use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

use crate::constants::*;
use crate::error::*;
use crate::states::*;

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub user_wallet: Signer<'info>,

    /// edtion nft mint
    pub nft_mint: Box<Account<'info, Mint>>,

    /// where the NFT is returned
    #[account(
        mut,
        constraint = user_token_account.mint == nft_mint.key() @ Errors::InvalidTokenMint,
        constraint = user_token_account.owner == user_wallet.key() @ Errors::InvalidTokenOwner
    )]
    pub user_token_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: PDA, seeds verified
    #[account(
        seeds = [
            NFT_AUTHORITY.as_ref()
        ],
        bump
    )]
    pub nft_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        constraint = nft_authority_token_account.mint == nft_mint.key() @ Errors::InvalidTokenMint,
        constraint = nft_authority_token_account.owner == nft_authority.key() @ Errors::InvalidTokenOwner
    )]
    pub nft_authority_token_account: Box<Account<'info, TokenAccount>>,

    // only the wallet that staked can unstake; rent goes back to it
    #[account(
        mut,
        seeds = [
            NFT_STAKE.as_ref(),
            nft_mint.key().as_ref()
        ],
        bump = stake_info.bump,
        constraint = stake_info.user_wallet == user_wallet.key() @ Errors::AccessDenied,
        close = user_wallet
    )]
    pub stake_info: Box<Account<'info, StakeInfo>>,

    #[account(
        mut,
        seeds = [
            USER_REPUTATION_INFO.as_ref(),
            user_wallet.key().as_ref()
        ],
        bump = reputation_info.bump,
        constraint = reputation_info.user_wallet == user_wallet.key() @ Errors::AccessDenied
    )]
    pub reputation_info: Box<Account<'info, ReputationPointInfo>>,

    /// token program
    pub token_program: Program<'info, Token>,
}

impl<'info> Unstake<'info> {
    pub fn process(&mut self, authority_bump: u8) -> Result<()> {
        token::transfer(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                Transfer {
                    from: self.nft_authority_token_account.to_account_info(),
                    to: self.user_token_account.to_account_info(),
                    authority: self.nft_authority.to_account_info(),
                },
                &[&[NFT_AUTHORITY.as_ref(), &[authority_bump]]],
            ),
            1,
        )?;

        self.reputation_info.reputation_point = self
            .reputation_info
            .reputation_point
            .checked_sub(self.stake_info.points)
            .ok_or(Errors::MathOverflow)?;

        Ok(())
    }
}
