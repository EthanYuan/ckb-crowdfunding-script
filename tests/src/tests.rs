use super::*;

use ckb_testtool::builtin::ALWAYS_SUCCESS;
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

#[test]
fn test_success() {
    // deploy contract
    let mut context = Context::default();

    // prepare c-cell
    let (c_cell_dep, data_hash) = prepare_c_cell(&mut context);

    let contract_bin: Bytes = Loader::default().load_binary("ckb-crowdfunding-script");
    let out_point = context.deploy_cell(contract_bin);

    // prepare scripts
    let lock_script = context
        .build_script(&out_point, Bytes::from(data_hash.to_vec()))
        .expect("script");
    let lock_script_dep = CellDep::new_builder().out_point(out_point).build();

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

    // build transaction
    let tx = TransactionBuilder::default()
        .input(input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(lock_script_dep)
        .cell_dep(c_cell_dep)
        .build();
    let tx = context.complete_tx(tx);

    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}

// #[ignore]
#[test]
fn test_empty_args() {
    // deploy contract
    let mut context = Context::default();

    let contract_bin: Bytes = Loader::default().load_binary("ckb-crowdfunding-script");
    let out_point = context.deploy_cell(contract_bin);

    // prepare c-cell
    let (c_cell_dep, _data_hash) = prepare_c_cell(&mut context);

    // prepare scripts
    let lock_script = context
        .build_script(&out_point, Default::default())
        .expect("script");
    let lock_script_dep = CellDep::new_builder().out_point(out_point).build();

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

    // build transaction
    let tx = TransactionBuilder::default()
        .input(input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(lock_script_dep)
        .cell_dep(c_cell_dep)
        .build();
    let tx = context.complete_tx(tx);

    // run
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    println!("{:?}", err);
    assert_script_error(err, ERROR_EMPTY_ARGS);
}
