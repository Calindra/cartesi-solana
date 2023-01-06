use std::{
    fs::{self},
    io::Write,
    path::Path,
    process::{Child, Command, Stdio},
    str::FromStr,
};

use anchor_lang::{
    prelude::{AccountInfo, ProgramError, Pubkey, Clock},
    solana_program::{instruction::Instruction, program_stubs::SyscallStubs, stake_history::Epoch},
};
use serde::{Deserialize, Serialize};

use crate::{
    account_manager::{create_account_manager, set_data},
    anchor_lang::{TIMESTAMP, solana_program, prelude::Rent}, owner_manager, cpi,
};

#[derive(Serialize, Deserialize, Clone)]
pub struct AccountInfoSerialize {
    pub key: Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
    pub lamports: u64,
    pub data: Vec<u8>,
    pub owner: Pubkey,
    pub executable: bool,
    pub rent_epoch: Epoch,
}

fn execute_spawn(program_id: String) -> Child {
    let path = Path::new("./solana_smart_contract_bin/").join(&program_id);

    if !path.exists() {
        panic!("failed to find program path [{}]", path.display());
    }

    let exec = Command::new(&path).stdin(Stdio::piped()).spawn();

    match exec {
        Ok(child) => child,
        Err(e) => panic!(
            "failed to execute process: {} with program_id [{}], path [{}]",
            e,
            program_id,
            path.display()
        ),
    }
}

fn refresh_accounts(accounts: &[AccountInfo]) -> Result<(), ProgramError> {
    let account_manager = create_account_manager();
    println!("refresh_accounts {:?}", account_manager);

    for account_info in accounts.iter() {
        let account_data = account_manager.read_account(account_info.key);

        match account_data {
            Ok(account_data) => {
                println!("  refresh key {:?}; data.len() = {}", account_info.key, account_data.data.len());
                set_data(account_info, account_data.data);
                **account_info.try_borrow_mut_lamports()? = account_data.lamports;
                if account_info.owner != &account_data.owner {
                    owner_manager::change_owner(account_info.key.clone(), account_data.owner.to_owned());
                }
            }
            Err(_) => {
                println!("refresh_accounts account not found {:?}", account_info.key);
            }
        }
    }

    Ok(())
}

pub struct CartesiStubs {
    pub program_id: Pubkey,
}
impl SyscallStubs for CartesiStubs {
    fn sol_set_return_data(&self, data: &[u8]) {
        // let account_manager = create_account_manager();

        let path = Path::new("./solana_smart_contract_bin/").join("return_data.out");

        let program_id = self.program_id.to_string();

        let data = base64::encode(data);

        fs::write(path, format!("{}\n{}", program_id, data))
            .expect("failed to write return data file");
    }

    fn sol_get_return_data(&self) -> Option<(Pubkey, Vec<u8>)> {
        let path = Path::new("./solana_smart_contract_bin/").join("return_data.out");

        if !path.exists() {
            return None;
        }

        let bundle = fs::read_to_string(path).expect("failed to read return data file");

        let mut bundle = bundle.split("\n");

        let program_id = bundle.next().unwrap();
        let program_id = Pubkey::from_str(&program_id).unwrap();

        let data = bundle.next().unwrap();
        let data = base64::decode(data).unwrap();

        Some((program_id, data))
    }

    fn sol_invoke_signed(
        &self,
        instruction: &Instruction,
        account_infos: &[AccountInfo], // chaves publicas
        signers_seeds: &[&[&[u8]]],
    ) -> Result<(), ProgramError> {
        cpi::check_signature(&self.program_id, instruction, signers_seeds);
        
        let mut child = execute_spawn(instruction.program_id.to_string());
        let child_stdin = child.stdin.as_mut().unwrap();

        let instruction = bincode::serialize(&instruction).unwrap();
        let instruction = base64::encode(&instruction);

        let account_infos_serialized: Vec<AccountInfoSerialize> = account_infos
            .into_iter()
            .map(|account| AccountInfoSerialize {
                key: account.key.to_owned(),
                is_signer: account.is_signer,
                is_writable: account.is_writable,
                owner: account.owner.to_owned(),
                lamports: account.lamports.borrow_mut().to_owned(),

                // @todo: verify the serialized data by borsh
                data: account.data.borrow_mut().to_vec(),
                executable: account.executable,
                rent_epoch: account.rent_epoch,
            })
            .collect();

        let account_infos_serialized = bincode::serialize(&account_infos_serialized).unwrap();
        let account_infos_serialized = base64::encode(&account_infos_serialized);
        let program_id_serialized = bincode::serialize(&self.program_id).unwrap();
        let program_id_serialized = base64::encode(program_id_serialized);
        let signers_seeds = bincode::serialize(&signers_seeds).unwrap();
        let signers_seeds = base64::encode(&signers_seeds);

        child_stdin.write_all(b"Header: CPI")?;
        child_stdin.write_all(b"\n")?;

        child_stdin.write_all(instruction.as_bytes())?;
        child_stdin.write_all(b"\n")?;
        child_stdin.write_all(account_infos_serialized.as_bytes())?;
        child_stdin.write_all(b"\n")?;
        child_stdin.write_all(signers_seeds.as_bytes())?;
        child_stdin.write_all(b"\n")?;

        unsafe {
            child_stdin.write_all(TIMESTAMP.to_string().as_bytes())?;
        }
        child_stdin.write_all(b"\n")?;

        child_stdin.write_all(program_id_serialized.as_bytes())?;
        child_stdin.write_all(b"\n")?;

        drop(child_stdin);

        let output = child.wait_with_output()?;
        println!("output: {:?}", output);

        let exit_code = output.status.code();

        match exit_code {
            None => {
                println!("Program failed to run");
                return Err(ProgramError::Custom(1));
            }
            Some(code) => {
                if code == 0 {
                    refresh_accounts(account_infos)?;

                    println!("Program exited with success code");
                } else {
                    println!("Program exited with error code: {}", code);
                    return Err(ProgramError::Custom(1));
                }
            }
        }

        Ok(())
    }

    fn sol_get_rent_sysvar(&self, var_addr: *mut u8) -> u64 {
        unsafe {
            // *(var_addr as *mut _ as *mut Rent) = Rent::default();
            *(var_addr as *mut _ as *mut solana_program::rent::Rent) = Rent::default();
        }
        solana_program::entrypoint::SUCCESS
    }

    fn sol_get_clock_sysvar(&self, var_addr: *mut u8) -> u64 {
        unsafe {
            *(var_addr as *mut _ as *mut Clock) = Clock {
                slot: 1,
                epoch_start_timestamp: crate::anchor_lang::TIMESTAMP,
                epoch: 1,
                leader_schedule_epoch: 1,
                unix_timestamp: crate::anchor_lang::TIMESTAMP,
            };
        }
        solana_program::entrypoint::SUCCESS
    }
}
