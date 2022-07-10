use {
  crate::{error, instruction::EscrowInstruction, state::Escrow},
  borsh::BorshDeserialize,
  solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
  },
  spl_token::{instruction, state::Account as TokenAccount},
  std::cell::RefMut,
};

fn assert_owned_by(account: &AccountInfo, owner: &Pubkey) -> ProgramResult {
  if account.owner != owner {
    Err(ProgramError::IncorrectProgramId)
  } else {
    Ok(())
  }
}

fn close_escrow_account(main_account: &AccountInfo, escrow_account: &AccountInfo) -> ProgramResult {
  // Return lamports to main account
  let returned_amount: u64 = main_account
    .lamports()
    .checked_add(escrow_account.lamports())
    .ok_or(error::EscrowError::AmountOverflow)?;

  let mut lamports: RefMut<&mut u64> = main_account.lamports.borrow_mut();

  **lamports = returned_amount;

  **escrow_account.lamports.borrow_mut() = 0;
  escrow_account.data.replace(&mut []);
  Ok(())
}

pub struct Processor;
impl Processor {
  pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
  ) -> ProgramResult {
    // instructionの中には送ったaccountの量が設定されている。
    // ここからfrontendがどのinstructionを呼び出そうとしていたかを判定する。
    // enumのmatchを使用している
    // https://doc.rust-lang.org/rust-by-example/flow_control/match/destructuring/destructure_enum.html
    let instruction = EscrowInstruction::try_from_slice(instruction_data)?;

    match instruction {
      EscrowInstruction::InitEscrow(args) => {
        msg!("Instruction: Init Escrow");
        Self::process_init_escrow(program_id, accounts, args.data.amount)
      }
      EscrowInstruction::Exchange(args) => {
        msg!("Instruction: Exchange Escrow");
        Self::process_exchange(program_id, accounts, args.data.amount)
      }
    }
  }

  pub fn process_init_escrow(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
  ) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let initializer = next_account_info(account_info_iter)?; //aliceのwallet account

    if !initializer.is_signer {
      return Err(ProgramError::MissingRequiredSignature);
    }

    let temp_token_account = next_account_info(account_info_iter)?; // aliceが作成した一時的なtoken account
    let token_to_receive_account = next_account_info(account_info_iter)?; // aliceのY(受け取り用の)、のtoken account

    // Make sure Token Account is owned by Token Program
    assert_owned_by(token_to_receive_account, &spl_token::id())?;

    let escrow_account: &AccountInfo = next_account_info(account_info_iter)?; // escrowするアカウント

    let rent: &Rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?; // rentを計算するaccount

    if !rent.is_exempt(escrow_account.lamports(), escrow_account.data_len()) {
      return Err(error::EscrowError::NotRentExmpt.into());
    }

    let mut escrow_info: Escrow = Escrow::unpack_unchecked(&escrow_account.data.borrow())?;
    if escrow_info.is_initialized() {
      return Err(ProgramError::AccountAlreadyInitialized);
    }

    escrow_info.is_initialized = true;
    escrow_info.initializer_pubkey = *initializer.key;
    escrow_info.temp_token_account_pubkey = *temp_token_account.key;
    escrow_info.initializer_token_to_receive_account_pubkey = *token_to_receive_account.key;
    escrow_info.expected_amount = amount;

    Escrow::pack(escrow_info, &mut escrow_account.data.borrow_mut())?;

    let escrow_seed = &["escrow".as_bytes(), program_id.as_ref()];

    // Program derived address for Cross Program Invocation
    let (pda_key, _bump_seed) = Pubkey::find_program_address(escrow_seed, program_id);

    let token_program: &AccountInfo = next_account_info(account_info_iter)?; // token program(すべてのtoken accountが従属(belongs to)するprogram。ownerを別のownerにできる)

    // token_program_id: &Pubkey,
    // owned_pubkey: &Pubkey,
    // new_authority_pubkey: Option<&Pubkey>,
    // authority_type: AuthorityType,
    // owner_pubkey: &Pubkey,
    // signer_pubkeys: &[&Pubkey])

    let owner_change_instruction = instruction::set_authority(
      token_program.key,
      temp_token_account.key,
      Some(&pda_key),
      instruction::AuthorityType::AccountOwner,
      initializer.key,
      &[&initializer.key],
    )?;

    // Transfer temporary token account ownership to PDA
    invoke(
      &owner_change_instruction,
      &[
        temp_token_account.clone(),
        initializer.clone(),
        token_program.clone(),
      ],
    )?;

    Ok(())
  }

  fn process_exchange(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let taker: &AccountInfo = next_account_info(account_info_iter)?;
    if !taker.is_signer {
      return Err(ProgramError::MissingRequiredSignature);
    }

    let taker_sending_token_account = next_account_info(account_info_iter)?; // bobのY(送信する用の)のtoken account
    let taker_receive_token_account = next_account_info(account_info_iter)?; // bobのX(受け取る)のtoken account

    let pda_temp_token_account: &AccountInfo = next_account_info(account_info_iter)?; //aliceが作った一時的なtoken account
    let pda_temp_token_account_info: TokenAccount =
      TokenAccount::unpack(&pda_temp_token_account.data.borrow())?;

    let escrow_seed = &["escrow".as_bytes(), program_id.as_ref()];

    // Find a valid program address and its corresponding bump seed which must be passed as an additional seed when calling invoke_signed
    let (pda_key, bump_seed) = Pubkey::find_program_address(escrow_seed, program_id);

    msg!(
      "amount {} pda_temp_amount {}",
      amount,
      pda_temp_token_account_info.amount
    );
    if amount != pda_temp_token_account_info.amount {
      return Err(error::EscrowError::ExpectedAmountMismatch.into());
    }

    let initializer_main_account: &AccountInfo = next_account_info(account_info_iter)?; //aliceのwallet account

    let initializer_receive_token_account: &AccountInfo = next_account_info(account_info_iter)?; //aliceのY(受け取り用)のtoken account

    let escrow_account: &AccountInfo = next_account_info(account_info_iter)?;

    msg!("unpacking escrow_info");
    let escrow_info: Escrow = Escrow::unpack_unchecked(&escrow_account.data.borrow())?;

    if escrow_info.temp_token_account_pubkey != *pda_temp_token_account.key {
      return Err(ProgramError::InvalidAccountData);
    }

    if escrow_info.initializer_pubkey != *initializer_main_account.key {
      return Err(ProgramError::InvalidAccountData);
    }

    if escrow_info.initializer_token_to_receive_account_pubkey
      != *initializer_receive_token_account.key
    {
      return Err(ProgramError::InvalidAccountData);
    }

    let token_program = next_account_info(account_info_iter)?; // token program(すべてのtoken accountが従属(belongs to)するprogram。ownerを別のownerにできる)

    msg!("Start transfer");
    // Yトークンをbobのtoken accountからaliceのtoken accountに送信する
    // 署名はbobによって行われる
    let transfer_to_initializer_instruction = spl_token::instruction::transfer(
      token_program.key,
      taker_sending_token_account.key,
      initializer_receive_token_account.key,
      taker.key,
      &[&taker.key],
      escrow_info.expected_amount,
    )?;

    // transfer tokens to initializer's receive token account
    invoke(
      &transfer_to_initializer_instruction,
      &[
        taker_sending_token_account.clone(),
        initializer_receive_token_account.clone(),
        taker.clone(),
        token_program.clone(),
      ],
    )?;

    let pda_account = next_account_info(account_info_iter)?; // pdaのアカウント。 aliceの一時的に作成したtoken accountの権限を持つ

    msg!("Start transfer to taker");

    // Xトークンをaliceの一時的に作成したtoken accountからbobのtoken accountに送信する
    // aliceの作成したtemp token accountの権限はpdaにあるので、pdaの権限で実行される
    let transfer_to_taker_instruction = spl_token::instruction::transfer(
      token_program.key,
      pda_temp_token_account.key,
      taker_receive_token_account.key,
      &pda_key,
      &[&pda_key],
      pda_temp_token_account_info.amount,
    )?;

    // Signer seeds to let pda invoke program as pda does not own private key
    let signers_seeds = &["escrow".as_bytes(), program_id.as_ref(), &[bump_seed]];
    // transfer tokens to taker's receive token account
    invoke_signed(
      &transfer_to_taker_instruction,
      &[
        pda_temp_token_account.clone(),
        taker_receive_token_account.clone(),
        pda_account.clone(),
        token_program.clone(),
      ],
      &[signers_seeds],
    )?;

    msg!("Close Account");
    let close_pda_temp_account_instruction = spl_token::instruction::close_account(
      token_program.key,
      pda_temp_token_account.key,
      initializer_main_account.key,
      &pda_key,
      &[&pda_key],
    )?;
    // close pda temp account
    invoke_signed(
      &close_pda_temp_account_instruction,
      &[
        pda_temp_token_account.clone(),
        initializer_main_account.clone(),
        pda_account.clone(),
        token_program.clone(),
      ],
      &[signers_seeds],
    )?;

    // Finally closing escrow account
    close_escrow_account(&initializer_main_account, &escrow_account)?;
    Ok(())
  }
}
