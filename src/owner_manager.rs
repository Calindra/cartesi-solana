use solana_program::{pubkey::Pubkey, msg};


static mut OWNERS: Vec<Pubkey> = Vec::new();
static mut POINTERS: Vec<(*mut &Pubkey, Pubkey)> = Vec::new();

pub fn clear() {
    unsafe {
        OWNERS.clear();
        POINTERS.clear();
    }
}

pub fn add_ptr(p: *mut Pubkey, key: Pubkey) {
    unsafe {
        POINTERS.push((p as *mut &Pubkey, key));
    }
}

pub fn change_owner<'a>(key: Pubkey, new_owner: Pubkey) {
    unsafe {
        let tot = OWNERS.len();
        OWNERS.push(new_owner);
        let pointers = &POINTERS;
        for (i, item) in pointers.iter().enumerate() {
            if item.1.to_string() == key.to_string() {
                let old = *item.0;
                *item.0 = &OWNERS[tot];
                msg!(
                    "change_owner: i[{}] account[{:?}] old[{:?}] new[{:?}]",
                    i,
                    key,
                    old,
                    new_owner
                );
                return;
            }
        }
        panic!("Account [{:?}] not found, change owner failed.", key);
    }
}
