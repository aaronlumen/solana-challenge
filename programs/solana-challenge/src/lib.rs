use anchor_lang::prelude::*;

pub mod process;
pub mod error;
pub mod states;
pub mod constants;

use process::*;

declare_id!("8HDnZaF3YyJb2zKeK7ND1dfgYWUAbfXqS9gpfjL1q3nR");

#[program]
pub mod solana_challenge {
    use super::*;

    // escrow MasterEdition NFT to pda
    pub fn escrow_master_nft(
        ctx: Context<EscrowMasterNft>,
        args: EscrowMasterNftArgs
    ) -> Result<()> {
        // get bump value
        let bump = *ctx.bumps.get("nft_type").unwrap();
        let slot_bump = *ctx.bumps.get("kind_slot").unwrap();
        ctx.accounts.process(bump, slot_bump, args)
    }

    // mint edition nft with selected MasterEdition NFT
    pub fn mint_edition(
        ctx: Context<MintEdition>,
        args: MintEditionArgs
    ) -> Result<()> {
        // get bump value
        let bump = *ctx.bumps.get("nft_attribute").unwrap();
        let authority_bump = *ctx.bumps.get("nft_authority").unwrap();
        ctx.accounts.process(bump, authority_bump, args)
    }

    // create PDA to save the reputation point 
    pub fn init_user_reputation(
        ctx: Context<InitUserReputation>,
    ) -> Result<()> {
        // get bump value
        let bump = *ctx.bumps.get("reputation_info").unwrap();
        ctx.accounts.process(bump)
    }

    // stake edition NFT 
    pub fn stake(
        ctx: Context<Stake>,
    ) -> Result<()> {
        // get bump value
        let bump = *ctx.bumps.get("stake_info").unwrap();
        ctx.accounts.process(bump)
    }

    // unstake edition NFT: return it and subtract its points
    pub fn unstake(
        ctx: Context<Unstake>,
    ) -> Result<()> {
        let authority_bump = *ctx.bumps.get("nft_authority").unwrap();
        ctx.accounts.process(authority_bump)
    }
}

