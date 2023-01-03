use anchor_lang::prelude::Pubkey;
use borsh::BorshSerialize;
use cartesi_solana::{
    account_manager::{self, create_account_manager, serialize_with_padding, AccountFileData},
    adapter::load_account_info_data,
    executor::{Executor, LineReader, DefaultStdin},
    owner_manager,
    transaction::{self, Signature},
};
use solana_sdk::{
    hash::Hash,
    instruction::CompiledInstruction,
    message::{Message, MessageHeader}, account::AccountSharedData,
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
    unsafe {
        owner_manager::POINTERS.clear();
        owner_manager::OWNERS.clear();
    }
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
        "0", // instruction index
        "12345", // timestamp
    ]);

    let mut executor = Executor::create_with_stdin(stdin);

    create_account_with_space("6Tw6Z6SsM3ypmGsB3vpSx8midhhyTvTwdPd7K413LyyY", 32);

    executor.get_processor_args(|program_id, accounts, data| {
        assert_eq!(
            program_id.to_string(),
            "2QB8wEBJ8jjMQuZPvj3jaZP7JJb5j21u4xbxTnwsZRfv".to_string()
        );
        assert_eq!(accounts.len(), 7);
        assert_eq!(data, [141, 132, 233, 130, 168, 183, 10, 119]);
    });
}

#[test]
fn executor_should_load_change_and_save_account_infos() {
    setup();
    let payload = create_payload();
    let stdin = MyLineReader::create(vec![
        "Header: External CPI",
        "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
        &payload,
        "0", // instruction index
        "12345", // timestamp
    ]);

    let mut executor = Executor::create_with_stdin(stdin);

    create_account_with_space("6Tw6Z6SsM3ypmGsB3vpSx8midhhyTvTwdPd7K413LyyY", 32);

    executor.get_processor_args(|_program_id, accounts, _data| {
        let borsh_structure = BorshStructure {
            key: Pubkey::from_str("4xRtyUw1QSVZSGi1BUb7nbYBk8TC9P1K1AE2xtxwaZmV").unwrap(),
        };
        let account_info = &accounts[1];
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
fn executor_should_save_account_info_resized() {
    setup();
    let payload = create_payload();
    let stdin = MyLineReader::create(vec![
        "Header: External CPI",
        "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
        &payload,
        "0", // instruction index
        "12345", // timestamp
    ]);

    let mut executor = Executor::create_with_stdin(stdin);

    executor.get_processor_args(|_program_id, accounts, _data| {
        let borsh_structure = BorshStructure {
            key: Pubkey::from_str("4xRtyUw1QSVZSGi1BUb7nbYBk8TC9P1K1AE2xtxwaZmV").unwrap(),
        };
        let account_info = &accounts[1];
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

fn _executor() {
    let stdin = DefaultStdin{};
    Executor::create_with_stdin(stdin);
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
