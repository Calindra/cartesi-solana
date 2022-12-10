use std::{
    fs,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use anchor_lang::{
    prelude::{AccountInfo, Pubkey},
    solana_program::{
        entrypoint::ProgramResult,
        instruction::CompiledInstruction,
        message::{Message, MessageHeader},
    },
};

use cartesi_solana::{
    adapter::{call_smart_contract_base64, eth_address_to_pubkey, parse_processor_args, persist_accounts},
    owner_manager, transaction, account_manager,
};

use cartesi_solana::{anchor_lang::solana_program::hash::Hash, transaction::Signature};

fn setup() {
    println!("\n\n***** setup *****\n");
    let dir = std::env::temp_dir();
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let final_temp_dir = format!(
        "{}/{}",
        dir.as_os_str().to_str().unwrap(),
        since_the_epoch.subsec_nanos()
    );
    println!("{}", final_temp_dir);
    fs::create_dir(&final_temp_dir).unwrap();
    std::env::set_var("SOLANA_DATA_PATH", final_temp_dir);
    std::env::set_var(
        "PORTAL_ADDRESS",
        "0xf8c694fd58360de278d5ff2276b7130bfdc0192a",
    );
    unsafe {
        owner_manager::POINTERS.clear();
        owner_manager::OWNERS.clear();
    }
    account_manager::clear();
}

#[test]
fn it_should_convert_eth_address_to_public_key() {
    // We implemented the same front behavior
    let bytes = hex::decode("f39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap();
    let pubkey = eth_address_to_pubkey(&bytes);
    assert_eq!(
        pubkey.to_string(),
        "1111111111112RXi1yn6kTp7G8Td7o6z3Ciqw9v2"
    );
}

#[test]
fn it_should_call_the_solana_program_entry() {
    setup();

    let signature: Signature = bincode::deserialize(&[0; 64]).unwrap();
    let signatures = vec![signature];
    let header = MessageHeader {
        num_required_signatures: 1,
        num_readonly_signed_accounts: 0,
        num_readonly_unsigned_accounts: 5,
    };
    let account_keys = vec![
        "1111111111112RXi1yn6kTp7G8Td7o6z3Ciqw9v2",
        "6Tw6Z6SsM3ypmGsB3vpSx8midhhyTvTwdPd7K413LyyY",
        "11111111111111111111111111111111",
        "2QB8wEBJ8jjMQuZPvj3jaZP7JJb5j21u4xbxTnwsZRfv",
        "4xRtyUw1QSVZSGi1BUb7nbYBk8TC9P1K1AE2xtxwaZmV",
        "SysvarRent111111111111111111111111111111111",
        "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
    ]
    .into_iter()
    .map(|str_pk| Pubkey::from_str(str_pk).unwrap())
    .collect();

    let recent_blockhash = Hash::default();

    let instruction = CompiledInstruction {
        program_id_index: 3,
        accounts: [1, 0, 4, 6, 2, 5].to_vec(),
        data: vec![141, 132, 233, 130, 168, 183, 10, 119],
    };
    let instructions = vec![instruction];

    let message = Message {
        header,
        account_keys,
        recent_blockhash,
        instructions,
    };
    let transaction = transaction::Transaction {
        signatures,
        message,
    };
    let transaction_bytes = bincode::serialize(&transaction).unwrap();
    let base64_encoded = base64::encode(transaction_bytes);
    println!("base64 = {}", base64_encoded);

    let payload = &base64_encoded;
    let msg_sender = "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266";
    fn prog_entry(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
        assert_eq!(
            program_id.to_string(),
            "2QB8wEBJ8jjMQuZPvj3jaZP7JJb5j21u4xbxTnwsZRfv"
        );
        assert_eq!(accounts.len(), 6);
        assert_eq!(data, &[141, 132, 233, 130, 168, 183, 10, 119]);
        Ok(())
    }
    let instruction_index = 0;

    call_smart_contract_base64(payload, msg_sender, instruction_index, prog_entry);
}

#[test]
fn it_should_parse_args() {
    setup();

    let signature: Signature = bincode::deserialize(&[0; 64]).unwrap();
    let signatures = vec![signature];
    let header = MessageHeader {
        num_required_signatures: 1,
        num_readonly_signed_accounts: 0,
        num_readonly_unsigned_accounts: 5,
    };
    let account_keys = vec![
        "1111111111112RXi1yn6kTp7G8Td7o6z3Ciqw9v2",
        "6Tw6Z6SsM3ypmGsB3vpSx8midhhyTvTwdPd7K413LyyY",
        "11111111111111111111111111111111",
        "2QB8wEBJ8jjMQuZPvj3jaZP7JJb5j21u4xbxTnwsZRfv",
        "4xRtyUw1QSVZSGi1BUb7nbYBk8TC9P1K1AE2xtxwaZmV",
        "SysvarRent111111111111111111111111111111111",
        "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
    ]
    .into_iter()
    .map(|str_pk| Pubkey::from_str(str_pk).unwrap())
    .collect();

    let recent_blockhash = Hash::default();

    let instruction = CompiledInstruction {
        program_id_index: 3,
        accounts: [1, 0, 4, 6, 2, 5].to_vec(),
        data: vec![141, 132, 233, 130, 168, 183, 10, 119],
    };
    let instructions = vec![instruction];

    let message = Message {
        header,
        account_keys,
        recent_blockhash,
        instructions,
    };
    let transaction = transaction::Transaction {
        signatures,
        message,
    };
    let transaction_bytes = bincode::serialize(&transaction).unwrap();
    let base64_encoded = base64::encode(transaction_bytes);
    println!("base64 = {}", base64_encoded);

    let payload = &base64_encoded;
    let msg_sender = "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266";
    fn entry<'a>(program_id: &'a Pubkey, accounts: &'a [AccountInfo<'a>], data: &[u8]) -> ProgramResult {
        assert_eq!(
            program_id.to_string(),
            "2QB8wEBJ8jjMQuZPvj3jaZP7JJb5j21u4xbxTnwsZRfv"
        );
        assert_eq!(accounts.len(), 6);
        assert_eq!(data, &[141, 132, 233, 130, 168, 183, 10, 119]);
        Ok(())
    }
    let instruction_index = 0;

    let (program_id, accounts, data) = parse_processor_args(payload, msg_sender, instruction_index);
    entry(&program_id, &accounts, &data).unwrap();
}


#[test]
fn it_should_change_the_owner_inside_entry() {
    setup();

    let signature: Signature = bincode::deserialize(&[0; 64]).unwrap();
    let signatures = vec![signature];
    let header = MessageHeader {
        num_required_signatures: 1,
        num_readonly_signed_accounts: 0,
        num_readonly_unsigned_accounts: 5,
    };
    let account_keys = vec![
        "1111111111112RXi1yn6kTp7G8Td7o6z3Ciqw9v2",
        "6Tw6Z6SsM3ypmGsB3vpSx8midhhyTvTwdPd7K413LyyY",
        "11111111111111111111111111111111",
        "2QB8wEBJ8jjMQuZPvj3jaZP7JJb5j21u4xbxTnwsZRfv",
        "4xRtyUw1QSVZSGi1BUb7nbYBk8TC9P1K1AE2xtxwaZmV",
        "SysvarRent111111111111111111111111111111111",
        "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
    ]
    .into_iter()
    .map(|str_pk| Pubkey::from_str(str_pk).unwrap())
    .collect();

    let recent_blockhash = Hash::default();

    let instruction = CompiledInstruction {
        program_id_index: 3,
        accounts: [1, 0, 4, 6, 2, 5].to_vec(),
        data: vec![141, 132, 233, 130, 168, 183, 10, 119],
    };
    let instructions = vec![instruction];

    let message = Message {
        header,
        account_keys,
        recent_blockhash,
        instructions,
    };
    let transaction = transaction::Transaction {
        signatures,
        message,
    };
    let transaction_bytes = bincode::serialize(&transaction).unwrap();
    let base64_encoded = base64::encode(transaction_bytes);
    println!("base64 = {}", base64_encoded);

    let payload = &base64_encoded;
    let msg_sender = "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266";
    fn entry<'a>(program_id: &'a Pubkey, accounts: &'a [AccountInfo<'a>], data: &[u8]) -> ProgramResult {
        assert_eq!(
            program_id.to_string(),
            "2QB8wEBJ8jjMQuZPvj3jaZP7JJb5j21u4xbxTnwsZRfv"
        );
        assert_eq!(accounts.len(), 6);
        assert_eq!(data, &[141, 132, 233, 130, 168, 183, 10, 119]);
        let new_owner = Pubkey::from_str("97cRDQwrhrfvrWkjNgZ9JVAv9iMuBLU5igYFPmZ8vPhw").unwrap();
        owner_manager::change_owner(accounts[0].key.to_owned(), new_owner);
        Ok(())
    }
    let instruction_index = 0;

    let (program_id, accounts, data) = parse_processor_args(payload, msg_sender, instruction_index);
    entry(&program_id, &accounts, &data).unwrap();
    assert!(accounts[0].owner.to_string() == "97cRDQwrhrfvrWkjNgZ9JVAv9iMuBLU5igYFPmZ8vPhw");
    persist_accounts(&accounts);
}
