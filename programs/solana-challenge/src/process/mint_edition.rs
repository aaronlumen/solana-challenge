use anchor_lang::{prelude::*, solana_program::{program::invoke_signed}};
use anchor_spl::token::{self, Mint, Token, TokenAccount, MintTo};
use crate::constants::*;
use crate::error::*;
use crate::states::*;
use mpl_token_metadata::utils::get_supply_off_master_edition;
use std::mem::size_of;

#[derive(Accounts)]
pub struct MintEdition<'info> {
    /// user wallet
    #[account(mut)]
    pub user_wallet: Signer<'info>,

    /// master edtion nft mint address
    #[account(
        mut,
        constraint = master_edtion_nft_mint.key() == nft_kind.nft_mint @ Errors::InvalidTokenMint,
    )]
    pub master_edtion_nft_mint: Box<Account<'info, Mint>>,

    /// user token account for edtion nft
    #[account(
        mut, 
        constraint = user_token_account.owner == user_wallet.key() @ Errors::InvalidTokenOwner,
        constraint = user_token_account.mint == new_mint.key() @ Errors::InvalidTokenMint,
    )]
    pub user_token_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: checked in program
    #[account(
        mut, 
        owner=mpl_token_metadata::id()
    )]
    pub master_edition: UncheckedAccount<'info>,

    /// CHECK: checked in program
    #[account(
        mut, 
        owner=mpl_token_metadata::id()
    )]
    pub master_edition_metadata: UncheckedAccount<'info>,

    /// CHECK: checked in program
    #[account(mut)]
    pub edition_marker: UncheckedAccount<'info>,

    /// CHECK: checked in program
    #[account(mut)]
    pub new_metadata: UncheckedAccount<'info>,

    /// CHECK: checked in program
    #[account(mut)]
    pub new_edition: UncheckedAccount<'info>,

    #[account(mut)]
    pub new_mint: Signer<'info>,

    /// CHECK: pinned to the Metaplex Token Metadata program
    #[account(address = mpl_token_metadata::id())]
    pub token_metadata_program: UncheckedAccount<'info>,

    /// master edtion nft info pda
    #[account(
        seeds = [
            NFT_KIND.as_ref(),
            master_edtion_nft_mint.key().as_ref()
        ],
        bump = nft_kind.bump,
    )]
    pub nft_kind: Box<Account<'info, NftKind>>,

    /// CHECK: Safe account
    #[account(
        seeds = [
            NFT_AUTHORITY.as_ref()
        ],
        bump
    )]
    pub nft_authority: UncheckedAccount<'info>,

    #[account(
        mut, 
        constraint = master_edtion_nft_vault.owner == nft_authority.key() @ Errors::InvalidTokenOwner,
        constraint = master_edtion_nft_vault.mint == nft_kind.nft_mint @ Errors::InvalidTokenMint,
    )]
    pub master_edtion_nft_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        seeds = [
            ATTRIBUTE.as_ref(),
            new_mint.key().as_ref()
        ],
        bump,
        payer = user_wallet,
        space = 8 + size_of::<NftAttribute>(),
    )]
    pub nft_attribute: Box<Account<'info, NftAttribute>>,

    /// system program
    pub system_program: Program<'info, System>,

    /// token program
    pub token_program: Program<'info, Token>,
}


impl<'info> MintEdition<'info> {
    pub fn process(&mut self, pda_bump: u8, authority_bump: u8, args: MintEditionArgs) -> Result<()> {
        validate_attribute(args.health)?;
        validate_attribute(args.mana)?;
        validate_attribute(args.power)?;

        // get edtion number of master edtion
        let edition = get_supply_off_master_edition(&self.master_edition.to_account_info())?
                .checked_add(1)
                .ok_or(Errors::MathOverflow)?;

        // mint new token account to user wallet for edtion nft
        token::mint_to(
            CpiContext::new(
            self.token_program.to_account_info().clone(), 
            MintTo {
                    mint: self.new_mint.to_account_info().clone(),
                    to: self.user_token_account.to_account_info().clone(),
                    authority: self.user_wallet.to_account_info().clone(),
                }
            ),
            1,
        )?;

        // mint edtion nft from master edtion NFT
        let tx = mpl_token_metadata::instruction::mint_new_edition_from_master_edition_via_token(
            self.token_metadata_program.key(),
            self.new_metadata.key(),
            self.new_edition.key(),
            self.master_edition.key(),
            self.new_mint.key(),
            self.user_wallet.key(),
            self.user_wallet.key(),
            self.nft_authority.key(),
            self.master_edtion_nft_vault.key(),
            self.user_wallet.key(),
            self.master_edition_metadata.key(),
            self.master_edtion_nft_mint.key(),
            edition,
        );
        
        invoke_signed(
            &tx,
            &[
                self.token_metadata_program.to_account_info().clone(),
                self.new_metadata.to_account_info().clone(),
                self.new_edition.to_account_info().clone(),
                self.master_edition.to_account_info().clone(),
                self.new_mint.to_account_info().clone(),
                self.edition_marker.to_account_info().clone(),
                self.user_wallet.to_account_info().clone(),
                self.nft_authority.to_account_info().clone(),
                self.master_edtion_nft_vault.to_account_info().clone(),
                self.user_wallet.to_account_info().clone(),
                self.master_edition_metadata.to_account_info().clone(),
                self.master_edtion_nft_mint.to_account_info().clone(),
                self.token_program.to_account_info().clone(),
                self.system_program.to_account_info().clone(),
            ],
            &[&[NFT_AUTHORITY.as_ref(), &[authority_bump]]],
        )?;

        self.nft_attribute.user_wallet = self.user_wallet.key();
        self.nft_attribute.nft_mint = self.new_mint.key();
        self.nft_attribute.health = args.health;
        self.nft_attribute.mana = args.mana;
        self.nft_attribute.power = args.power;
        self.nft_attribute.nft_kind = self.nft_kind.nft_kind;
        self.nft_attribute.bump = pda_bump;

        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct MintEditionArgs {
    /// health
    pub health: u32,

    /// mana
    pub mana: u32,

    /// power
    pub power: u32,
}