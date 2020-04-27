use std::io;
pub enum State {
    Closed,
    Listen,
    SynRcvd,
    // Estab,
}

pub struct Connection {
    state: State,
    send: SendSequence,
    recv: RecvSequenceSpace,
}

struct SendSequence {
    una: u32,
    nxt: u32,
    wnd: u16,
    up: bool,
    wl1: usize,
    wl2: usize,
    iss: u32,
}

struct RecvSequenceSpace {
    nxt: u32,
    wnd: u16,
    up: bool,
    irs: u32,
}

impl Default for State {
    fn default() -> Self {
        Self::Listen
    }
}

// impl Default for Connection {
// fn default() -> Self {
// Connection {
// state: State::Listen,
// }
// }
// }

impl Connection {
    pub fn accept(
        nic: &mut dyn tun::Device,
        iph: etherparse::Ipv4HeaderSlice,
        tcph: etherparse::TcpHeaderSlice,
        data: &[u8],
    ) -> Option<Self> {
        let mut buf = [0; 1500];
        if !tcph.syn() {
            // how the hell you made in
            return None;
        }
        let iss = 0;
        let mut c = Connection {
            state: State::SynRcvd,
            send: SendSequence {
                iss,
                una: iss,
                nxt: iss + 1,
                wnd: 10,
                up: false,
                wl1: 0,
                wl2: 0,
            },
            recv: RecvSequenceSpace {
                irs: tcph.sequence_number(),
                nxt: tcph.sequence_number() + 1,
                wnd: tcph.window_size(),
                up: false,
            },
        };
        // keep track of sender info

        let mut syn_ack = etherparse::TcpHeader::new(
            tcph.destination_port(),
            tcph.source_port(),
            c.send.iss,
            c.send.wnd,
        );
        syn_ack.acknowledgment_number = c.recv.nxt;
        syn_ack.syn = true;
        syn_ack.ack = true;
        let ip = etherparse::Ipv4Header::new(
            syn_ack.header_len(),
            64,
            etherparse::IpTrafficClass::Tcp,
            iph.destination_addr().octets(),
            iph.source_addr().octets(),
        );
        println!("got ip header: \n {:02x?} ", iph);
        println!("got tcp header: \n {:02x?} ", tcph);

        let unwritten = {
            let mut unwritten = &mut buf[..];

            ip.write(&mut unwritten).unwrap();
            syn_ack.write(&mut unwritten).unwrap();
            unwritten.len()
        };
        println!(
            "responding with some bytes {:02x?}",
            &buf[..buf.len() - unwritten]
        );
        let something = nic.write(&mut buf[..unwritten]).unwrap();
        println!("Wrote {}", something);
        Some(c)
    }

    pub fn on_packet(
        &mut self,
        nic: &mut dyn tun::Device,
        iph: etherparse::Ipv4HeaderSlice,
        tcph: etherparse::TcpHeaderSlice,
        data: &[u8],
    ) -> Option<Self> {
        unimplemented!()
    }
}
