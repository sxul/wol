use clap::{Arg, Command};
use if_addrs::get_if_addrs;
use ipnet::Ipv4Net;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::path::PathBuf;

const WOL_PORT: u16 = 9;
const MAGIC_HEADER: [u8; 6] = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];

fn main() {
    let matches = Command::new("Wake on LAN")
        .version("1.0")
        .author("sxul07 <sxul07@hotmail.com>")
        .about("Wake up devices on the network")
        .arg(
            Arg::new("mac_address")
                .value_name("MAC_ADDRESS")
                .help("Target MAC address, e.g. 00:11:22:33:44:55")
                .num_args(0..)
                .required_unless_present("file"), 
        )
        .arg(
            Arg::new("file")
                .short('f')
                .long("file")
                .value_name("FILE")
                .value_parser(clap::value_parser!(PathBuf))
                .help("Reads target MAC addresses from a file, one per line. If this option is used, the mac_address option is ignored. Lines starting with # or // are ignored."),
        )
        .arg(
            Arg::new("net")
                .short('n')
                .long("net")
                .value_name("NET")
                .num_args(0..)
                .action(clap::ArgAction::Append)
                .help("Specify the network address to send the broadcast, use CIDR notation, e.g. 192.168.1.0/24"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enables verbose mode")
                .action(clap::ArgAction::SetTrue)
        )
        .get_matches();

    let verbose_mode = matches.get_flag("verbose");

    let mac_addresses = if let Some(file_path) = matches.get_one::<PathBuf>("file") {
        // if file not exist or is not file, it will return with error code
        if !file_path.exists() || !file_path.is_file() {
            println!(
                "Error: file not exist or is not file, input file path: {:?}",
                file_path
            );
            std::process::exit(1);
        }

        read_mac_addresses_from_file(file_path)
    } else {
        matches.get_many::<String>("mac_address")
        .unwrap()
        .map(|s| s.to_string())
        .collect()
    };

    let networks = if let Some(custom_net) = matches.get_many::<String>("net") {
        custom_net
            .into_iter()
            .map(|net| {
                match net.parse::<Ipv4Net>() {
                    Ok(v) => v,
                    Err(err) => {
                        println!(
                            "Error: {}. Correct address in CIDR notation, e.g. 192.168.1.0/24",
                            err
                        );
                        std::process::exit(1);
                    }
                }
            })
            .collect()
    } else {
        get_local_ip_nets()
    };

    for mac_address in &mac_addresses {
        // check the mac address format
        if mac_address.len() != 17 {
            println!(
                "Error: invalid MAC address length (should be 17), original MAC address: {}",
                mac_address
            );
            continue;
        }
        if !mac_address.contains(':') && !mac_address.contains('-') {
            println!(
                "Error: invalid MAC address format (should be separated by : or -), original MAC address: {}",
                mac_address
            );
            continue;
        }

        // uppercase the mac address
        let mac_address = mac_address.to_uppercase();

        send_wol_packet(&mac_address, &networks, verbose_mode);
    }
}

fn send_wol_packet(mac_address: &str, networks: &Vec<Ipv4Net>, verbose_mode: bool) {
    let socket =
        UdpSocket::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0)).unwrap();
    for broadcast_net in networks {
        match send_magic_packet(&socket, mac_address, &broadcast_net) {
            Ok(_) => {}
            Err(err) => {
                println!("Error: {}, original MAC address: {}", err, mac_address);
                break;
            }
        }
        if verbose_mode {
            println!(
                "Sent magic packet to {}, and broadcasted on {}",
                mac_address, broadcast_net
            );
        }
    }
}

fn send_magic_packet(
    socket: &UdpSocket,
    target_mac: &str,
    ip_net: &Ipv4Net,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut mac_parts: Vec<&str> = target_mac.split(':').collect();
    if mac_parts.len() != 6 {
        mac_parts = target_mac.split('-').collect();
        if mac_parts.len() != 6 {
            return Err("Invalid MAC address format, should be 6 parts".into());
        }
    }

    let mut mac_bytes = [0u8; 6];
    for (i, part) in mac_parts.iter().enumerate() {
        // parse the mac address
        if part.len() != 2 {
            return Err("Invalid MAC address format, should be 2 characters per part".into());
        }
        mac_bytes[i] = match u8::from_str_radix(part, 16) {
            Ok(v) => v,
            Err(_) => return Err("Invalid MAC address format, should be hex".into()),
        }
    }

    let mut magic_packet = [0u8; 102];

    magic_packet[..6].copy_from_slice(&MAGIC_HEADER);

    for i in 0..16 {
        magic_packet[6 + i * 6..6 + (i + 1) * 6].copy_from_slice(&mac_bytes);
    }

    let broadcast_address = ip_net.broadcast();

    let dest = SocketAddr::new(broadcast_address.into(), WOL_PORT);

    socket.set_broadcast(true)?;
    socket.send_to(&magic_packet, dest)?;

    Ok(())
}

fn get_local_ip_nets() -> Vec<Ipv4Net> {
    let if_addrs = get_if_addrs().unwrap();
    let mut ip_nets = Vec::new();

    for if_addr in if_addrs {
        if let if_addrs::IfAddr::V4(if_v4_addr) = if_addr.addr {
            let ip = if_v4_addr.ip;
            let netmask = if_v4_addr.netmask;
            let prefix_len = netmask
                .octets()
                .iter()
                .fold(0, |acc, &octet| acc + octet.count_ones() as u8);
            let ip_net = Ipv4Net::new(ip, prefix_len).unwrap();
            ip_nets.push(ip_net);
        }
    }

    ip_nets
}

fn read_mac_addresses_from_file(file_path: &PathBuf) -> Vec<String> {
    let file = File::open(file_path).unwrap();
    let reader = BufReader::new(file);
    let mut mac_addresses = Vec::new();

    for line in reader.lines() {
        let line = match line {
            Ok(v) => v.trim().to_string(),
            Err(_) => break, // if read line error, break the loop
        };
        // skip empty lines
        if line.is_empty() {
            continue;
        }
        // skip line starts with #
        if line.starts_with('#') {
            continue;
        }
        // skip line starts with //
        if line.starts_with("//") {
            continue;
        }
        // check the mac address length (should be 17)
        if line.len() != 17 {
            continue;
        }
        // invalid MAC address format (should be separated by :)
        if !line.contains(':') && !line.contains('-') {
            continue;
        }
        mac_addresses.push(line);
    }

    mac_addresses
}
