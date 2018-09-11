#[macro_use] extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate rmp_serde;
extern crate byteorder;
extern crate rust_libindy_wrapper as indy;
#[macro_use]
mod utils;

use indy::did::Did;
use indy::ErrorCode;
use std::sync::mpsc::channel;
use std::time::Duration;
use utils::b58::{FromBase58, IntoBase58};
use utils::constants::{DID_1, SEED_1, VERKEY_1};
use utils::setup::{Setup, SetupConfig};
use utils::wallet::Wallet;

const VALID_TIMEOUT: Duration = Duration::from_secs(5);
const INVALID_TIMEOUT: Duration = Duration::from_micros(1);
const INVALID_HANDLE: i32 = 583741;

#[inline]
fn assert_verkey_len(verkey: &str) {
    assert_eq!(32, verkey.from_base58().unwrap().len());
}


#[cfg(test)]
mod create_new_did {
    use super::*;

    #[inline]
    fn assert_did_length(did: &str) {
        assert_eq!(16, did.from_base58().unwrap().len());
    }

    #[test]
    fn create_did_with_empty_json() {
        let wallet = Wallet::new();

        let (did, verkey) = Did::new(wallet.handle, "{}").unwrap();

        assert_did_length(&did);
        assert_verkey_len(&verkey);
    }

    #[test]
    fn create_did_with_seed() {
        let wallet = Wallet::new();

        let config = json!({
            "seed": SEED_1
        }).to_string();

        let (did, verkey) = Did::new(wallet.handle, &config).unwrap();

        assert_eq!(DID_1, did);
        assert_eq!(VERKEY_1, verkey);
    }

    #[test]
    fn create_did_with_cid() {
        let wallet = Wallet::new();

        let config = json!({
            "seed": SEED_1,
            "cid": true,
        }).to_string();

        let (did, verkey) = Did::new(wallet.handle, &config).unwrap();

        assert_eq!(VERKEY_1, did);
        assert_eq!(VERKEY_1, verkey);
    }

    #[test]
    fn create_did_with_did() {
        let wallet = Wallet::new();

        let config = json!({
            "did": DID_1
        }).to_string();

        let (did, verkey) = Did::new(wallet.handle, &config).unwrap();

        assert_eq!(DID_1, did);
        assert_ne!(VERKEY_1, verkey);
    }

    #[test]
    fn create_did_with_crypto_type() {
        let wallet = Wallet::new();

        let config = json!({
            "crypto_type": "ed25519"
        }).to_string();

        let result = Did::new(wallet.handle, &config);

        assert!(result.is_ok());

    }

    #[test]
    fn create_did_with_invalid_wallet_handle() {
        let result = Did::new(INVALID_HANDLE, "{}");
        assert_eq!(ErrorCode::WalletInvalidHandle, result.unwrap_err());
    }

    #[test]
    fn create_wallet_empty_config() {
        let wallet = Wallet::new();
        
        let result = Did::new(wallet.handle, "");

        assert!(result.is_err());
    }

    #[test]
    fn create_did_async_no_config() {
        let wallet = Wallet::new();
        let (sender, receiver) = channel();

        Did::new_async(
            wallet.handle,
            "{}",
            move |ec, did, verkey| { sender.send((ec, did, verkey)).unwrap(); }
        );

        let (ec, did, verkey) = receiver.recv_timeout(VALID_TIMEOUT).unwrap();
        
        assert_eq!(ErrorCode::Success, ec);
        assert_did_length(&did);
        assert_verkey_len(&verkey);
    }
    
    #[test]
    fn create_did_async_with_seed() {
        let wallet = Wallet::new();
        let (sender, receiver) = channel();
        let config = json!({
            "seed": SEED_1
        }).to_string();

        Did::new_async(
            wallet.handle,
            &config,
            move |ec, did, key| { sender.send((ec, did, key)).unwrap(); }
        );

        let (ec, did, verkey) = receiver.recv_timeout(VALID_TIMEOUT).unwrap();

        assert_eq!(ErrorCode::Success, ec);
        assert_eq!(DID_1, did);
        assert_eq!(VERKEY_1, verkey);
    }

    #[test]
    fn create_did_async_invalid_wallet() {
        let (sender, receiver) = channel();

        Did::new_async(
            INVALID_HANDLE,
            "{}",
            move |ec, did, key| sender.send((ec, did, key)).unwrap()
        );

        let result = receiver.recv_timeout(VALID_TIMEOUT).unwrap();

        let expected = (ErrorCode::WalletInvalidHandle, String::new(), String::new());
        assert_eq!(expected, result);
    }

    #[test]
    fn create_did_timeout_no_config() {
        let wallet = Wallet::new();
        let (did, verkey) = Did::new_timeout(
            wallet.handle,
            "{}",
            VALID_TIMEOUT
        ).unwrap();

        assert_did_length(&did);
        assert_verkey_len(&verkey);
    }

    #[test]
    fn create_did_timeout_with_seed() {
        let wallet = Wallet::new();
        let config = json!({"seed": SEED_1}).to_string();
        let (did, verkey) = Did::new_timeout(
            wallet.handle,
            &config,
            VALID_TIMEOUT
        ).unwrap();

        assert_eq!(DID_1, did);
        assert_eq!(VERKEY_1, verkey);
    }

    #[test]
    fn create_did_timeout_invalid_wallet() {
        let result = Did::new_timeout(INVALID_HANDLE, "{}", VALID_TIMEOUT);
        assert_eq!(ErrorCode::WalletInvalidHandle, result.unwrap_err());
    }

    #[test]
    fn create_did_timeout_timeouts() {
        let wallet = Wallet::new();
        let config = json!({"seed": SEED_1}).to_string();
        let result = Did::new_timeout(
            wallet.handle,
            &config,
            INVALID_TIMEOUT
        );

        assert_eq!(ErrorCode::CommonIOError, result.unwrap_err());
    }
}

#[cfg(test)]
mod replace_keys_start {
    use super::*;

    #[test]
    fn replace_keys_start() {
        let wallet = Wallet::new();
        let (did, verkey) = Did::new(wallet.handle, "{}").unwrap();

        let new_verkey = Did::replace_keys_start(wallet.handle, &did, "{}").unwrap();

        assert_verkey_len(&new_verkey);
        assert_ne!(verkey, new_verkey);
    }

    #[test]
    fn replace_keys_start_invalid_wallet() {
        let wallet = Wallet::new();

        let result = Did::replace_keys_start(INVALID_HANDLE, DID_1, "{}");

        assert_eq!(ErrorCode::WalletInvalidHandle, result.unwrap_err());
    }

    #[test]
    fn replace_keys_start_with_seed() {
        let wallet = Wallet::new();
        let (did, verkey) = Did::new(wallet.handle, "{}").unwrap();
        let config = json!({"seed": SEED_1}).to_string();

        let new_verkey = Did::replace_keys_start(wallet.handle, &did, &config).unwrap();

        assert_eq!(VERKEY_1, new_verkey);
        assert_ne!(verkey, new_verkey);
    }

    #[test]
    fn replace_keys_start_valid_crypto_type() {
        let wallet = Wallet::new();
        let (did, verkey) = Did::new(wallet.handle, "{}").unwrap();
        let config = json!({"crypto_type": "ed25519"}).to_string();

        let new_verkey = Did::replace_keys_start(wallet.handle, &did, &config).unwrap();

        assert_verkey_len(&new_verkey);
        assert_ne!(verkey, new_verkey);
    }

    #[test]
    fn replace_keys_start_invalid_crypto_type() {
        let wallet = Wallet::new();
        let (did, verkey) = Did::new(wallet.handle, "{}").unwrap();
        let config = json!({"crypto_type": "ed25518"}).to_string();

        let result = Did::replace_keys_start(wallet.handle, &did, &config);

        assert_eq!(ErrorCode::UnknownCryptoTypeError, result.unwrap_err());
    }

    #[test]
    fn replace_keys_start_invalid_did() {
        let wallet = Wallet::new();
        let result = Did::replace_keys_start(wallet.handle, DID_1, "{}");

        assert_eq!(ErrorCode::WalletItemNotFound, result.unwrap_err());
    }

    #[test]
    fn replace_keys_start_async() {
        let wallet = Wallet::new();
        let (sender, receiver) = channel();
        let (did, verkey) = Did::new(wallet.handle, "{}").unwrap();

        Did::replace_keys_start_async(
            wallet.handle,
            &did,
            "{}",
            move |ec, verkey| sender.send((ec, verkey)).unwrap()
        );

        let (ec, new_verkey) = receiver.recv_timeout(VALID_TIMEOUT).unwrap();

        assert_eq!(ErrorCode::Success, ec);
        assert_verkey_len(&new_verkey);
        assert_ne!(verkey, new_verkey);
    }

    #[test]
    fn replace_keys_start_async_invalid_wallet() {
        let (sender, receiver) = channel();

        Did::replace_keys_start_async(
            INVALID_HANDLE,
            DID_1,
            "{}",
            move |ec, verkey| sender.send((ec, verkey)).unwrap()
        );

        let (ec, new_verkey) = receiver.recv_timeout(VALID_TIMEOUT).unwrap();

        assert_eq!(ErrorCode::WalletInvalidHandle, ec);
    }

    #[test]
    fn replace_keys_start_async_with_seed() {
        let wallet = Wallet::new();
        let (sender, receiver) = channel();
        let (did, verkey) = Did::new(wallet.handle, "{}").unwrap();
        let config = json!({"seed": SEED_1}).to_string();

        Did::replace_keys_start_async(
            wallet.handle,
            &did,
            &config,
            move |ec, verkey| sender.send((ec, verkey)).unwrap()
        );

        let (ec, new_verkey) = receiver.recv_timeout(VALID_TIMEOUT).unwrap();

        assert_eq!(ErrorCode::Success, ec);
        assert_eq!(VERKEY_1, new_verkey);
        assert_ne!(verkey, new_verkey);
    }

    #[test]
    fn replace_keys_start_timeout() {
        let wallet = Wallet::new();
        let (did, verkey) = Did::new(wallet.handle, "{}").unwrap();

        let new_verkey = Did::replace_keys_start_timeout(
            wallet.handle,
            &did,
            "{}",
            VALID_TIMEOUT
        ).unwrap();

        assert_verkey_len(&new_verkey);
        assert_ne!(verkey, new_verkey);
    }

    #[test]
    fn replace_keys_start_timeout_with_seed() {
        let wallet = Wallet::new();
        let (did, verkey) = Did::new(wallet.handle, "{}").unwrap();
        let config = json!({"seed": SEED_1}).to_string();

        let new_verkey = Did::replace_keys_start_timeout(
            wallet.handle,
            &did,
            &config,
            VALID_TIMEOUT
        ).unwrap();

        assert_eq!(VERKEY_1, new_verkey);
        assert_ne!(verkey, new_verkey);
    }

    #[test]
    fn replace_keys_start_timeout_invalid_wallet() {
        let result = Did::replace_keys_start_timeout(
            INVALID_HANDLE,
            DID_1,
            "{}",
            VALID_TIMEOUT
        );

        assert_eq!(ErrorCode::WalletInvalidHandle, result.unwrap_err());
    }

    #[test]
    fn replace_keys_start_timeout_timeouts() {
        let wallet = Wallet::new();
        let (did, verkey) = Did::new(wallet.handle, "{}").unwrap();

        let result = Did::replace_keys_start_timeout(
            wallet.handle,
            &did,
            "{}",
            INVALID_TIMEOUT
        );

        assert_eq!(ErrorCode::CommonIOError, result.unwrap_err());
    }
}

#[cfg(test)]
mod replace_keys_apply {
    use super::*;

    fn setup() -> (Wallet, String, String) {
        let wallet = Wallet::new();
        let (did, verkey) = Did::new(wallet.handle, "{}").unwrap();

        (wallet, did, verkey)
    }

    #[inline]
    fn start_key_replacement(wallet: &Wallet, did: &str) {
        let config = json!({"seed": SEED_1}).to_string();
        Did::replace_keys_start(wallet.handle, did, &config).unwrap();
    }

    #[test]
    fn replace_keys_apply() {
        let (wallet, did, verkey) = setup();
        start_key_replacement(&wallet, &did);

        let result = Did::replace_keys_apply(wallet.handle, &did);

        assert_eq!((), result.unwrap());

        let new_verkey = Did::get_ver_key_local(wallet.handle, &did).unwrap();

        assert_eq!(VERKEY_1, new_verkey);
        assert_ne!(verkey, new_verkey);
    }

    #[test]
    fn replace_keys_apply_without_replace_keys_start() {
        let (wallet, did, _) = setup();

        let result = Did::replace_keys_apply(wallet.handle, &did);

        assert_eq!(ErrorCode::WalletItemNotFound, result.unwrap_err());
    }

    #[test]
    fn replace_keys_apply_invalid_did() {
        let wallet = Wallet::new();

        let result = Did::replace_keys_apply(wallet.handle, DID_1);

        assert_eq!(ErrorCode::WalletItemNotFound, result.unwrap_err());
    }

    #[test]
    fn replace_keys_apply_invalid_wallet() {
        let result = Did::replace_keys_apply(INVALID_HANDLE, DID_1);
        assert_eq!(ErrorCode::WalletInvalidHandle, result.unwrap_err());
    }

    #[test]
    fn replace_keys_apply_async() {
        let (wallet, did, verkey) = setup();
        let (sender, receiver) = channel();
        start_key_replacement(&wallet, &did);

        Did::replace_keys_apply_async(
            wallet.handle,
            &did,
            move |ec| sender.send(ec).unwrap()
        );

        let ec = receiver.recv_timeout(VALID_TIMEOUT).unwrap();
        let new_verkey = Did::get_ver_key_local(wallet.handle, &did).unwrap();

        assert_eq!(ErrorCode::Success, ec);
        assert_eq!(VERKEY_1, new_verkey);
        assert_ne!(verkey, new_verkey);
    }

    #[test]
    fn replace_keys_apply_async_invalid_wallet() {
        let (sender, receiver) = channel();

        Did::replace_keys_apply_async(
            INVALID_HANDLE,
            DID_1,
            move |ec| sender.send(ec).unwrap()
        );

        let ec = receiver.recv_timeout(VALID_TIMEOUT).unwrap();

        assert_eq!(ErrorCode::WalletInvalidHandle, ec);
    }

    #[test]
    fn replace_keys_apply_timeout() {
        let (wallet, did, verkey) = setup();
        start_key_replacement(&wallet, &did);

        let result = Did::replace_keys_apply_timeout(
            wallet.handle,
            &did,
            VALID_TIMEOUT
        );
        let new_verkey = Did::get_ver_key_local(wallet.handle, &did).unwrap();

        assert_eq!((), result.unwrap());
        assert_eq!(VERKEY_1, new_verkey);
        assert_ne!(verkey, new_verkey);
    }

    #[test]
    fn replace_keys_apply_timeout_invalid_wallet() {
        let result = Did::replace_keys_apply_timeout(
            INVALID_HANDLE,
            DID_1,
            VALID_TIMEOUT
        );

        assert_eq!(ErrorCode::WalletInvalidHandle, result.unwrap_err());
    }

    #[test]
    fn replace_keys_apply_timeout_timeouts() {
        let result = Did::replace_keys_apply_timeout(
            INVALID_HANDLE,
            DID_1,
            INVALID_TIMEOUT
        );

        assert_eq!(ErrorCode::CommonIOError, result.unwrap_err());
    }
}

#[cfg(test)]
mod test_store_their_did {
    use super::*;

    #[test]
    fn store_their_did() {
        let wallet = Wallet::new();
        let config = json!({"did": VERKEY_1}).to_string();

        let result = Did::store_their_did(wallet.handle, &config);
    
        assert_eq!((), result.unwrap());

        let verkey = Did::get_ver_key_local(wallet.handle, VERKEY_1).unwrap();

        assert_eq!(VERKEY_1, verkey);
    }

    #[test]
    fn store_their_did_with_verkey() {
        let wallet = Wallet::new();
        let config = json!({"did": DID_1, "verkey": VERKEY_1}).to_string();

        let result = Did::store_their_did(wallet.handle, &config);
    
        assert_eq!((), result.unwrap());

        let verkey = Did::get_ver_key_local(wallet.handle, DID_1).unwrap();

        assert_eq!(VERKEY_1, verkey);
    }

    #[test]
    fn store_their_did_with_crypto_verkey() {
        let wallet = Wallet::new();
        let config = json!({
            "did": DID_1,
            "verkey": format!("{}:ed25519", VERKEY_1)
        }).to_string();

        let result = Did::store_their_did(wallet.handle, &config);

        assert_eq!((), result.unwrap());

        let verkey = Did::get_ver_key_local(wallet.handle, DID_1).unwrap();

        assert_eq!(format!("{}:ed25519", VERKEY_1), verkey);
    }

    #[test]
    fn store_their_did_empty_identify_json() {
        let wallet = Wallet::new();

        let result = Did::store_their_did(wallet.handle, "{}");

        assert_eq!(ErrorCode::CommonInvalidStructure, result.unwrap_err());
    }

    #[test]
    fn store_their_did_invalid_handle() {
        let config = json!({"did": DID_1, "verkey": VERKEY_1}).to_string();
        let result = Did::store_their_did(INVALID_HANDLE, &config);
        assert_eq!(ErrorCode::WalletInvalidHandle, result.unwrap_err());
    }

    #[test]
    fn store_their_did_abbreviated_verkey() {
        let wallet = Wallet::new();
        let config = json!({
            "did": "8wZcEriaNLNKtteJvx7f8i",
            "verkey": "~NcYxiDXkpYi6ov5FcYDi1e"
        }).to_string();

        let result = Did::store_their_did(wallet.handle, &config);
        
        assert_eq!((), result.unwrap());
    }

    #[test]
    fn store_their_did_invalid_did() {
        let wallet = Wallet::new();
        let config = json!({"did": "InvalidDid"}).to_string();

        let result = Did::store_their_did(wallet.handle, &config);

        assert_eq!(ErrorCode::CommonInvalidStructure, result.unwrap_err());
    }

    #[test]
    fn store_their_did_with_invalid_verkey() {
        let wallet = Wallet::new();
        let config = json!({
            "did": DID_1,
            "verkey": "InvalidVerkey"
        }).to_string();

        let result = Did::store_their_did(wallet.handle, &config);

        assert_eq!(ErrorCode::CommonInvalidStructure, result.unwrap_err());
    }

    #[test]
    fn store_their_did_with_invalid_crypto_verkey() {
        let wallet = Wallet::new();
        let config = json!({
            "did": DID_1,
            "verkey": format!("{}:bad_crypto_type", VERKEY_1)
        }).to_string();

        let result = Did::store_their_did(wallet.handle, &config);

        assert_eq!(ErrorCode::UnknownCryptoTypeError, result.unwrap_err());
    }

    #[test]
    fn store_their_did_duplicate() {
        let wallet = Wallet::new();
        let config = json!({"did": DID_1, "verkey": VERKEY_1}).to_string();

        Did::store_their_did(wallet.handle, &config).unwrap();

        let result = Did::store_their_did(wallet.handle, &config);

        assert_eq!(ErrorCode::WalletItemAlreadyExists, result.unwrap_err());
    }

    #[test]
    /*
    This test resulted from the ticket https://jira.hyperledger.org/browse/IS-802
    Previously, an error was being thrown because rollback wasn't happening.
    This test ensures the error is no longer occuring.
    */
    fn store_their_did_multiple_error_fixed() {
        let wallet = Wallet::new();
        let config = json!({"did": DID_1, "verkey": VERKEY_1}).to_string();

        Did::store_their_did(wallet.handle, &config).unwrap();

        let result = Did::store_their_did(wallet.handle, &config);
        assert_eq!(ErrorCode::WalletItemAlreadyExists, result.unwrap_err());

        let result = Did::store_their_did(wallet.handle, &config);
        assert_eq!(ErrorCode::WalletItemAlreadyExists, result.unwrap_err());
    }

    #[test]
    fn store_their_did_async_with_verkey() {
        let wallet = Wallet::new();
        let config = json!({"did": DID_1, "verkey": VERKEY_1}).to_string();
        let (sender, receiver) = channel();

        Did::store_their_did_async(
            wallet.handle,
            &config,
            move |ec| sender.send(ec).unwrap()
        );

        let ec = receiver.recv_timeout(VALID_TIMEOUT).unwrap();

        assert_eq!(ErrorCode::Success, ec);

        let verkey = Did::get_ver_key_local(wallet.handle, DID_1).unwrap();

        assert_eq!(VERKEY_1, verkey);
    }

    #[test]
    fn store_their_did_async_invalid_wallet() {
        let config = json!({"did": DID_1, "verkey": VERKEY_1}).to_string();
        let (sender, receiver) = channel();

        Did::store_their_did_async(
            INVALID_HANDLE,
            &config,
            move |ec| sender.send(ec).unwrap()
        );

        let ec = receiver.recv_timeout(VALID_TIMEOUT).unwrap();

        assert_eq!(ErrorCode::WalletInvalidHandle, ec);
    }
    
    #[test]
    fn store_their_did_timeout_with_verkey() {
        let wallet = Wallet::new();
        let config = json!({"did": DID_1, "verkey": VERKEY_1}).to_string();

        let result = Did::store_their_did_timeout(
            wallet.handle,
            &config,
            VALID_TIMEOUT
        );

        assert_eq!((), result.unwrap());

        let verkey = Did::get_ver_key_local(wallet.handle, DID_1).unwrap();

        assert_eq!(VERKEY_1, verkey);
    }

    #[test]
    fn store_their_did_timeout_invalid_wallet() {
        let config = json!({"did": DID_1, "verkey": VERKEY_1}).to_string();

        let result = Did::store_their_did_timeout(
            INVALID_HANDLE,
            &config,
            VALID_TIMEOUT
        );

        assert_eq!(ErrorCode::WalletInvalidHandle, result.unwrap_err());
    }

    #[test]
    fn store_their_did_timeout_timeouts() {
        let config = json!({"did": DID_1, "verkey": VERKEY_1}).to_string();

        let result = Did::store_their_did_timeout(
            INVALID_HANDLE,
            &config,
            INVALID_TIMEOUT
        );

        assert_eq!(ErrorCode::CommonIOError, result.unwrap_err())
    }
}

#[cfg(test)]
mod test_get_verkey_local {
    use super::*;

    #[test]
    fn get_verkey_local_my_did() {
        let wallet = Wallet::new();
        let config = json!({"seed": SEED_1}).to_string();
        let (did, verkey) = Did::new(wallet.handle, &config).unwrap();

        let stored_verkey = Did::get_ver_key_local(wallet.handle, &did).unwrap();

        assert_eq!(verkey, stored_verkey);
    }

    #[test]
    fn get_verkey_local_their_did() {
        let wallet = Wallet::new();
        let config = json!({"did": DID_1, "verkey": VERKEY_1}).to_string();
        Did::store_their_did(wallet.handle, &config).unwrap();

        let stored_verkey = Did::get_ver_key_local(wallet.handle, DID_1).unwrap();

        assert_eq!(VERKEY_1, stored_verkey);
    }

    #[test]
    fn get_verkey_local_invalid_did() {
        let wallet = Wallet::new();
        let result = Did::get_ver_key_local(wallet.handle, DID_1);

        assert_eq!(ErrorCode::WalletItemNotFound, result.unwrap_err());
    }

    #[test]
    fn get_verkey_local_invalid_wallet() {
        let result = Did::get_ver_key_local(INVALID_HANDLE, DID_1);
        assert_eq!(ErrorCode::WalletInvalidHandle, result.unwrap_err());
    }

    #[test]
    fn get_verkey_local_async() {
        let wallet = Wallet::new();
        let (sender, receiver) = channel();
        let config = json!({"seed": SEED_1}).to_string();
        let (did, verkey) = Did::new(wallet.handle, &config).unwrap();

        Did::get_ver_key_local_async(
            wallet.handle,
            &did,
            move |ec, verkey| sender.send((ec, verkey)).unwrap()
        );

        let (ec, stored_verkey) = receiver.recv_timeout(VALID_TIMEOUT).unwrap();

        assert_eq!(ErrorCode::Success, ec);
        assert_eq!(VERKEY_1, stored_verkey);
    }

    #[test]
    fn get_verkey_local_async_invalid_wallet() {
        let (sender, receiver) = channel();

        Did::get_ver_key_local_async(
            INVALID_HANDLE,
            DID_1,
            move |ec, verkey| sender.send((ec, verkey)).unwrap()
        );

        let (ec, stored_verkey) = receiver.recv_timeout(VALID_TIMEOUT).unwrap();

        assert_eq!(ErrorCode::WalletInvalidHandle, ec);
        assert_eq!(String::from(""), stored_verkey);
    }

    #[test]
    fn get_verkey_local_timeout() {
        let wallet = Wallet::new();
        let config = json!({"seed": SEED_1}).to_string();
        let (did, verkey) = Did::new(wallet.handle, &config).unwrap();

        let stored_verkey = Did::get_ver_key_local_timeout(
            wallet.handle,
            &did,
            VALID_TIMEOUT
        ).unwrap();

        assert_eq!(verkey, stored_verkey);
    }

    #[test]
    fn get_verkey_local_timeout_invalid_wallet() {
        let result = Did::get_ver_key_local_timeout(
            INVALID_HANDLE,
            DID_1,
            VALID_TIMEOUT
        );

        assert_eq!(ErrorCode::WalletInvalidHandle, result.unwrap_err());
    }

    #[test]
    fn get_verkey_local_timeout_timeouts() {
        let result = Did::get_ver_key_local_timeout(
            INVALID_HANDLE,
            DID_1,
            INVALID_TIMEOUT
        );
        
        assert_eq!(ErrorCode::CommonIOError, result.unwrap_err());
    }

    #[cfg(test)]
    mod test_get_verkey_ledger {
        use super::*;
        use indy::ledger::Ledger;

        #[test]
        fn get_verkey_my_did() {
            let wallet = Wallet::new();
            let (did, verkey) = Did::new(wallet.handle, "{}").unwrap();

            let stored_verkey = Did::get_ver_key(
                -1,
                wallet.handle,
                &did
            ).unwrap();

            assert_eq!(verkey, stored_verkey);
        }

        #[test]
        fn get_verkey_their_did() {
            let wallet = Wallet::new();
            let config = json!({"did": DID_1, "verkey": VERKEY_1}).to_string();
            Did::store_their_did(wallet.handle, &config).unwrap();

            let stored_verkey = Did::get_ver_key(
                -1,
                wallet.handle,
                DID_1,
            ).unwrap();

            assert_eq!(VERKEY_1, stored_verkey);
        }

        #[test]
        fn get_verkey_not_on_ledger() {
            let wallet = Wallet::new();
            let wallet2 = Wallet::new();
            let setup = Setup::new(&wallet, SetupConfig {
                connect_to_pool: true,
                num_trustees: 0,
                num_users: 0,
                num_nodes: 4
            });
            let pool_handle = setup.pool_handle.unwrap();

            let (did, verkey) = Did::new(wallet.handle, "{}").unwrap();

            let result = Did::get_ver_key(
                pool_handle,
                wallet2.handle,
                &did
            );

            assert_eq!(ErrorCode::WalletItemNotFound, result.unwrap_err());
        }

        #[test]
        fn get_verkey_on_ledger() {
            let wallet = Wallet::new();
            let wallet2 = Wallet::new();
            let setup = Setup::new(&wallet, SetupConfig {
                connect_to_pool: true,
                num_trustees: 1,
                num_users: 1,
                num_nodes: 4
            });
            let pool_handle = setup.pool_handle.unwrap();
            let user = &setup.users.as_ref().unwrap()[0];

            let ledger_verkey = Did::get_ver_key(
                pool_handle,
                wallet2.handle,
                &user.did
            ).unwrap();

            assert_eq!(ledger_verkey, user.verkey);
        }

        #[test]
        fn get_verkey_invalid_pool() {
            let wallet = Wallet::new();

            let result = Did::get_ver_key(-1, wallet.handle, DID_1);

            assert_eq!(ErrorCode::PoolLedgerInvalidPoolHandle, result.unwrap_err());
        }

        #[test]
        fn get_verkey_invalid_wallet() {
            let result = Did::get_ver_key(-1, INVALID_HANDLE, DID_1);
            assert_eq!(ErrorCode::WalletInvalidHandle, result.unwrap_err());
        }

        #[test]
        fn get_verkey_async_my_did() {
            let (sender, receiver) = channel();
            let wallet = Wallet::new();
            let (did, verkey) = Did::new(wallet.handle, "{}").unwrap();

            Did::get_ver_key_async(
                -1,
                wallet.handle,
                &did,
                move |ec, verkey| sender.send((ec, verkey)).unwrap()
            );

            let (ec, stored_verkey) = receiver.recv_timeout(VALID_TIMEOUT).unwrap();

            assert_eq!(verkey, stored_verkey);
        }

        #[test]
        fn get_verkey_async_invalid_wallet() {
            let (sender, receiver) = channel();

            Did::get_ver_key_async(
                -1,
                INVALID_HANDLE,
                DID_1,
                move |ec, verkey| sender.send((ec, verkey)).unwrap()
            );

            let (ec, stored_verkey) = receiver.recv_timeout(VALID_TIMEOUT).unwrap();

            assert_eq!(ErrorCode::WalletInvalidHandle, ec);
            assert_eq!(String::from(""), stored_verkey);
        }

        #[test]
        fn get_verkey_timeout_my_did() {
            let wallet = Wallet::new();
            let config = json!({"seed": SEED_1}).to_string();
            let (did, verkey) = Did::new(wallet.handle, &config).unwrap();

            let stored_verkey = Did::get_ver_key_timeout(
                -1,
                wallet.handle,
                &did,
                VALID_TIMEOUT
            ).unwrap();

            assert_eq!(verkey, stored_verkey);
        }

        #[test]
        fn get_verkey_timeout_invalid_wallet() {
            let result = Did::get_ver_key_timeout(
                -1,
                INVALID_HANDLE, 
                DID_1,
                VALID_TIMEOUT
            );
    
            assert_eq!(ErrorCode::WalletInvalidHandle, result.unwrap_err());
        }

        #[test]
        fn get_verkey_timeout_timeouts() {
            let result = Did::get_ver_key_timeout(
                -1,
                INVALID_HANDLE, 
                DID_1,
                INVALID_TIMEOUT
            );
    
            assert_eq!(ErrorCode::CommonIOError, result.unwrap_err());
        }
    }
}