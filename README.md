# Wake On Lan
Wake-on-LAN (WOL) can be useful if you need to remotely wake up a computer on your local network. WOL is a remote power management technology that allows you to send a special "magic packet" over the network to wake up a computer that is asleep.

This project is a WOL command line tool written in Rust. It allows you to send WOL magic packets over your local network to wake up a specific computer that is in sleep mode. This tool has the following features

Supports reading target MAC addresses from command line arguments or a file.
Supports using the `-n` or `--net` option to specify the network interface to use, so you can choose the right one in case of multiple network interfaces.
Supports waking up multiple computers by sending magic packets to multiple broadcast networks.

# How to Use

## Install

### Build from source

Make sure that you have the Rust programming language installed before you do this build. Then run the following command to build the project:

```bash
cargo build --release
```

After the build is complete, you can find the executable file in the `target/release` directory.

## Usage

To use this tool, simply enter the following command in your terminal:

```bash
wol [OPTIONS] <MAC_ADDRESS>
```

Where MAC_ADDRESS is the MAC address of the computer you want to wake up, separated by : or - between the bytes. If you need to wake up multiple computers, you can specify multiple MAC addresses in the command line arguments.

This tool supports the following command line options

`-h`, `--help`: Display help information.

`-f`, `--file FILE`: Reads MAC addresses from the specified file, one address per line. The addresses in the file should be separated by `:` between bytes.

`-n`, `--net IP_ADDRESS`: Specify the IP address of the network interface to use. If this option is not specified, **all network interfaces** will be used.

`-v`, `--verbose`: Display verbose information, such as the network interface used to send the magic packet.

`-V`, `--version`: Display version information.

### Examples

```bash
# Wake up a single computer
wol 01:23:45:67:89:ab

# Wake up multiple computers (not implemented yet)
wol 01:23:45:67:89:ab 23-45-67-89-ab-cd

# Read MAC addresses from a file and wake up computers
wol -f addresses.txt

# Specify a network interface and wake up a computer
wol -n 192.168.1.10/24 01:23:45:67:89:ab
```

# License

This project is released under the MIT licence. See the [LICENSE](LICENSE) file for more details.
