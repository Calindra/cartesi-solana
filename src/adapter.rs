use crate::account_manager::{self, create_account_info, create_account_manager, AccountFileData, set_data};
use crate::transaction::Signature;
use crate::{owner_manager, transaction};
use anchor_lang::prelude::Pubkey;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::{prelude::AccountInfo, solana_program::entrypoint::ProgramResult};
use serde::{Deserialize, Serialize};
use std::io;
use std::str::FromStr;

pub fn eth_address_to_pubkey(eth_address: &[u8]) -> Pubkey {
    assert!(
        eth_address.len() == 20,
        "Ethereum address must have 20 bytes len"
    );
    let mut bytes = eth_address.to_vec();
    let mut zeroes = vec![0; 12];
    bytes.append(&mut zeroes);
    bytes.reverse();
    Pubkey::new(&bytes)
}

fn get_read_line() -> Vec<u8> {
    let mut line = String::new();
    io::stdin().read_line(&mut line).unwrap();
    base64::decode(&line.trim()).unwrap()
}

fn get_processor_args_from_cpi<'a>() -> (Pubkey, Vec<AccountInfo<'a>>, Vec<u8>, bool) {
    let instruction = get_read_line();
    let accounts = get_read_line();
    let signers_seed = get_read_line();

    let mut timestamp = String::new();
    io::stdin().read_line(&mut timestamp).unwrap();

    let timestamp: i64 = timestamp
        .trim()
        .parse()
        .expect("Timestamp is not an integer");
    unsafe {
        crate::anchor_lang::TIMESTAMP = timestamp;
    }

    // todo validate signers_seed
    let signers_seed: Vec<Vec<Vec<u8>>> = bincode::deserialize(&signers_seed).unwrap();

    let instruction: Instruction = bincode::deserialize(&instruction).unwrap();
    let accounts: Vec<AccountInfoSerialize> = bincode::deserialize(&accounts).unwrap();
    let pubkeys: Vec<Pubkey> = instruction.accounts.iter().map(|acc| acc.pubkey).collect();
    println!("CPI accounts: {:?}", pubkeys);
    let accounts: Vec<AccountInfo<'a>> = accounts
        .iter()
        .map(|account| {
            create_account_info(
                &account.key,
                account.is_signer,
                account.is_writable,
                account.lamports,
                account.data[..].to_vec(),
                account.owner,
                account.executable,
            )
        })
        .collect();

    let mut ordered_accounts = vec![];
    for key in pubkeys.iter() {
        let account_item = accounts.iter().find(|acc| acc.key == key);
        match account_item {
            Some(account_info) => ordered_accounts.push(account_info.to_owned()),
            None => panic!("Account not found {:?}", key),
        }
    }

    let pubkeys_2: Vec<&Pubkey> = ordered_accounts.iter().map(|acc| acc.key).collect();
    println!("CPI accounts[2]: {:?}", pubkeys_2);

    // the addresses changes when you push to vec
    // so we need to get the pointers here, after
    let tot = ordered_accounts.len();
    for j in 0..tot {
        let p: *mut &Pubkey = std::ptr::addr_of_mut!(ordered_accounts[j].owner);
        owner_manager::add_ptr(p as *mut Pubkey, ordered_accounts[j].key.clone());
    }

    (instruction.program_id, ordered_accounts, instruction.data, true)
}

fn get_processor_args_from_external<'a>() -> (Pubkey, Vec<AccountInfo<'a>>, Vec<u8>, bool) {
    let mut msg_sender = String::new();
    io::stdin().read_line(&mut msg_sender).unwrap();
    let mut payload = String::new();
    io::stdin().read_line(&mut payload).unwrap();
    let mut instruction_index = String::new();
    io::stdin().read_line(&mut instruction_index).unwrap();
    let instruction_index: usize = instruction_index
        .trim()
        .parse()
        .expect("Input is not an integer");
    let mut timestamp = String::new();
    io::stdin().read_line(&mut timestamp).unwrap();

    let timestamp: i64 = timestamp
        .trim()
        .parse()
        .expect("Timestamp is not an integer");
    unsafe {
        crate::anchor_lang::TIMESTAMP = timestamp;
    }

    return parse_processor_args(
        &payload[..(&payload.len() - 1)],
        &msg_sender[..(&msg_sender.len() - 1)],
        instruction_index,
    );
}

pub fn get_processor_args<'a>() -> (Pubkey, Vec<AccountInfo<'a>>, Vec<u8>, bool) {
    #[cfg(not(target_arch = "bpf"))]
    {
        let mut header = String::new();
        io::stdin().read_line(&mut header).unwrap();
        println!("header: {}", header);

        match check_header(header.as_str()) {
            SmartContractType::ExternalPi => get_processor_args_from_external(),
            SmartContractType::CPI => get_processor_args_from_cpi(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct AccountInfoSerialize {
    pub key: Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
    pub lamports: u64,
    pub data: Vec<u8>,
    pub owner: Pubkey,
    pub executable: bool,
    pub rent_epoch: u64,
}

type SolanaEntrypoint = fn(&Pubkey, &[AccountInfo], &[u8]) -> ProgramResult;

fn call_smart_contract_cpi(solana_program_entrypoint: SolanaEntrypoint) -> io::Result<()> {
    let (program_id, accounts, data, last_instruction) = get_processor_args_from_cpi();
    let resp = solana_program_entrypoint(&program_id, &accounts, &data);
    resp.unwrap();
    // @todo maybe remove last_instruction
    persist_accounts(&accounts, last_instruction);

    Ok(())
}

enum SmartContractType {
    ExternalPi,
    CPI,
}

fn check_header(header: &str) -> SmartContractType {
    let header = header.trim();

    if header == "Header: External CPI" {
        SmartContractType::ExternalPi
    } else if header == "Header: CPI" {
        SmartContractType::CPI
    } else {
        panic!("Invalid header [{}]", header);
    }
}

pub fn call_solana_cpi(entry: SolanaEntrypoint) -> io::Result<()> {
    call_smart_contract_cpi(entry).unwrap();

    Ok(())
}

fn call_solana_program_external(entry: SolanaEntrypoint) -> io::Result<()> {
    let mut msg_sender = String::new();
    io::stdin().read_line(&mut msg_sender)?;

    let mut payload = String::new();
    io::stdin().read_line(&mut payload)?;

    let mut instruction_index = String::new();
    io::stdin().read_line(&mut instruction_index)?;
    let instruction_index: usize = instruction_index
        .trim()
        .parse()
        .expect("Input is not an integer");
    let mut timestamp = String::new();
    io::stdin().read_line(&mut timestamp)?;

    let timestamp: i64 = timestamp
        .trim()
        .parse()
        .expect("Timestamp is not an integer");
    unsafe {
        crate::anchor_lang::TIMESTAMP = timestamp;
    }

    call_smart_contract_base64(
        &payload[..(&payload.len() - 1)],
        &msg_sender[..(&msg_sender.len() - 1)],
        instruction_index,
        entry,
    );

    Ok(())
}

pub fn call_solana_program(entry: SolanaEntrypoint) -> io::Result<()> {
    #[cfg(not(target_arch = "bpf"))]
    {
        let mut header = String::new();
        io::stdin().read_line(&mut header)?;

        match check_header(&header) {
            SmartContractType::CPI => {
                call_solana_cpi(entry)?;
            }
            SmartContractType::ExternalPi => {
                call_solana_program_external(entry)?;
            }
        }
    }
    Ok(())
}

fn load_account_info_data(pubkey: &Pubkey) -> (Vec<u8>, u64, Pubkey) {
    let account_manager = create_account_manager();
    let read_account_data_file = account_manager.read_account(&pubkey);
    match read_account_data_file {
        Ok(account_data_file) => {
            return (
                account_data_file.data,
                account_data_file.lamports,
                account_data_file.owner,
            )
        }
        Err(_) => {
            let lamports = 0;
            let info_data = vec![];
            let owner = Pubkey::from_str("11111111111111111111111111111111").unwrap();
            return (info_data, lamports, owner);
        }
    };
}

#[derive(Serialize, Deserialize)]
pub struct AccountJson {
    key: String,
    owner: String,
    data: String,
    lamports: String,
}

fn check_signature(key: &Pubkey, sender_bytes: &[u8], _signature: &Signature) -> bool {
    sender_bytes == &key.to_bytes()[12..]
}

pub fn parse_processor_args<'a>(
    payload: &str,
    msg_sender: &str,
    instruction_index: usize,
) -> (Pubkey, Vec<AccountInfo<'a>>, Vec<u8>, bool) {
    println!("sender => {:?}", msg_sender);
    println!("payload => {:?}", payload);
    println!("instruction_index => {:?}", instruction_index);
    let decoded = base64::decode(payload).unwrap();
    let tx: transaction::Transaction = bincode::deserialize(&decoded).unwrap();
    let sender_bytes: Vec<u8> = hex::decode(&msg_sender[2..])
        .unwrap()
        .into_iter()
        .rev()
        .collect();
    let tx_instruction = &tx.message.instructions[instruction_index];
    let last_instruction = instruction_index == &tx.message.instructions.len() - 1;
    let mut accounts: Vec<AccountInfo> = vec![];
    for key in tx.message.account_keys.iter() {
        let (data, lamports, owner) = load_account_info_data(&key);
        create_account_info(key, true, true, lamports, data, owner, true);
    }
    account_manager::clear();
    let pidx: usize = (tx_instruction.program_id_index).into();
    let program_id: &Pubkey = &tx.message.account_keys[pidx];
    for (i, key) in tx.message.account_keys.iter().enumerate() {
        let (data, lamports, owner) = load_account_info_data(&key);
        println!("loading account with key = {:?}; data.len() = {}; program_id = {:?}", &key, data.len(), program_id);
        let mut is_signer = false;
        if tx.signatures.len() > i {
            let signature = &tx.signatures[i];
            is_signer = check_signature(&key, &sender_bytes, &signature);
        }
        let is_writable = true; // todo
        let executable = true;
        let account_info = create_account_info(
            key,
            is_signer,
            is_writable,
            lamports,
            data,
            owner,
            executable,
        );
        accounts.push(account_info.to_owned());
    }
    for (i, key) in tx.message.account_keys.iter().enumerate() {
        assert_eq!(key, accounts[i].key);
    }
    
    let mut ordered_accounts: Vec<AccountInfo> = Vec::new();
    let tot = tx_instruction.accounts.len();
    for j in 0..tot {
        let index = tx_instruction.accounts[j];
        let i: usize = index.into();
        ordered_accounts.push(accounts[i].to_owned());
    }

    // the addresses changes when you push to vec
    // so we need to get the pointers here, after
    for j in 0..tot {
        let p: *mut &Pubkey = std::ptr::addr_of_mut!(ordered_accounts[j].owner);
        owner_manager::add_ptr(p as *mut Pubkey, ordered_accounts[j].key.clone());
    }

    for acc in ordered_accounts.iter() {
        println!("- ordered_accounts = {:?}", acc.key);
        println!("     owner = {:?}", acc.owner.to_string());
    }
    println!("last_instruction = {}; {}/{}", last_instruction, instruction_index + 1, tx.message.instructions.len());
    (
        program_id.to_owned(),
        ordered_accounts,
        tx_instruction.data.to_owned(),
        last_instruction,
    )
}

pub fn persist_accounts(accounts: &[AccountInfo], delete: bool) {
    let account_manager = create_account_manager();
    for acc in accounts.iter() {
        let data = acc.data.borrow_mut();
        let lamports: u64 = **acc.lamports.borrow_mut();
        let account_file_data = AccountFileData {
            owner: acc.owner.to_owned(),
            data: data.to_vec(),
            lamports,
        };
        println!("should delete = {}", delete);
        if delete && lamports <= 0 {
            account_manager.delete_account(&acc.key).unwrap();
            println!("! deleted = {:?}", acc.key);
        } else {
            account_manager
                .write_account(&acc.key, &account_file_data)
                .unwrap();
            println!("   saved = {:?};", acc.key);
            println!("     owner = {:?}", acc.owner.to_string());
        }
    }
}

pub fn call_smart_contract_base64(
    payload: &str,
    msg_sender: &str,
    instruction_index: usize,
    solana_program_entrypoint: fn(&Pubkey, &[AccountInfo], &[u8]) -> ProgramResult,
) {
    let (program_id, accounts, data, last_instruction) =
        parse_processor_args(payload, msg_sender, instruction_index);
    let resp = solana_program_entrypoint(&program_id, &accounts, &data);
    resp.unwrap();
    // match resp {
    //     Ok(_) => {
    //         println!("Success!");
    //     }
    //     Err(_) => {
    //         println!("Error: Something is not right! Handle any errors plz");
    //     }
    // }
    println!("Persist {:?} accounts...", program_id);
    persist_accounts(&accounts, last_instruction);
}
