use anchor_lang::prelude::{Pubkey, msg};
use once_cell::sync::Lazy;
pub static mut OWNERS: Lazy<Vec<Pubkey>> = Lazy::new(|| vec![]);

/*
#0 170.8 error[E0015]: cannot call non-const fn `Mutex::<Vec<(*mut &Pubkey, Pubkey)>>::new` in statics
#0 170.8  --> ctsi_sol/src/owner_manager.rs:7:63
#0 170.8   |
#0 170.8 7 | pub static mut POINTERS: Mutex<Vec<(*mut &Pubkey, Pubkey)>> = Mutex::new(vec![]);
#0 170.8   |                                                               ^^^^^^^^^^^^^^^^^^
#0 170.8   |
#0 170.8   = note: calls in statics are limited to constant functions, tuple structs and tuple variants
*/
pub static mut POINTERS: Lazy<Vec<(*mut &Pubkey, Pubkey)>> = Lazy::new(|| vec![]);

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

