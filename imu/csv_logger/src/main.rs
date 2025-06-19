use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
mod args;
use args::Parser;
use csv;
use driver;
use serde::{Deserialize, Serialize};

use signal_hook;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use driver::protocol::DEFAULT_BAUDRATE;
use driver::protocol::VID_PID as DEFAULT_VID_PID;

const VERSION: driver::AdisVersion = driver::AdisVersion::ADIS16505_1BMLZ;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
struct LogOutput {
    timestamp_pc: f64,
    #[serde(flatten)]
    data: driver::protocol::adis::BurstData,
}

type MainResult = Result<(), ()>;
fn main() -> MainResult {
    let args = args::Args::parse();

    let timeout = args
        .timeout_ns
        .map(|nanos| driver::Duration::from_nanos(nanos));

    let mut adis = if let Some(path) = args.device {
        driver::AdisDevice::from_device_name(path, args.baud_rate, VERSION, timeout)
    } else {
        driver::AdisDevice::from_vid_pid(args.vid, args.pid, args.baud_rate, VERSION, timeout)
    }
    .expect("Could not connect to device.");

    let log_path = Path::new(args.log_path.as_str())
        .with_file_name(args.log_name)
        .with_extension("csv");

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

    let mut writer = csv::WriterBuilder::new()
        .buffer_capacity(2048)
        .delimiter(b',')
        .has_headers(true)
        .from_path(log_path)
        .expect("Could not open log file.");

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
            writer
                .serialize(LogOutput {
                    timestamp_pc: reception_time
                        .duration_since(UNIX_EPOCH)
                        .expect("Timing error in PC.")
                        .as_secs_f64(),
                    data: m,
                })
                .expect("Could not write into file.");
        }
    }

    writer.flush().expect("Writer was not able to flush data.");

    return Ok(());
}
