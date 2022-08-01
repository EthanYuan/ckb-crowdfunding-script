use super::*;

use crate::helper::sign_tx;

use ckb_system_scripts::BUNDLED_CELL;
use ckb_testtool::builtin::ALWAYS_SUCCESS;
use ckb_testtool::ckb_crypto::secp::Privkey;
use ckb_testtool::ckb_error::Error;
use ckb_testtool::ckb_hash::blake2b_256;
use ckb_testtool::ckb_types::{bytes::Bytes, core::TransactionBuilder, packed::*, prelude::*};
use ckb_testtool::context::Context;

const MAX_CYCLES: u64 = 10_000_000;

// error numbers
const ERROR_EMPTY_ARGS: i8 = 5;

fn assert_script_error(err: Error, err_code: i8) {
    let error_string = err.to_string();
    assert!(
        error_string.contains(format!("error code {} ", err_code).as_str()),
        "error_string: {}, expected_error_code: {}",
        error_string,
        err_code
    );
}

fn prepare_c_cell(context: &mut Context) -> (CellDep, [u8; 32]) {
    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());

    let contract_bin_2: Bytes = Loader::default().load_binary("ckb-project-type-id");
    let out_point_2 = context.deploy_cell(contract_bin_2);

    // prepare c-cell
    let lock_script_for_c_cell = context
        .build_script(&always_success_out_point, Bytes::from(vec![42]))
        .expect("lock_script_for_c_cell");
    let data = [
        0, 0, 0, 0, 0, 0, 0, 10, 0, 0, 0, 200, 0, 0, 0, 100, 0, 0, 0, 0, 0, 0, 0, 20, 0, 0, 0, 50,
        6, 0, 0, 0, 0, 0, 0, 0, 30, 0, 0, 0, 50, 6,
    ];
    let data_hash = blake2b_256(data);
    let type_script = context.build_script(&out_point_2, Bytes::from(data_hash.to_vec()));
    let type_script = type_script.pack();
    let c_cell = CellOutput::new_builder()
        .capacity(500u64.pack())
        .lock(lock_script_for_c_cell.clone())
        .type_(type_script)
        .build();
    let c_cell = context.create_cell(c_cell, Bytes::copy_from_slice(&data));
    let c_cell_dep = CellDep::new_builder().out_point(c_cell).build();
    (c_cell_dep, data_hash)
}

fn parepare_receiver_key() -> (String, String, [u8; 20]) {
    // random generation
    // receiver_address: "ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqvgs9hktyzvdk4x33phd7pkvyccq6g9tnq4y2d5j"
    // receiver_address_pk: "f8c30a5090d047c2eb4fde48de1034324edda6b1be0d84bbcb8644c5f1e944e0"
    // receiver lock hash 160: [164, 115, 144, 235, 133, 93, 188, 69, 232, 219, 43, 186, 250, 76, 21, 212, 222, 136, 109, 103]
    ("ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqvgs9hktyzvdk4x33phd7pkvyccq6g9tnq4y2d5j".to_string(), 
        "f8c30a5090d047c2eb4fde48de1034324edda6b1be0d84bbcb8644c5f1e944e0".to_string(),
        [164, 115, 144, 235, 133, 93, 188, 69, 232, 219, 43, 186, 250, 76, 21, 212, 222, 136, 109, 103])
}

fn prepare_sender_key() -> (String, String, [u8; 20]) {
    // random generation
    // sender_address: "ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqtkuyljsvw7qkwq0jc8rx9ekwyeh5wkrfs0mh696"
    // sender_address_pk: "263922d46c3acf249ae4f76f348bb0bb15ed6a5dc80f0253d3384ff078fe9fd8"
    // sender lock hash 160: [152, 130, 119, 177, 187, 68, 13, 158, 215, 120, 60, 184, 77, 93, 71, 97, 70, 85, 100, 155]

    ("ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqtkuyljsvw7qkwq0jc8rx9ekwyeh5wkrfs0mh696".to_string(),
        "263922d46c3acf249ae4f76f348bb0bb15ed6a5dc80f0253d3384ff078fe9fd8".to_string(), 
        [152, 130, 119, 177, 187, 68, 13, 158, 215, 120, 60, 184, 77, 93, 71, 97, 70, 85, 100, 155])
}

#[test]
fn test_receiver_success() {
    // deploy contract
    let mut context = Context::default();

    // prepare c-cell
    let (c_cell_dep, data_hash) = prepare_c_cell(&mut context);

    let contract_bin: Bytes = Loader::default().load_binary("ckb-crowdfunding-script");
    let out_point = context.deploy_cell(contract_bin);

    // prepare address
    let (receiver_address, receiver_key, receiver_lock_hash_h160) = parepare_receiver_key();
    let (sender_address, sender_key, sender_lock_hash_h160) = prepare_sender_key();

    // prepare scripts
    let mut args = data_hash.to_vec();
    args.extend_from_slice(&receiver_lock_hash_h160);
    args.extend_from_slice(&sender_lock_hash_h160);
    let args = Bytes::from(args);
    let lock_script = context.build_script(&out_point, args).expect("script");
    let lock_script_dep = CellDep::new_builder().out_point(out_point).build();

    // prepare cell dep
    let secp256k1_data_bin = BUNDLED_CELL.get("specs/cells/secp256k1_data").unwrap();
    let secp256k1_data_out_point = context.deploy_cell(secp256k1_data_bin.to_vec().into());
    let secp256k1_data_dep = CellDep::new_builder()
        .out_point(secp256k1_data_out_point)
        .build();

    // prepare cells
    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(1000u64.pack())
            .lock(lock_script.clone())
            .build(),
        Bytes::new(),
    );
    let input = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();
    let outputs = vec![
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .lock(lock_script.clone())
            .build(),
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .lock(lock_script)
            .build(),
    ];

    let outputs_data = vec![Bytes::new(); 2];

    let mut witnesses = vec![];
    witnesses.push(Bytes::new());

    // build transaction
    let tx = TransactionBuilder::default()
        .input(input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(lock_script_dep)
        .cell_dep(c_cell_dep)
        .cell_dep(secp256k1_data_dep)
        .witnesses(witnesses.pack())
        .build();
    let tx = context.complete_tx(tx);

    let private_key = Privkey::from_str(&receiver_key).unwrap();
    let tx = sign_tx(tx, &private_key);

    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}

#[test]
fn test_sender_success() {
    // deploy contract
    let mut context = Context::default();

    // prepare c-cell
    let (c_cell_dep, data_hash) = prepare_c_cell(&mut context);

    let contract_bin: Bytes = Loader::default().load_binary("ckb-crowdfunding-script");
    let out_point = context.deploy_cell(contract_bin);

    // prepare address
    let (receiver_address, receiver_key, receiver_lock_hash_h160) = parepare_receiver_key();
    let (sender_address, sender_key, sender_lock_hash_h160) = prepare_sender_key();

    // prepare scripts
    let mut args = data_hash.to_vec();
    args.extend_from_slice(&receiver_lock_hash_h160);
    args.extend_from_slice(&sender_lock_hash_h160);
    let args = Bytes::from(args);
    let lock_script = context.build_script(&out_point, args).expect("script");
    let lock_script_dep = CellDep::new_builder().out_point(out_point).build();

    // prepare cell dep
    let secp256k1_data_bin = BUNDLED_CELL.get("specs/cells/secp256k1_data").unwrap();
    let secp256k1_data_out_point = context.deploy_cell(secp256k1_data_bin.to_vec().into());
    let secp256k1_data_dep = CellDep::new_builder()
        .out_point(secp256k1_data_out_point)
        .build();

    // prepare cells
    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(1000u64.pack())
            .lock(lock_script.clone())
            .build(),
        Bytes::new(),
    );
    let input = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();
    let outputs = vec![
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .lock(lock_script.clone())
            .build(),
        CellOutput::new_builder()
            .capacity(499u64.pack())
            .lock(lock_script)
            .build(),
    ];

    let outputs_data = vec![Bytes::new(); 2];

    let mut witnesses = vec![];
    witnesses.push(Bytes::new());

    // build transaction
    let tx = TransactionBuilder::default()
        .input(input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(lock_script_dep)
        .cell_dep(c_cell_dep)
        .cell_dep(secp256k1_data_dep)
        .witnesses(witnesses.pack())
        .build();
    let tx = context.complete_tx(tx);

    let private_key = Privkey::from_str(&sender_key).unwrap();
    let tx = sign_tx(tx, &private_key);

    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}
