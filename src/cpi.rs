
/// Cross-Program Invocations
///
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

pub fn check_signature(
    signer_program_id: &Pubkey,
    instruction: &Instruction,
    pda_signature: &[&[&[u8]]],
) {
    let internal: Vec<&AccountMeta> = instruction
        .accounts
        .iter()
        .filter(|acc| !is_external(&acc.pubkey))
        .collect();
    for (i, acc) in internal.iter().enumerate() {
        if acc.is_signer {
            let expected =
                Pubkey::create_program_address(pda_signature[i], signer_program_id).unwrap();
            assert_eq!(expected, acc.pubkey);
        }
    }
}

fn is_external(pubkey: &Pubkey) -> bool {
    let bytes = pubkey.to_bytes();
    bytes[0..4] == [0, 0, 0, 0]
}
