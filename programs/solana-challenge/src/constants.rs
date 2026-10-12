pub const NFT_KIND: &[u8] = b"nft-kind1";

pub const KIND_SLOT: &[u8] = b"kind-slot";

pub const NFT_AUTHORITY: &[u8] = b"nft-authority";

pub const ATTRIBUTE: &[u8] = b"nft-attribute";

pub const NFT_STAKE: &[u8] = b"nft-stake";

pub const USER_REPUTATION_INFO: &[u8] = b"user-reputation-info";

/// Inclusive bounds for user-chosen attributes (health, mana, power).
pub const MIN_ATTRIBUTE: u32 = 1;
pub const MAX_ATTRIBUTE: u32 = 100;

/// Only this wallet may escrow the three MasterEdition NFTs.
/// Replace with the real admin key before deploying.
pub mod admin {
    anchor_lang::declare_id!("9MBZ3Wqnrkt249FMPC5ne4QTMVnHWgSGVxoeSTdBQgK3");
}
