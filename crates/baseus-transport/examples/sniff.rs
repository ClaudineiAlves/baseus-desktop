// Passive protocol sniffer: connects to the earbuds and timestamps every notification,
// so state changes driven from *another* client (the vendor Android app) can be
// correlated against a known sequence of taps.
//
//   cargo run -p baseus-transport --example sniff
//
// Prints wall-clock time, the decoded frame and raw bytes. Runs until Ctrl-C.
use baseus_protocol::{framing::Frame, BaseusModel};
use baseus_transport::{win::ble::GattTransport, DeviceMatch};

fn stamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    let secs = now.as_secs() % 86400;
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        secs / 3600,
        (secs % 3600) / 60,
        secs % 60,
        now.subsec_millis()
    )
}

#[tokio::main]
async fn main() {
    let entries: Vec<DeviceMatch> = BaseusModel::all()
        .iter()
        .flat_map(|m| {
            let (notify_uuid, write_uuid) = m.gatt_uuids();
            let service_uuid = m.service_uuid();
            m.advertising_names().iter().map(move |&name| DeviceMatch {
                name,
                service_uuid,
                notify_uuid,
                write_uuid,
            })
        })
        .collect();

    let (mut transport, idx) = match GattTransport::connect_any(&entries).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("connection failed: {e}");
            std::process::exit(1);
        }
    };
    eprintln!(
        "connected to {} — listening, Ctrl-C to stop",
        entries[idx].name
    );

    // Handshake only; this tool never changes device state, so anything observed is
    // attributable to the other client.
    let _ = transport.send(&[0xBA, 0x05, 0x00]).await;

    loop {
        match transport.next_notification().await {
            Ok(data) => {
                let raw = data
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<Vec<_>>()
                    .join(" ");
                match Frame::decode(&data) {
                    Ok(f) => println!(
                        "{}  cmd={:#04x}  payload={:02x?}   | {raw}",
                        stamp(),
                        f.cmd,
                        f.payload
                    ),
                    Err(_) => println!("{}  (unframed)                      | {raw}", stamp()),
                }
            }
            Err(e) => {
                eprintln!("{}  stream ended: {e}", stamp());
                break;
            }
        }
    }
}
