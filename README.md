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
$ solana config set --url localhost // point to local or else it will be deployed to other clusters that you set
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