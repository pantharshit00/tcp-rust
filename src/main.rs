use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::io::Read;
use std::net::Ipv4Addr;

mod tcp;

#[derive(Hash, Eq, PartialEq, Clone, Debug, Copy)]
struct Quad {
    src: (Ipv4Addr, u16),
    dst: (Ipv4Addr, u16),
}

fn main() {
    let mut connections: HashMap<Quad, tcp::Connection> = Default::default();
    let mut config = tun::Configuration::default();
    config
        .name("utun5")
        .address((192, 168, 0, 1))
        .netmask((255, 255, 255, 0))
        .destination((192, 168, 0, 2))
        .up();

    let mut dev = tun::create(&config).unwrap();
    let mut buf = [0; 1500];

    loop {
        let amount = dev.read(&mut buf).unwrap();

        match etherparse::Ipv4HeaderSlice::from_slice(&buf[4..amount]) {
            Ok(iph) => {
                let src = iph.source_addr();
                let dest = iph.destination_addr();
                let proto = iph.protocol();
                if proto != 0x06 {
                    // skip any non tcp packet
                    continue;
                }
                match etherparse::TcpHeaderSlice::from_slice(&buf[4 + iph.slice().len()..]) {
                    Ok(tcph) => {
                        let data_idx = iph.slice().len() + tcph.slice().len();
                        match connections.entry(Quad {
                            src: (src, tcph.source_port()),
                            dst: (dest, tcph.destination_port()),
                        }) {
                            Entry::Occupied(mut c) => {
                                c.get_mut()
                                    .on_packet(&mut dev, iph, tcph, &buf[data_idx..amount]);
                            }
                            Entry::Vacant(e) => {
                                if let Some(c) = tcp::Connection::accept(
                                    &mut dev,
                                    iph,
                                    tcph,
                                    &buf[data_idx..amount],
                                ) {
                                    e.insert(c);
                                }
                            }
                        }
                    }
                    Err(e) => println!("Ignoring weird tcp packet {}", e),
                }
            }
            Err(e) => println!("Ignoring packet {}", e),
        }
    }
}
