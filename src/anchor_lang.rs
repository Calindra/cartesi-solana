pub use anchor_lang::*;

pub static mut TIMESTAMP: i64 = 0;
// Replacing the
// anchor_lang::system_program::create_account
#[cfg(not(target_arch = "bpf"))]
pub mod system_program {
    use anchor_lang::prelude::{CpiContext, Pubkey, Result};
    // use anchor_lang::solana_program;
    use crate::{account_manager, owner_manager};
    pub use anchor_lang::system_program::*;

    pub fn create_account<'a, 'b, 'c, 'info>(
        ctx: CpiContext<'a, 'b, 'c, 'info, CreateAccount<'info>>,
        lamports: u64,
        space: u64,
        owner: &Pubkey,
    ) -> Result<()> {
        anchor_lang::prelude::msg!("Inside lang system_program create_account... {}", lamports);
        owner_manager::change_owner(ctx.accounts.to.key.clone(), owner.clone());
        **ctx.accounts.to.try_borrow_mut_lamports()? += lamports;
        account_manager::set_data_size(&ctx.accounts.to, space.try_into().unwrap());

        // let ix = solana_program::system_instruction::create_account(
        //     ctx.accounts.from.key,
        //     ctx.accounts.to.key,
        //     lamports,
        //     space,
        //     owner,
        // );
        // solana_program::program::invoke_signed(
        //     &ix,
        //     &[ctx.accounts.from, ctx.accounts.to],
        //     ctx.signer_seeds,
        // )
        // .map_err(Into::into)
        Ok(())
    }

    pub fn transfer<'a, 'b, 'c, 'info>(
        ctx: CpiContext<'a, 'b, 'c, 'info, Transfer<'info>>,
        lamports: u64,
    ) -> Result<()> {
        **ctx.accounts.from.try_borrow_mut_lamports()? -= lamports;
        **ctx.accounts.to.try_borrow_mut_lamports()? += lamports;
        // let ix = crate::solana_program::system_instruction::transfer(
        //     ctx.accounts.from.key,
        //     ctx.accounts.to.key,
        //     lamports,
        // );
        // crate::solana_program::program::invoke_signed(
        //     &ix,
        //     &[ctx.accounts.from, ctx.accounts.to],
        //     ctx.signer_seeds,
        // )
        // .map_err(Into::into)
        Ok(())
    }
}

#[cfg(not(target_arch = "bpf"))]
pub mod solana_program {
    pub use anchor_lang::solana_program::*;

    // anchor_lang::solana_program::program::invoke_signed
    pub mod program {
        pub use anchor_lang::solana_program::program::*;
        use anchor_lang::{
            prelude::AccountInfo,
            solana_program::{entrypoint::ProgramResult, instruction::Instruction},
        };
        pub fn invoke_signed(
            instruction: &Instruction,
            account_infos: &[AccountInfo],
            _signers_seeds: &[&[&[u8]]],
        ) -> ProgramResult {
            anchor_lang::prelude::msg!("anchor_lang::solana_program::program::invoke_signed...");
            // Check that the account RefCells are consistent with the request
            for account_meta in instruction.accounts.iter() {
                for account_info in account_infos.iter() {
                    if account_meta.pubkey == *account_info.key {
                        if account_meta.is_writable {
                            let _ = account_info.try_borrow_mut_lamports()?;
                            let _ = account_info.try_borrow_mut_data()?;
                        } else {
                            let _ = account_info.try_borrow_lamports()?;
                            let _ = account_info.try_borrow_data()?;
                        }
                        break;
                    }
                }
            }

            //invoke_signed_unchecked(instruction, account_infos, signers_seeds)
            Ok(())
        }
    }
    pub mod system_instruction {
        use anchor_lang::prelude::Result;
        pub use anchor_lang::solana_program::system_instruction::*;
        use anchor_lang::{
            prelude::{CpiContext, Pubkey},
            solana_program,
            system_program::{Allocate, CreateAccount},
        };

        pub fn allocate<'a, 'b, 'c, 'info>(
            ctx: CpiContext<'a, 'b, 'c, 'info, Allocate<'info>>,
            space: u64,
        ) -> Result<()> {
            println!("allocate");

            let ix = solana_program::system_instruction::allocate(
                ctx.accounts.account_to_allocate.key,
                space,
            );
            solana_program::program::invoke_signed(
                &ix,
                &[ctx.accounts.account_to_allocate],
                ctx.signer_seeds,
            )
            .map_err(Into::into)
        }

        pub fn create_account_x<'a, 'b, 'c, 'info>(
            ctx: CpiContext<'a, 'b, 'c, 'info, CreateAccount<'info>>,
            lamports: u64,
            space: u64,
            owner: &Pubkey,
        ) -> Result<()> {
            anchor_lang::prelude::msg!("Inside create_account?...");
            let ix = ::anchor_lang::solana_program::system_instruction::create_account(
                ctx.accounts.from.key,
                ctx.accounts.to.key,
                lamports,
                space,
                owner,
            );
            crate::anchor_lang::solana_program::program::invoke_signed(
                &ix,
                &[ctx.accounts.from, ctx.accounts.to],
                ctx.signer_seeds,
            )
            .map_err(Into::into)
        }
    }
}

#[cfg(not(target_arch = "bpf"))]
pub mod prelude {
    use std::str::FromStr;

    use anchor_lang::prelude::Rent as AnchorRent;
    pub use anchor_lang::prelude::*;
    use anchor_lang::solana_program::sysvar::SysvarId;
    use core::result::Result as StdResult;
    use serde::{Deserialize, Serialize};

    use super::TIMESTAMP;

    pub struct Clock {
        pub unix_timestamp: i64,
    }

    impl Clock {
        pub fn get() -> Result<Clock> {
            unsafe {
                let unix_timestamp = TIMESTAMP;
                Ok(Clock { unix_timestamp })
            }
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct Rent {}
    impl Rent {
        pub fn get() -> StdResult<AnchorRent, ProgramError> {
            Ok(AnchorRent::default())
        }

        pub fn size_of() -> usize {
            bincode::serialized_size(&Self::default()).unwrap() as usize
        }

        pub fn from_account_info(
            _account_info: &AccountInfo,
        ) -> StdResult<AnchorRent, ProgramError> {
            Rent::get()
        }

        pub fn to_account_info(&self, account_info: &mut AccountInfo) -> Option<()> {
            bincode::serialize_into(&mut account_info.data.borrow_mut()[..], self).ok()
        }

        pub fn default() -> AnchorRent {
            AnchorRent::default()
        }
    }
    impl SysvarId for Rent {
        fn id() -> Pubkey {
            Pubkey::from_str("SysvarRent111111111111111111111111111111111").unwrap()
        }
        fn check_id(_: &Pubkey) -> bool {
            true
        }
    }

    // impl<'a, 'b> anchor_lang::prelude::SolanaSysvar for Rent {}
}
