// Import heap related library from `alloc`
// https://doc.rust-lang.org/alloc/index.html
use alloc::{vec, vec::Vec};

use crate::error::Error;
use ckb_std::{ckb_constants::Source, syscalls::load_cell_data};

#[derive(Debug)]
pub struct MilestoneInfo {
    time: u64,                    // epoch, from the c-cell
    amount: u32,                  // give it to creator, the unit is CKB
    approval_ratio_threshold: u8, // 0 ~ 10, e.g. 6 means 60% of the votes in favor can be passed
}

#[derive(Debug)]
pub struct CrowdfundingInfo {
    pledge_time: u64,               // epoch, from the c-cell
    pledge_threshold: u32,          // threshold for starting a project, the unit is CKB
    startup_amount: u32,            // start-up capital for creators
    milestones: Vec<MilestoneInfo>, // milestones
}

impl CrowdfundingInfo {
    pub fn try_from(cell_dep_index: usize) -> Result<Self, Error> {
        let mut buf = [0u8; 146]; // MAX: 10 milestones
        let len = load_cell_data(&mut buf, 0, cell_dep_index, Source::CellDep)?;
        let mut milestones = vec![];
        for start in (16..len).step_by(13) {
            let time = as_u64_be(&buf[start..start + 8]);
            let amount = as_u32_be(&buf[start + 8..start + 12]);
            let approval_ratio_threshold = buf[start + 12];
            milestones.push(MilestoneInfo {
                time,
                amount,
                approval_ratio_threshold,
            })
        }
        let pledge_time = as_u64_be(&buf[0..8]);
        let pledge_threshold = as_u32_be(&buf[8..12]);
        let startup_amount = as_u32_be(&buf[12..16]);
        Ok(CrowdfundingInfo {
            pledge_time,
            pledge_threshold,
            startup_amount,
            milestones,
        })
    }
}

// pledge_time 8 bytes, pledge_threshold 4 bytes, startup_amount 4 bytes, milestones: [time 8bytes, amount 4bytes, approval_ratio_threshold 1 byte]

pub fn as_u32_be(array: &[u8]) -> u32 {
    ((array[0] as u32) << 24)
        + ((array[1] as u32) << 16)
        + ((array[2] as u32) << 8)
        + ((array[3] as u32) << 0)
}

pub fn as_u64_be(array: &[u8]) -> u64 {
    ((array[0] as u64) << 56)
        + ((array[1] as u64) << 48)
        + ((array[2] as u64) << 40)
        + ((array[3] as u64) << 32)
        + ((array[4] as u64) << 24)
        + ((array[5] as u64) << 16)
        + ((array[6] as u64) << 8)
        + ((array[7] as u64) << 0)
}
