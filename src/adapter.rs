use crate::account_manager::{create_account_manager, AccountFileData};
use crate::transaction::Signature;
use crate::{owner_manager, transaction};
use anchor_lang::prelude::Pubkey;
use anchor_lang::{prelude::AccountInfo, solana_program::entrypoint::ProgramResult};
use serde::{Deserialize, Serialize};
use std::io;
use std::{cell::RefCell, rc::Rc, str::FromStr};

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

pub fn call_solana_program(
    entry: fn(&Pubkey, &[AccountInfo], &[u8]) -> ProgramResult,
) -> io::Result<()> {
    #[cfg(not(target_arch = "bpf"))]
    {
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

fn check_signature(pubkey: &Pubkey, sender_bytes: &[u8], _signature: &Signature) -> bool {
    sender_bytes == &pubkey.to_bytes()[12..]
}

pub fn call_smart_contract_base64(
    payload: &str,
    msg_sender: &str,
    instruction_index: usize,
    entry: fn(&Pubkey, &[AccountInfo], &[u8]) -> ProgramResult,
) {
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
    let mut accounts = Vec::new();
    let mut params = Vec::new();
    let mut i = 0;
    for pubkey in tx.message.account_keys.iter() {
        println!("loading pk = {:?}", pubkey);
        let (a, b, c) = load_account_info_data(&pubkey);
        let mut is_signer = false;
        if tx.signatures.len() > i {
            let signature = &tx.signatures[i];
            is_signer = check_signature(&pubkey, &sender_bytes, &signature);
        }
        params.push((a, b, c, pubkey, is_signer));
        i = i + 1;
    }
    for param in params.iter_mut() {
        let key = &param.3;
        let is_signer = &param.4;
        let is_writable = true;
        let executable = true;
        let account_info = AccountInfo {
            key,
            is_signer: *is_signer,
            is_writable,
            lamports: Rc::new(RefCell::new(&mut param.1)),
            data: Rc::new(RefCell::new(&mut param.0)),
            owner: &param.2,
            executable,
            rent_epoch: 1,
        };
        accounts.push(account_info);
    }
    let pidx: usize = (tx_instruction.program_id_index).into();
    println!(
        "tx_instruction.program_id_index = {:?}",
        tx_instruction.program_id_index
    );
    let program_id = accounts[pidx].key;
    println!("tx_instruction.program_id = {:?}", accounts[pidx].key);
    println!(
        "tx.message.header.num_required_signatures = {:?}",
        tx.message.header.num_required_signatures
    );
    println!(
        "tx.message.header.num_readonly_signed_accounts = {:?}",
        tx.message.header.num_readonly_signed_accounts
    );
    println!(
        "tx.message.header.num_readonly_unsigned_accounts = {:?}",
        tx.message.header.num_readonly_unsigned_accounts
    );
    println!("signatures.len() = {:?}", tx.signatures.len());
    println!("accounts indexes = {:?}", tx_instruction.accounts);
    println!(
        "method dispatch's sighash = {:?}",
        &tx_instruction.data[..8]
    );
    let mut ordered_accounts = Vec::new();
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

    let resp = entry(&program_id, &ordered_accounts, &tx_instruction.data);

    resp.unwrap();
    // match resp {
    //     Ok(_) => {
    //         println!("Success!");
    //     }
    //     Err(_) => {
    //         println!("Error: Something is not right! Handle any errors plz");
    //     }
    // }
    let account_manager = create_account_manager();
    for acc in ordered_accounts.iter() {
        let data = acc.data.borrow_mut();
        let lamports: u64 = **acc.lamports.borrow_mut();
        let account_file_data = AccountFileData {
            owner: acc.owner.to_owned(),
            data: data.to_vec(),
            lamports: lamports,
        };
        if lamports <= 0 {
            account_manager.delete_account(&acc.key).unwrap();
            println!("! deleted = {:?}", acc.key);
        } else {
            account_manager
                .write_account(&acc.key, &account_file_data)
                .unwrap();
            println!("   saved = {:?}", acc.key);
            println!("     owner = {:?}", acc.owner.to_string());
        }
    }
}
