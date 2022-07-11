/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
import { deserializeUnchecked, serialize } from 'borsh';
import { NATIVE_MINT, Token, TOKEN_PROGRAM_ID } from '@solana/spl-token';
import {
  Connection,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  TransactionInstruction,
} from '@solana/web3.js';
import type { Connection as ConnectionType, Signer } from '@solana/web3.js';

/**
 * Create MintTokenAccount
 * Token Account that holds amounts to send to
 * owner: payer(initializer)
 * authorizer: payer(initializer)
 */
export async function createMintTokenAccount({
  connection,
  payer,
  amount,
}: {
  connection: Connection;
  payer: Signer;
  amount: number;
  }): Promise<[PublicKey, Token]> {
  // Create token (not yet issued)
  // https://solana-labs.github.io/solana-program-library/token/js/modules.html#createMint
  const mintToken = await Token.createMint(
    connection,
    payer,
    payer.publicKey,
    null,
    0, // Consider it as NFT
    TOKEN_PROGRAM_ID,
  );
  // create token account which associated token.
  // https://solana-labs.github.io/solana-program-library/token/js/modules.html#createAccount
  const account = await mintToken.createAccount(payer.publicKey);
  // Mint the TOKEN and issue as many as the number of ACCOUNT
  // https://solana-labs.github.io/solana-program-library/token/js/modules.html#mintTo
  await mintToken.mintTo(account, payer, [], amount);
  return [account, mintToken];
}

