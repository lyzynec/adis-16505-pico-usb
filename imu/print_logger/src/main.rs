use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs::File;
use std::io::{BufWriter, Write};
mod args;
use args::Parser;
use driver;
use serde::{Deserialize, Serialize};

use signal_hook;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use driver::protocol::DEFAULT_BAUDRATE;
use driver::protocol::VID_PID as DEFAULT_VID_PID;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
struct LogOutput {
    timestamp_pc: f64,
    data: driver::protocol::adis::BurstData,
}

type MainResult = Result<(), ()>;
fn main() -> MainResult {
    let args = args::Args::parse();

    let timeout = args
        .timeout_ns
        .map(|nanos| driver::Duration::from_nanos(nanos));

    let version = driver::AdisVersion::from_id(args.board_id, args.board_version)
        .expect("Unknown board");

    let mut adis = if let Some(path) = args.device {
        driver::AdisDevice::from_device_name(path, args.baud_rate, version, timeout)
    } else {
        driver::AdisDevice::from_vid_pid(args.vid, args.pid, args.baud_rate, version, timeout)
    }
    .expect("Could not connect to device.");

    let log_path = Path::new(args.log_path.as_str())
        .with_file_name(args.log_name)
        .with_extension("txt");

    adis.send_restart().expect("Could not restart device.");

    let cfg_burst_mode = match args.burst_mode {
        16 => driver::protocol::cfg::CFG::Burst32(driver::protocol::cfg::Burst32::Disabled),
        32 => driver::protocol::cfg::CFG::Burst32(driver::protocol::cfg::Burst32::Enabled),
        _ => panic!("Invalid burst mode, only 16 and 32 are valid options."),
    };
    adis.send_config(cfg_burst_mode)
        .expect("Could not set burst mode.");

    let cfg_burst_sel = match args.burst_sel {
        0 => driver::protocol::cfg::CFG::BurstSel(driver::protocol::cfg::BurstSel::Sel0),
        1 => driver::protocol::cfg::CFG::BurstSel(driver::protocol::cfg::BurstSel::Sel1),
        _ => panic!("Invalid burst sel, only 0 and 1 are valid options."),
    };
    adis.send_config(cfg_burst_sel)
        .expect("Could not set burst sel.");

    let cfg_burst_en = driver::protocol::cfg::CFG::BurstEn(true);
    adis.send_config(cfg_burst_en)
        .expect("Could not enable burst.");


    let out_file = File::create(log_path).expect("Could not create file.");
    let mut writer = BufWriter::new(out_file);

    let end_manual = Arc::new(AtomicBool::new(false));
    let end_systemd = Arc::new(AtomicBool::new(false));

    signal_hook::flag::register(
        signal_hook::consts::SIGINT,
        std::sync::Arc::clone(&end_manual),
    )
    .expect("Could not hook manual signal.");

    signal_hook::flag::register(
        signal_hook::consts::SIGTERM,
        std::sync::Arc::clone(&end_systemd),
    )
    .expect("Could not hook systemd signal.");

    while !end_manual.load(Ordering::Relaxed) && !end_systemd.load(Ordering::Relaxed) {
        let messages = adis.expect_burst().expect("There was error while reading.");
        let reception_time = SystemTime::now();

        for m in messages {
            let out = LogOutput {
                timestamp_pc: reception_time
                    .duration_since(UNIX_EPOCH)
                    .expect("Timing error in PC.")
                    .as_secs_f64(),
                data: m,
            };
            writer.write(format!("{:#?}\n", out).as_bytes()).expect("Could not write to file.");
        }
    }

    writer.flush().expect("Writer was not able to flush data.");

    return Ok(());
}
