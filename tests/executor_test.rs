use anchor_lang::{prelude::{AccountInfo, AccountMeta, Pubkey}, solana_program};
use borsh::BorshSerialize;
use cartesi_solana::{
    account_manager::{self, create_account_manager, AccountFileData},
    adapter::{load_account_info_data, self},
    cartesi_stub::AccountInfoSerialize,
    executor::{DefaultStdin, Executor, LineReader},
    owner_manager,
    transaction::{self, Signature},
};
use solana_sdk::{
    hash::Hash,
    instruction::CompiledInstruction,
    message::{Message, MessageHeader},
};
use std::{
    fmt::Write,
    fs, io,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

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
    owner_manager::clear();
    account_manager::clear();
}

#[test]
fn executor_should_load_program_args() {
    setup();
    let payload = create_payload();
    let stdin = MyLineReader::create(vec![
        "Header: External CPI",
        "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
        &payload,
        "0",     // instruction index
        "12345", // timestamp
    ]);

    let mut executor = Executor::create_with_stdin(stdin);

    create_account_with_space("6Tw6Z6SsM3ypmGsB3vpSx8midhhyTvTwdPd7K413LyyY", 32);

    executor.get_processor_args(|program_id, accounts, data| {
        assert_eq!(
            program_id.to_string(),
            "2QB8wEBJ8jjMQuZPvj3jaZP7JJb5j21u4xbxTnwsZRfv".to_string()
        );
        assert_eq!(accounts.len(), 6);
        assert_eq!(data, &[141, 132, 233, 130, 168, 183, 10, 119]);
        assert_eq!(adapter::get_timestamp(), 12345);
    });
}

#[test]
fn executor_should_call_crazy_lifetime() {
    setup();
    let payload = create_payload();
    let stdin = MyLineReader::create(vec![
        "Header: External CPI",
        "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
        &payload,
        "0",     // instruction index
        "12345", // timestamp
    ]);

    let mut executor = Executor::create_with_stdin(stdin);

    create_account_with_space("6Tw6Z6SsM3ypmGsB3vpSx8midhhyTvTwdPd7K413LyyY", 32);

    executor.get_processor_args(|program_id, accounts, data| {
        process_instruction(&program_id, accounts, &data);
    });
}
fn process_instruction<'a>(
    _program_id: &'a Pubkey,
    _accounts: &'a [AccountInfo<'a>],
    _instruction_data: &[u8],
) {
}

#[test]
fn executor_should_load_change_and_save_account_infos() {
    setup();
    let payload = create_payload();
    let stdin = MyLineReader::create(vec![
        "Header: External CPI",
        "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
        &payload,
        "0",     // instruction index
        "12345", // timestamp
    ]);

    let mut executor = Executor::create_with_stdin(stdin);

    create_account_with_space("6Tw6Z6SsM3ypmGsB3vpSx8midhhyTvTwdPd7K413LyyY", 32);

    executor.get_processor_args(|_program_id, accounts, _data| {
        let borsh_structure = BorshStructure {
            key: Pubkey::from_str("4xRtyUw1QSVZSGi1BUb7nbYBk8TC9P1K1AE2xtxwaZmV").unwrap(),
        };
        let account_info = &accounts[0];
        // let keys: Vec<Pubkey> = accounts.iter().map(|a| a.key.to_owned()).collect();
        // println!("keys = {:?}", keys);
        **account_info.lamports.try_borrow_mut().unwrap() += 100;
        borsh_structure
            .serialize(&mut *account_info.try_borrow_mut_data().unwrap())
            .unwrap();
    });

    let (data, _, _) = load_account_info_data(
        &Pubkey::from_str("6Tw6Z6SsM3ypmGsB3vpSx8midhhyTvTwdPd7K413LyyY").unwrap(),
    );
    let expected = Pubkey::from_str("4xRtyUw1QSVZSGi1BUb7nbYBk8TC9P1K1AE2xtxwaZmV").unwrap();
    assert_eq!(data, expected.to_bytes());
}

#[test]
fn executor_should_change_the_owner() {
    setup();
    let payload = create_payload();
    let stdin = MyLineReader::create(vec![
        "Header: External CPI",
        "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
        &payload,
        "0",     // instruction index
        "12345", // timestamp
    ]);

    let mut executor = Executor::create_with_stdin(stdin);

    executor.get_processor_args(|_program_id, accounts, _data| {
        let borsh_structure = BorshStructure {
            key: Pubkey::from_str("4xRtyUw1QSVZSGi1BUb7nbYBk8TC9P1K1AE2xtxwaZmV").unwrap(),
        };
        let account_info = &accounts[0];
        let new_owner = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap();
        owner_manager::change_owner(*account_info.key, new_owner);
        account_manager::set_data_size(account_info, 32);
        **account_info.lamports.try_borrow_mut().unwrap() += 100;
        borsh_structure
            .serialize(&mut *account_info.try_borrow_mut_data().unwrap())
            .unwrap();
    });

    let (_, _, owner) = load_account_info_data(
        &Pubkey::from_str("6Tw6Z6SsM3ypmGsB3vpSx8midhhyTvTwdPd7K413LyyY").unwrap(),
    );
    let expected = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap();
    assert_eq!(owner, expected);
}

#[test]
fn executor_should_save_account_info_resized() {
    setup();
    let payload = create_payload();
    let stdin = MyLineReader::create(vec![
        "Header: External CPI",
        "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
        &payload,
        "0",     // instruction index
        "12345", // timestamp
    ]);

    let mut executor = Executor::create_with_stdin(stdin);

    executor.get_processor_args(|_program_id, accounts, _data| {
        let borsh_structure = BorshStructure {
            key: Pubkey::from_str("4xRtyUw1QSVZSGi1BUb7nbYBk8TC9P1K1AE2xtxwaZmV").unwrap(),
        };
        let account_info = &accounts[0];
        account_manager::set_data_size(account_info, 32);
        **account_info.lamports.try_borrow_mut().unwrap() += 100;
        borsh_structure
            .serialize(&mut *account_info.try_borrow_mut_data().unwrap())
            .unwrap();
    });

    let (data, _, _) = load_account_info_data(
        &Pubkey::from_str("6Tw6Z6SsM3ypmGsB3vpSx8midhhyTvTwdPd7K413LyyY").unwrap(),
    );
    let expected = Pubkey::from_str("4xRtyUw1QSVZSGi1BUb7nbYBk8TC9P1K1AE2xtxwaZmV").unwrap();
    assert_eq!(data, expected.to_bytes());
}

#[test]
fn executor_with_default_stdin() {
    let stdin = DefaultStdin {};
    Executor::create_with_stdin(stdin);
}

#[test]
fn executor_cpi_read_arguments() {
    setup();
    let payload = create_instruction_payload();
    let cpi_accounts = create_cpi_accounts(0);
    let signers_seeds = create_signers_seeds();
    let caller_program_id = create_cpi_program_id();
    let stdin = MyLineReader::create(vec![
        "Header: CPI",
        &payload,
        &cpi_accounts,
        &signers_seeds,
        "12345", // timestamp
        &caller_program_id,
    ]);
    let mut executor = Executor::create_with_stdin(stdin);
    executor.get_processor_args(|program_id, accounts, data| {
        let spl_token_program_id =
            Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap();
        assert_eq!(program_id, &spl_token_program_id);
        assert_eq!(accounts.len(), 1);
        assert_eq!(data, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 0]);
    });
}

#[test]
fn executor_cpi_save_borsh_serialization() {
    setup();
    let payload = create_instruction_payload();
    let cpi_accounts = create_cpi_accounts(32);
    let signers_seeds = create_signers_seeds();
    let caller_program_id = create_cpi_program_id();
    let stdin = MyLineReader::create(vec![
        "Header: CPI",
        &payload,
        &cpi_accounts,
        &signers_seeds,
        "12345", // timestamp
        &caller_program_id,
    ]);

    let mut executor = Executor::create_with_stdin(stdin);
    executor.get_processor_args(|_program_id, accounts, _data| {
        let borsh_structure = BorshStructure {
            key: Pubkey::from_str("4xRtyUw1QSVZSGi1BUb7nbYBk8TC9P1K1AE2xtxwaZmV").unwrap(),
        };
        let account_info = &accounts[0];
        **account_info.lamports.try_borrow_mut().unwrap() += 100;
        borsh_structure
            .serialize(&mut *account_info.try_borrow_mut_data().unwrap())
            .unwrap();
    });
    let (data, _, _) = load_account_info_data(
        &Pubkey::from_str("6Tw6Z6SsM3ypmGsB3vpSx8midhhyTvTwdPd7K413LyyY").unwrap(),
    );
    let expected = Pubkey::from_str("4xRtyUw1QSVZSGi1BUb7nbYBk8TC9P1K1AE2xtxwaZmV").unwrap();
    assert_eq!(data, expected.to_bytes());
}

#[test]
fn executor_cpi_save_borsh_serialization_with_account_data_resize() {
    setup();
    let payload = create_instruction_payload();
    let cpi_accounts = create_cpi_accounts(0);
    let signers_seeds = create_signers_seeds();
    let caller_program_id = create_cpi_program_id();
    let stdin = MyLineReader::create(vec![
        "Header: CPI",
        &payload,
        &cpi_accounts,
        &signers_seeds,
        "12345", // timestamp
        &caller_program_id,
    ]);

    let mut executor = Executor::create_with_stdin(stdin);
    executor.get_processor_args(|_program_id, accounts, _data| {
        let borsh_structure = BorshStructure {
            key: Pubkey::from_str("4xRtyUw1QSVZSGi1BUb7nbYBk8TC9P1K1AE2xtxwaZmV").unwrap(),
        };
        let account_info = &accounts[0];

        // resize the account data
        account_manager::set_data_size(account_info, 32);
        **account_info.lamports.try_borrow_mut().unwrap() += 100;
        borsh_structure
            .serialize(&mut *account_info.try_borrow_mut_data().unwrap())
            .unwrap();
    });
    let (data, _, _) = load_account_info_data(
        &Pubkey::from_str("6Tw6Z6SsM3ypmGsB3vpSx8midhhyTvTwdPd7K413LyyY").unwrap(),
    );
    let expected = Pubkey::from_str("4xRtyUw1QSVZSGi1BUb7nbYBk8TC9P1K1AE2xtxwaZmV").unwrap();
    assert_eq!(data, expected.to_bytes());
}

#[test]
fn executor_cpi_save_new_owner() {
    setup();
    let payload = create_instruction_payload();
    let cpi_accounts = create_cpi_accounts(0);
    let signers_seeds = create_signers_seeds();
    let caller_program_id = create_cpi_program_id();
    let stdin = MyLineReader::create(vec![
        "Header: CPI",
        &payload,
        &cpi_accounts,
        &signers_seeds,
        "12345", // timestamp
        &caller_program_id,
    ]);

    create_account_with_space("6Tw6Z6SsM3ypmGsB3vpSx8midhhyTvTwdPd7K413LyyY", 32);

    let mut executor = Executor::create_with_stdin(stdin);
    executor.get_processor_args(|_program_id, accounts, _data| {
        let account_info = &accounts[0];
        let new_owner = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap();
        owner_manager::change_owner(*account_info.key, new_owner);
        **account_info.lamports.try_borrow_mut().unwrap() += 1234567;
    });
    let (_, lamports, owner) = load_account_info_data(
        &Pubkey::from_str("6Tw6Z6SsM3ypmGsB3vpSx8midhhyTvTwdPd7K413LyyY").unwrap(),
    );
    let expected = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap();
    assert_eq!(owner, expected);
    assert_eq!(lamports, 1234567);
}

#[test]
fn executor_cpi_save_new_owner_and_serialize() {
    setup();
    let payload = create_instruction_payload();
    let cpi_accounts = create_cpi_accounts(32);
    let signers_seeds = create_signers_seeds();
    let caller_program_id = create_cpi_program_id();
    let stdin = MyLineReader::create(vec![
        "Header: CPI",
        &payload,
        &cpi_accounts,
        &signers_seeds,
        "12345", // timestamp
        &caller_program_id,
    ]);

    create_account_with_space("6Tw6Z6SsM3ypmGsB3vpSx8midhhyTvTwdPd7K413LyyY", 32);

    let mut executor = Executor::create_with_stdin(stdin);
    executor.get_processor_args(|_program_id, accounts, _data| {
        let account_info = &accounts[0];
        let expected_account_key =
            Pubkey::from_str("6Tw6Z6SsM3ypmGsB3vpSx8midhhyTvTwdPd7K413LyyY").unwrap();
        assert_eq!(account_info.key, &expected_account_key);

        let new_owner = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap();
        owner_manager::change_owner(*account_info.key, new_owner);
        **account_info.lamports.try_borrow_mut().unwrap() += 1234567;

        // serialize borsh
        let borsh_structure = BorshStructure {
            key: Pubkey::from_str("4xRtyUw1QSVZSGi1BUb7nbYBk8TC9P1K1AE2xtxwaZmV").unwrap(),
        };
        assert_eq!(account_info.data_len(), 32);
        borsh_structure
            .serialize(&mut *account_info.try_borrow_mut_data().unwrap())
            .unwrap();
    });
    let (_, lamports, owner) = load_account_info_data(
        &Pubkey::from_str("6Tw6Z6SsM3ypmGsB3vpSx8midhhyTvTwdPd7K413LyyY").unwrap(),
    );
    let expected = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap();
    assert_eq!(owner, expected);
    assert_eq!(lamports, 1234567);
}

//
// Helper functions
//

fn create_cpi_program_id() -> String {
    let key = Pubkey::from_str("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").unwrap();
    let key = bincode::serialize(&key).unwrap();
    base64::encode(key)
}

fn create_account_with_space(key: &str, space: usize) {
    let key = Pubkey::from_str(&key).unwrap();
    let account_manager = create_account_manager();
    let owner = Pubkey::default();
    let account_file_data = AccountFileData {
        owner,
        data: vec![0u8; space],
        lamports: 100,
    };
    account_manager
        .write_account(&key, &account_file_data)
        .unwrap();
}

#[derive(BorshSerialize)]
struct BorshStructure {
    key: Pubkey,
}

struct MyLineReader {
    pub lines: Vec<String>,
    pub current_line: usize,
}

impl MyLineReader {
    fn create(lines: Vec<&str>) -> Self {
        let lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
        Self {
            lines,
            current_line: 0,
        }
    }
}

impl LineReader for MyLineReader {
    fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
        let line_with_ender = format!("{}\n", &self.lines[self.current_line]);
        buf.write_str(&line_with_ender).unwrap();
        self.current_line += 1;
        Ok(1)
    }
}

fn create_instruction_payload() -> String {
    let program_id = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap();
    let account_meta = AccountMeta {
        pubkey: Pubkey::from_str("6Tw6Z6SsM3ypmGsB3vpSx8midhhyTvTwdPd7K413LyyY").unwrap(),
        is_signer: false,
        is_writable: false,
    };
    let accounts = vec![account_meta];
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0];
    let instruction = solana_program::instruction::Instruction {
        program_id,
        accounts,
        data,
    };
    let serialized = bincode::serialize(&instruction).unwrap();
    base64::encode(serialized)
}

fn create_signers_seeds() -> String {
    let signers_seeds: &[&[&[u8]]] = &[];
    let signers_seeds = bincode::serialize(&signers_seeds).unwrap();
    let signers_seeds = base64::encode(&signers_seeds);
    signers_seeds
}

fn create_cpi_accounts(data_size: usize) -> String {
    let accounts: Vec<AccountInfoSerialize> = vec![AccountInfoSerialize {
        key: Pubkey::from_str("6Tw6Z6SsM3ypmGsB3vpSx8midhhyTvTwdPd7K413LyyY").unwrap(),
        is_signer: false,
        is_writable: false,
        lamports: 0,
        data: vec![0u8; data_size],
        owner: Pubkey::default(),
        executable: false,
        rent_epoch: 1,
    }];
    let serialized = bincode::serialize(&accounts).unwrap();
    base64::encode(serialized)
}

fn create_payload() -> String {
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

    base64_encoded
}
