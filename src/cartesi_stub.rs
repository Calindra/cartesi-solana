use serde::{Deserialize, Serialize};
use solana_program::{self, pubkey::Pubkey, stake_history::Epoch};

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

#[cfg(not(target_arch = "bpf"))]
fn execute_spawn(program_id: String) -> std::process::Child {
    let path = std::path::Path::new(&crate::adapter::get_binary_base_path()).join(&program_id);

    if !path.exists() {
        panic!("failed to find program path [{}]", path.display());
    }

    let exec = std::process::Command::new(&path)
        .stdin(std::process::Stdio::piped())
        .spawn();

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

#[cfg(not(target_arch = "bpf"))]
fn refresh_accounts(accounts: &[solana_program::account_info::AccountInfo]) -> Result<(), solana_program::program_error::ProgramError> {
    let account_manager = crate::account_manager::create_account_manager();
    println!("refresh_accounts {:?}", account_manager);

    for account_info in accounts.iter() {
        let account_data = account_manager.read_account(account_info.key);

        match account_data {
            Ok(account_data) => {
                println!(
                    "  refresh key {:?}; data.len() = {}",
                    account_info.key,
                    account_data.data.len()
                );
                crate::account_manager::set_data(account_info, account_data.data);
                **account_info.try_borrow_mut_lamports()? = account_data.lamports;
                if account_info.owner != &account_data.owner {
                    crate::owner_manager::change_owner(
                        account_info.key.clone(),
                        account_data.owner.to_owned(),
                    );
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

#[cfg(not(target_arch = "bpf"))]
impl solana_program::program_stubs::SyscallStubs for CartesiStubs {
    fn sol_set_return_data(&self, data: &[u8]) {
        let path = std::path::Path::new(&crate::adapter::get_binary_base_path()).join("return_data.out");

        let program_id = self.program_id.to_string();

        let data = base64::encode(data);

        std::fs::write(&path, format!("{}\n{}", program_id, data))
            .expect(&format!("failed to write return data file: [{:?}]", &path));
    }

    fn sol_get_return_data(&self) -> Option<(Pubkey, Vec<u8>)> {
        let path = std::path::Path::new(&crate::adapter::get_binary_base_path()).join("return_data.out");

        if !path.exists() {
            return None;
        }

        let bundle = std::fs::read_to_string(path).expect("failed to read return data file");

        let mut bundle = bundle.split("\n");

        let program_id = bundle.next().unwrap();
        let program_id = <Pubkey as std::str::FromStr>::from_str(&program_id).unwrap();

        let data = bundle.next().unwrap();
        let data = base64::decode(data).unwrap();

        Some((program_id, data))
    }

    fn sol_invoke_signed(
        &self,
        instruction: &solana_program::instruction::Instruction,
        account_infos: &[solana_program::account_info::AccountInfo], // chaves publicas
        signers_seeds: &[&[&[u8]]],
    ) -> Result<(), solana_program::program_error::ProgramError> {
        crate::cpi::check_signature(&self.program_id, instruction, signers_seeds);

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

        write_all(child_stdin, b"Header: CPI")?;
        write_all(child_stdin, b"\n")?;

        write_all(child_stdin, instruction.as_bytes())?;
        write_all(child_stdin, b"\n")?;
        write_all(child_stdin, account_infos_serialized.as_bytes())?;
        write_all(child_stdin, b"\n")?;
        write_all(child_stdin, signers_seeds.as_bytes())?;
        write_all(child_stdin, b"\n")?;

        write_all(child_stdin, crate::adapter::get_timestamp().to_string().as_bytes())?;

        write_all(child_stdin, b"\n")?;

        write_all(child_stdin, program_id_serialized.as_bytes())?;
        write_all(child_stdin, b"\n")?;

        drop(child_stdin);

        let output = child.wait_with_output()?;
        println!("output: {:?}", output);

        let exit_code = output.status.code();

        match exit_code {
            None => {
                println!("Program failed to run");
                return Err(solana_program::program_error::ProgramError::Custom(1));
            }
            Some(code) => {
                if code == 0 {
                    refresh_accounts(account_infos)?;

                    println!("Program exited with success code");
                } else {
                    println!("Program exited with error code: {}", code);
                    return Err(solana_program::program_error::ProgramError::Custom(1));
                }
            }
        }

        Ok(())
    }

    fn sol_get_rent_sysvar(&self, var_addr: *mut u8) -> u64 {
        unsafe {
            *(var_addr as *mut _ as *mut solana_program::rent::Rent) =
                solana_program::rent::Rent::default();
        }
        solana_program::entrypoint::SUCCESS
    }

    fn sol_get_clock_sysvar(&self, var_addr: *mut u8) -> u64 {
        unsafe {
            *(var_addr as *mut _ as *mut solana_program::clock::Clock) =
                solana_program::clock::Clock {
                    slot: 1,
                    epoch_start_timestamp: crate::adapter::get_timestamp(),
                    epoch: 1,
                    leader_schedule_epoch: 1,
                    unix_timestamp: crate::adapter::get_timestamp(),
                };
        }
        solana_program::entrypoint::SUCCESS
    }
}

#[cfg(not(target_arch = "bpf"))]
fn write_all(child_stdin: &mut std::process::ChildStdin, buf: &[u8]) -> Result<(), solana_program::program_error::ProgramError> {
    std::io::Write::write_all(child_stdin, buf)?;
    Ok(())
}
