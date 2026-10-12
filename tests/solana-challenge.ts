import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { SolanaChallenge } from "../target/types/solana_challenge";
import { TOKEN_PROGRAM_ID, getAssociatedTokenAddress, createAssociatedTokenAccountInstruction, createInitializeMintInstruction, MINT_SIZE } from '@solana/spl-token';
import { PublicKey, Transaction, Keypair, LAMPORTS_PER_SOL, sendAndConfirmTransaction } from "@solana/web3.js";
import { assert } from "chai";
import * as constants from "./constants";
import { loadWalletKey } from "./utils";
import { createAuctionHouseOperationHandler, findEditionMarkerPda, findEditionPda, findMetadataPda } from "@metaplex-foundation/js";
var BigNumber = require('big-number');
import { PROGRAM_ID } from '@metaplex-foundation/mpl-token-metadata';

describe("solana-challenge", async () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SolanaChallenge as Program<SolanaChallenge>;

  const adminWallet = loadWalletKey("./tests/admin.json");

  const masterEditionNfts = [
    new anchor.web3.PublicKey("4dhzPDZoGkRg48AUgpQ1X5Fd1VJoXtDjLw3EhTAUMJKV"),
    new anchor.web3.PublicKey("8ViEVGXziqXJzRjFjq9pk76hLsiX258TukkWjVNQtt2E"),
    new anchor.web3.PublicKey("7NLaGCXvLofDDC74T39iFgRJPxeK6XP41FzV3adikb4E")
  ];

  const [nftAuthorityPda, nftAuthorityBump] = await PublicKey.findProgramAddress(
    [
      Buffer.from(constants.NFT_AUTHORITY),
    ],
    program.programId
  );

  // new edition nft 
  const newEditionNft = anchor.web3.Keypair.generate();
  console.log("New edition NFT: ", newEditionNft.publicKey.toString());
  // user wallet
  const user: Keypair = anchor.web3.Keypair.generate();
  console.log("User: ", user.publicKey.toString())
  // selected master edition nft
  const masterEditionNft = masterEditionNfts[0];

  let nftAttributePda;
  let nftAttributeBump;

  it("Escrow 3 MasterEdition NFTs!", async () => {
    // Escrow Master NFTs

    for(let i = 0;i<masterEditionNfts.length;i++) {
      // get pda of nft_type pda
      const [nftKindPda, nftKindBump] = await PublicKey.findProgramAddress(
        [
          Buffer.from(constants.NFT_KIND),
          masterEditionNfts[i].toBuffer(),
        ],
        program.programId
      );

      // get kind slot pda (one per kind)
      const [kindSlotPda] = await PublicKey.findProgramAddress(
        [
          Buffer.from(constants.KIND_SLOT),
          Buffer.from([i]),
        ],
        program.programId
      );
  
      // get ata of master edition nft of admin wallet
      const adminTokenAccount = await getAssociatedTokenAddress(
        masterEditionNfts[i],
        adminWallet.publicKey,
      );
  
      // get ata of master edition nft of pda
      const nftAuthorityTokenAccount = await getAssociatedTokenAddress(
        masterEditionNfts[i],
        nftAuthorityPda,
        true
      );
      
      let exist = await provider.connection.getAccountInfo(nftAuthorityTokenAccount);
      let instructions = [];
      if(!exist) {
        instructions.push(createAssociatedTokenAccountInstruction(
            adminWallet.publicKey,
            nftAuthorityTokenAccount,
            nftAuthorityPda,
            masterEditionNfts[i],
          )
        )
  
        await program.rpc.escrowMasterNft({
          kind: i
          },
          {
            accounts: {
              admin: adminWallet.publicKey,
              nftMint: masterEditionNfts[i],
              adminTokenAccount: adminTokenAccount,
              nftAuthority: nftAuthorityPda,
              nftAuthorityTokenAccount: nftAuthorityTokenAccount,
              nftType: nftKindPda,
              kindSlot: kindSlotPda,
              systemProgram: anchor.web3.SystemProgram.programId,
              tokenProgram: TOKEN_PROGRAM_ID,
            },
            instructions: instructions,
            signers: [adminWallet]
          }
        );
      }
    }
  });

  it("Mint edition nft of master edition!", async () => {
    // airdrop 2 SOL to user wallet
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(
        user.publicKey,
        2 * LAMPORTS_PER_SOL
      ),
      "confirmed"
    );

    // new edition mint address
    const newMintEdition = findEditionPda(newEditionNft.publicKey);

    // new edition metadata address
    const newMintMetadata = findMetadataPda(newEditionNft.publicKey);

    // get ata of edition nft of user wallet
    const userTokenAccount = await getAssociatedTokenAddress(
      newEditionNft.publicKey,
      user.publicKey,
    );

    
    let instructions = [];

    // create new account for edition nft
    const lamports: number = await program.provider.connection.getMinimumBalanceForRentExemption(MINT_SIZE);

    instructions.push(
      anchor.web3.SystemProgram.createAccount({
        fromPubkey: user.publicKey,
        newAccountPubkey: newEditionNft.publicKey,
        lamports,
        space: MINT_SIZE,
        programId: TOKEN_PROGRAM_ID,
      })
    );
    instructions.push(
      createInitializeMintInstruction(
        newEditionNft.publicKey,
        0,
        user.publicKey,
        user.publicKey
      )
    );

    instructions.push(createAssociatedTokenAccountInstruction(
        user.publicKey,
        userTokenAccount,
        user.publicKey,
        newEditionNft.publicKey,
      )
    );

    // get master edition mint address
    const masterEdition = findEditionPda(new anchor.web3.PublicKey(masterEditionNft));

    // get master edition meatadata address
    const masterEditionMetadata = findMetadataPda(new anchor.web3.PublicKey(masterEditionNft));

    const editionMarker = findEditionMarkerPda(new anchor.web3.PublicKey(masterEditionNft), BigNumber(1));

    // get nft kind pda of master edition nft
    const [nftKindPda, nftKindBump] = await PublicKey.findProgramAddress(
      [
        Buffer.from(constants.NFT_KIND),
        masterEditionNft.toBuffer(),
      ],
      program.programId
    );

    // get ata of master edition nft of pda
    const masterEdtionNftVault = await getAssociatedTokenAddress(
      masterEditionNft,
      nftAuthorityPda,
      true
    );

    // get nft attribute pda of edition nft
    [nftAttributePda, nftAttributeBump] = await PublicKey.findProgramAddress(
      [
        Buffer.from(constants.ATTRIBUTE),
        newEditionNft.publicKey.toBuffer(),
      ],
      program.programId
    );

    await program.rpc.mintEdition(
      {
        health: 50,
        mana: 45,
        power: 87
      },
      {
        accounts: {
          userWallet: user.publicKey,
          masterEdtionNftMint: masterEditionNft,
          userTokenAccount: userTokenAccount,
          masterEdition: masterEdition,
          masterEditionMetadata: masterEditionMetadata,
          editionMarker: editionMarker,
          newMetadata: newMintMetadata,
          newEdition: newMintEdition,
          newMint: newEditionNft.publicKey,
          tokenMetadataProgram: PROGRAM_ID,
          nftKind: nftKindPda,
          nftAuthority: nftAuthorityPda,
          masterEdtionNftVault: masterEdtionNftVault,
          nftAttribute: nftAttributePda,
          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
        instructions: instructions,
        signers: [ user, newEditionNft]
      }
    );
  });
    
  it("Stake edition NFT!", async () => {
    // get ata of edition nft of user wallet
    const userTokenAccount = await getAssociatedTokenAddress(
      newEditionNft.publicKey,
      user.publicKey,
    );

    // get ata of master edition nft of pda
    const nftAuthorityTokenAccount = await getAssociatedTokenAddress(
      newEditionNft.publicKey,
      nftAuthorityPda,
      true
    );

    let exist = await provider.connection.getAccountInfo(nftAuthorityTokenAccount);
    let instructions = [];
    if(!exist) {
      instructions.push(createAssociatedTokenAccountInstruction(
        user.publicKey,
          nftAuthorityTokenAccount,
          nftAuthorityPda,
          newEditionNft.publicKey,
        )
      )
    }

    // get stake info pda
    const [stakeInfoPda, stakeInfoBump] = await PublicKey.findProgramAddress(
      [
        Buffer.from(constants.NFT_STAKE),
        newEditionNft.publicKey.toBuffer(),
      ],
      program.programId
    );

    // get reputation info pda
    const [reputationInfoPda, reputationInfoBump] = await PublicKey.findProgramAddress(
      [
        Buffer.from(constants.USER_REPUTATION_INFO),
        user.publicKey.toBuffer(),
      ],
      program.programId
    );

    let reputationInfo = await provider.connection.getAccountInfo(reputationInfoPda);
    if(!reputationInfo) {
      await program.rpc.initUserReputation(
        {
          accounts: {
            userWallet: user.publicKey,
            reputationInfo: reputationInfoPda,
            systemProgram: anchor.web3.SystemProgram.programId,
          },
          signers: [ user ]
        }
      );
    }
    
    await program.rpc.stake(
      {
        accounts: {
          userWallet: user.publicKey,
          nftMint: newEditionNft.publicKey,
          userTokenAccount: userTokenAccount,
          nftAuthority: nftAuthorityPda,
          nftAuthorityTokenAccount: nftAuthorityTokenAccount,
          stakeInfo: stakeInfoPda,
          nftAttribute: nftAttributePda,
          reputationInfo: reputationInfoPda,
          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
        instructions: instructions,
        signers: [ user ]
      }
    );

    let reputationPointInfo = await program.account.reputationPointInfo.fetch(reputationInfoPda);
    console.log("Reputation Point Info: ", reputationPointInfo.reputationPoint.toString())
    // strength: health*2 + mana + power = 50*2 + 45 + 87
    assert.equal(reputationPointInfo.reputationPoint.toNumber(), 232);
  });

  it("Unstake edition NFT!", async () => {
    const userTokenAccount = await getAssociatedTokenAddress(newEditionNft.publicKey, user.publicKey);
    const nftAuthorityTokenAccount = await getAssociatedTokenAddress(newEditionNft.publicKey, nftAuthorityPda, true);
    const [stakeInfoPda] = await PublicKey.findProgramAddress(
      [Buffer.from(constants.NFT_STAKE), newEditionNft.publicKey.toBuffer()],
      program.programId
    );
    const [reputationInfoPda] = await PublicKey.findProgramAddress(
      [Buffer.from(constants.USER_REPUTATION_INFO), user.publicKey.toBuffer()],
      program.programId
    );

    await program.rpc.unstake({
      accounts: {
        userWallet: user.publicKey,
        nftMint: newEditionNft.publicKey,
        userTokenAccount: userTokenAccount,
        nftAuthority: nftAuthorityPda,
        nftAuthorityTokenAccount: nftAuthorityTokenAccount,
        stakeInfo: stakeInfoPda,
        reputationInfo: reputationInfoPda,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
      signers: [user],
    });

    const info = await program.account.reputationPointInfo.fetch(reputationInfoPda);
    assert.equal(info.reputationPoint.toNumber(), 0);
    assert.isNull(await provider.connection.getAccountInfo(stakeInfoPda));
  });
});
