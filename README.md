# solana escrow demo

This app is for verifying smart contracts(programs) implemented on Solana for self-learning.<br> 
This app implements token exchange with escrow account.

## Programs

Currently implemented under `rust` directory

### Simulate Escrow

- Idea is from [paulx's blog](https://paulx.dev/blog/2021/01/14/programming-on-solana-an-introduction) but following a real-world usecase
  - Use wallets to manage Accounts
  - Use Associated Token Account to transfer tokens

- other reference
  - [solana-sandbox](https://github.com/tomoima525/solana-sandbox)
  - [Program Derived Address日本語で](https://efficacious-flat-24a.notion.site/Program-Derived-Address-8537ebca002245639beb531842f87f2c)

<br>
If Alice's X token is exchanged for Bob's Y token + Alice becomes the initializer and Bob becomes the taker.

> I'll be referring to Alice as the initializer and Bob as the taker in the code (Alice inits the escrow, Bob takes the trade. Pls let me know if you can come up with better naming)

escrow init

1. Alice (initializer) creates a temporary account (temp token account).
2. pass the X token you want to pass to the temp token account. 
3. change the owner of the temp token account to PDA. 
4. save the terms of the exchange transaction in the escrow account
    - Which is Alice's token account to receive Y
    - Which temp token account

escrow exchange

1. check the terms and conditions of the exchange transaction stored in the escrow account and make sure they are OK.
2. Y token is sent from bob's token account to Alice's token account. Signature is done by bob.
3. X token is sent from Alice's temporarily created token account to bob's token account. The authority of Alice's temp token account is in pda, so it is executed with pda's authority.
4. delete the temp token account created by Alice.

## Client

### Mint & add metadata on NFT

- Solana has built-in API for minting(`@solana/spl-token`) and transfering transaction(`@solana/web3.js`). This code also showcases how we use them.

## Prerequisite

- rust environment
- node version v14+
- solana tookkit
- solana account that holds decent `SOL` to operate(mint, etc)
  - You can use [airdrop to fund yourself](https://spl.solana.com/token#airdrop-sol)

## How to start

- Install dependencies

```
$ cargo install
$ yarn install
```

- Set up local network for Solana

```
$ solana config set --url localhost --keypair {wallet path} // point to local or else it will be deployed to other clusters that you set
```

- airdrop
```
$ solana airdrop 1 {wallet public key} --url localhost
$ solana balance // confirmation
```

- Start local validator

```
$ solana-test-validator
```

- Build & deploy program

```
$ cargo build-bpf // compile program
$ solana program deploy /{your directory}/rust/target/deploy/escrow.so

// You should receive the program id as a result
Program Id: zvM2...
```

The above command will generate `escrow-keypair.json` which will be used in the client code.

- set `your wallet keypair` to `.env`.  
Create `.env` with reference to `.env_sample`.

### Test Escrow

```
$ yarn start:escrow
```