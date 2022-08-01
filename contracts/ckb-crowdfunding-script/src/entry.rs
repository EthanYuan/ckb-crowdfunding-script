// Import from `core` instead of from `std` since we are in no-std mode
use core::{cell, result::Result};

// Import heap related library from `alloc`
// https://doc.rust-lang.org/alloc/index.html
use alloc::{vec, vec::Vec};

// Import CKB syscalls and structures
// https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/index.html
use ckb_std::{
    ckb_constants::{CellField, Source},
    ckb_types::{bytes::Bytes, prelude::*},
    debug,
    high_level::{
        load_script, load_transaction, load_tx_hash, load_witness_args, look_for_dep_with_data_hash,
    },
    syscalls::{load_cell_by_field, load_cell_data},
};

use super::claim;
use super::data::CrowdfundingInfo;
use super::helper;
use super::withdraw;
use crate::error::Error;

pub fn main() -> Result<(), Error> {
    // remove below examples and write your code here

    let script = load_script()?;
    let args: Bytes = script.args().unpack();
    debug!("script args is {:?}", args);

    // return an error if args is invalid
    if args.is_empty() {
        return Err(Error::InvalidArgument);
    }

    // check c_cell_type_args in cell_deps
    let mut c_cell_type_args = [0u8; 32];
    c_cell_type_args.copy_from_slice(&args[0..32]);
    let index = look_for_dep_with_data_hash(&c_cell_type_args)?;
    debug!("index is {:?}", index);

    // read project data
    let crowdfunding_info = CrowdfundingInfo::try_from(index)?;
    debug!("crowdfunding_info is {:?}", crowdfunding_info);

    // parse script args
    let mut receiver_lock_hash = [0u8; 20];
    let mut sender_lock_hash = [0u8; 20];
    receiver_lock_hash.copy_from_slice(&args[32..52]);
    sender_lock_hash.copy_from_slice(&args[52..72]);

    // unlock
    // match helper::validate_signature_of_receiver_and_sender(&receiver_lock_hash, &sender_lock_hash)
    // {
    //     Ok(is_receiver) => {
    //         if is_receiver {
    //             claim::validate(&sender_lock_hash, &receiver_lock_hash, false)
    //         } else {
    //             withdraw::validate(&sender_lock_hash, false)
    //         }
    //     }
    //     Err(_) => Err(Error::NoMatchedSignature),
    // }

    Ok(())
}
