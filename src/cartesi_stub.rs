use std::{path::Path, process::{Child, Command, Stdio}, io::Write};

use anchor_lang::{solana_program::{instruction::Instruction, program_stubs::SyscallStubs, stake_history::Epoch}, prelude::{AccountInfo, Pubkey, ProgramError}};
use serde::{Deserialize, Serialize};

use crate::anchor_lang::TIMESTAMP;


#[derive(Serialize, Deserialize, Clone)]
struct AccountInfoSerialize {
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


pub struct CartesiStubs {}
impl SyscallStubs for CartesiStubs {
    fn sol_set_return_data(&self, data: &[u8]) {
        todo!("set_return_data");
    }

    fn sol_get_return_data(&self) -> Option<(Pubkey, Vec<u8>)> {
        todo!("sol_get_return_data");
        None
    }

    fn sol_invoke_signed(
        &self,
        instruction: &Instruction,
        account_infos: &[AccountInfo], // chaves publicas
        signers_seeds: &[&[&[u8]]],
    ) -> Result<(), ProgramError> {
        // @todo validate signers_seeds
        let mut child = execute_spawn(instruction.program_id.to_string());
        let child_stdin = child.stdin.as_mut().unwrap();

        let instruction = bincode::serialize(&instruction).unwrap();
        let instruction = base64::encode(&instruction);

        let account_infos: Vec<AccountInfoSerialize> = account_infos
            .into_iter()
            .map(|account| AccountInfoSerialize {
                key: account.key.to_owned(),
                is_signer: account.is_signer,
                is_writable: account.is_writable,
                owner: account.owner.to_owned(),
                lamports: account.lamports.borrow_mut().to_owned(),
                data: account.data.borrow_mut().to_vec(),
                executable: account.executable,
                rent_epoch: account.rent_epoch,
            })
            .collect();

        let account_infos = bincode::serialize(&account_infos).unwrap();
        let account_infos = base64::encode(&account_infos);

        let signers_seeds = bincode::serialize(&signers_seeds).unwrap();
        let signers_seeds = base64::encode(&signers_seeds);

        child_stdin.write_all(b"Header: CPI")?;
        child_stdin.write_all(b"\n")?;

        child_stdin.write_all(instruction.as_bytes())?;
        child_stdin.write_all(b"\n")?;
        child_stdin.write_all(account_infos.as_bytes())?;
        child_stdin.write_all(b"\n")?;
        child_stdin.write_all(signers_seeds.as_bytes())?;
        child_stdin.write_all(b"\n")?;

        unsafe {
            child_stdin.write_all(TIMESTAMP.to_string().as_bytes())?;
        }
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
                    println!("Program exited with success code");
                } else {
                    println!("Program exited with error code: {}", code);
                    return Err(ProgramError::Custom(1));
                }
            }
        }

        Ok(())
    }
}
