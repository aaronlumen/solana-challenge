use anchor_lang::prelude::*;

use crate::constants::*;
use crate::error::Errors;

pub const KIND_STRENGTH: u8 = 0;
pub const KIND_AGILITY: u8 = 1;
pub const KIND_INTELLIGENCE: u8 = 2;

pub fn validate_kind(kind: u8) -> Result<()> {
    require!(kind <= KIND_INTELLIGENCE, Errors::InvalidNftKind);
    Ok(())
}

pub fn validate_attribute(value: u32) -> Result<()> {
    require!(
        (MIN_ATTRIBUTE..=MAX_ATTRIBUTE).contains(&value),
        Errors::InvalidAttribute
    );
    Ok(())
}

/// Reputation for one NFT, per the spec:
///   strength:     health*2 + mana   + power
///   agility:      health   + mana   + power*2
///   intelligence: health   + mana*2 + power
pub fn nft_points(kind: u8, health: u32, mana: u32, power: u32) -> Result<u64> {
    let (h, m, p) = (health as u64, mana as u64, power as u64);
    let (wh, wm, wp) = match kind {
        KIND_STRENGTH => (2u64, 1u64, 1u64),
        KIND_AGILITY => (1, 1, 2),
        KIND_INTELLIGENCE => (1, 2, 1),
        _ => return Err(error!(Errors::InvalidNftKind)),
    };
    h.checked_mul(wh)
        .and_then(|x| x.checked_add(m.checked_mul(wm)?))
        .and_then(|x| x.checked_add(p.checked_mul(wp)?))
        .ok_or_else(|| error!(Errors::MathOverflow))
}

#[account]
pub struct NftKind {
    /// Master edtion nft mint address
    pub nft_mint: Pubkey,

    /// MasterEdtion NFT kind
    pub nft_kind: u8, // 0: strength, 1: agility, 2: intelligence

    /// bump
    pub bump: u8
}

/// One per kind; its existence guarantees each kind is registered exactly once.
#[account]
pub struct KindSlot {
    pub nft_mint: Pubkey,
    pub bump: u8,
}

#[account]
pub struct NftAttribute {
    /// wallet that minted the edition
    pub user_wallet: Pubkey,

    /// nft mint address
    pub nft_mint: Pubkey,

    /// health
    pub health: u32,

    /// mana
    pub mana: u32,

    /// power
    pub power: u32,

    /// nft kind
    pub nft_kind: u8, // 0: strength, 1: agility, 2: intelligence

    /// bump
    pub bump: u8
}

impl NftAttribute {
    pub fn points(&self) -> Result<u64> {
        nft_points(self.nft_kind, self.health, self.mana, self.power)
    }
}

#[account]
pub struct StakeInfo {
    /// user wallet
    pub user_wallet: Pubkey,

    /// nft mint address
    pub nft_mint: Pubkey,

    /// points credited when staked (subtracted exactly on unstake)
    pub points: u64,

    /// bump
    pub bump: u8
}

#[account]
pub struct ReputationPointInfo {
    /// user wallet
    pub user_wallet: Pubkey,

    /// reputation point
    pub reputation_point: u64,

    /// bump
    pub bump: u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formulas_match_spec() {
        // health 50, mana 45, power 87 (the values used in tests/solana-challenge.ts)
        assert_eq!(nft_points(KIND_STRENGTH, 50, 45, 87).unwrap(), 232);
        assert_eq!(nft_points(KIND_AGILITY, 50, 45, 87).unwrap(), 269);
        assert_eq!(nft_points(KIND_INTELLIGENCE, 50, 45, 87).unwrap(), 227);
    }

    #[test]
    fn each_kind_wins_on_its_own_stat() {
        let best = |h, m, p| {
            (0..=2u8).max_by_key(|&k| nft_points(k, h, m, p).unwrap()).unwrap()
        };
        assert_eq!(best(100, 10, 10), KIND_STRENGTH);
        assert_eq!(best(10, 10, 100), KIND_AGILITY);
        assert_eq!(best(10, 100, 10), KIND_INTELLIGENCE);
    }

    #[test]
    fn rejects_bad_kind_and_never_overflows() {
        assert!(nft_points(3, 1, 1, 1).is_err());
        assert!(nft_points(KIND_AGILITY, u32::MAX, u32::MAX, u32::MAX).is_ok());
    }

    #[test]
    fn attribute_bounds() {
        assert!(validate_attribute(0).is_err());
        assert!(validate_attribute(MIN_ATTRIBUTE).is_ok());
        assert!(validate_attribute(MAX_ATTRIBUTE).is_ok());
        assert!(validate_attribute(MAX_ATTRIBUTE + 1).is_err());
    }
}
