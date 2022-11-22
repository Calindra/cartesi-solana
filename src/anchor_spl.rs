pub use anchor_spl::*;
use anchor_lang::solana_program::{account_info::AccountInfo, clock::Epoch, pubkey::Pubkey};

#[derive(Clone)]
pub struct NativeAccountData {
    pub key: Pubkey,
    pub lamports: u64,
    pub data: Vec<u8>,
    pub program_id: Pubkey,
    pub is_signer: bool,
}

impl NativeAccountData {
    pub fn new(size: usize, program_id: Pubkey) -> Self {
        Self {
            key: Pubkey::new_unique(),
            lamports: 0,
            data: vec![0; size],
            program_id,
            is_signer: false,
        }
    }

    pub fn new_from_account_info(account_info: &AccountInfo) -> Self {
        Self {
            key: *account_info.key,
            lamports: **account_info.lamports.borrow(),
            data: account_info.data.borrow().to_vec(),
            program_id: *account_info.owner,
            is_signer: account_info.is_signer,
        }
    }

    pub fn as_account_info(&mut self) -> AccountInfo {
        AccountInfo::new(
            &self.key,
            self.is_signer,
            false,
            &mut self.lamports,
            &mut self.data[..],
            &self.program_id,
            false,
            Epoch::default(),
        )
    }
}

#[cfg(not(target_arch = "bpf"))]
pub mod token {

    use anchor_lang::{prelude::{CpiContext, Result}, solana_program::program_pack::Pack};
    pub use anchor_spl::token::*;
    // use anchor_lang::solana_program;
    use spl_token::state::{Account as InnerTokenAccount, AccountState as TokenAccountState};

    use crate::anchor_spl::NativeAccountData;

    pub fn initialize_account<'a, 'b, 'c, 'info>(
        ctx: CpiContext<'a, 'b, 'c, 'info, InitializeAccount<'info>>,
    ) -> Result<()> {
        anchor_lang::prelude::msg!("Inside token initialize_account...");
        let account_info = ctx.accounts.account;
        let token_account = InnerTokenAccount {
            mint: *ctx.accounts.mint.key,
            owner: *ctx.accounts.authority.key,
            amount: 100,
            state: TokenAccountState::Initialized,
            ..Default::default()
        };
        let mut account_data = NativeAccountData::new(InnerTokenAccount::LEN, spl_token::id());
        InnerTokenAccount::pack(token_account, &mut account_data.data).unwrap();
        // println!("{:?}", account_data.data);
        **account_info.lamports.borrow_mut() += 100000;
        // account_info.owner = &spl_token::ID;
        // println!("InnerTokenAccount::LEN = {:?}", InnerTokenAccount::LEN);
        println!("initialize_account: key = {:?}", account_info.key);
        println!("initialize_account: inner owner = {:?}", account_info.owner);
        println!("initialize_account: ctx.accounts.authority.key = {:?}", ctx.accounts.authority.key);
        // account_info.data.replace_with(f)
        let mut data = account_info.data.borrow_mut();
        let mut i = 0;
        for val in account_data.data.iter() {
            data[i] = *val;
            i = i + 1;
        }
        // println!("{:?}", data);
        // let ix = spl_token::instruction::initialize_account(
        //     &spl_token::ID,
        //     ctx.accounts.account.key,
        //     ctx.accounts.mint.key,
        //     ctx.accounts.authority.key,
        // )?;
        // solana_program::program::invoke_signed(
        //     &ix,
        //     &[
        //         ctx.accounts.account.clone(),
        //         ctx.accounts.mint.clone(),
        //         ctx.accounts.authority.clone(),
        //         ctx.accounts.rent.clone(),
        //     ],
        //     ctx.signer_seeds,
        // )
        // .map_err(Into::into)
        Ok(())
    }
}


