use anchor_lang::prelude::*;

#[error_code]
pub enum Errors {
    #[msg("Invalid Token Owner")]
    InvalidTokenOwner,

    #[msg("Invalid Token Mint")]
    InvalidTokenMint,

    #[msg("Access Denied")]
    AccessDenied,

    #[msg("Math overflow")]
    MathOverflow,

    #[msg("Invalid Master edtion. Should be strength or agility or intelligence")]
    InvalidNftKind,

    #[msg("Attribute out of range")]
    InvalidAttribute,

    #[msg("Mint is not a 1-of-1 NFT")]
    NotAnNft,
}
