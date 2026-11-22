use std::net::{Ipv4Addr, Ipv6Addr};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub struct DnsConfig {
    pub bind_address: String,
    pub port: u16,
    pub tld: String,
    pub target_ipv4: Ipv4Addr,
    pub target_ipv6: Ipv6Addr,
    pub upstream: Option<String>,
}

impl Default for DnsConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1".to_string(),
            port: 5353,
            tld: "test".to_string(),
            target_ipv4: Ipv4Addr::new(127, 0, 0, 1),
            target_ipv6: Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1),
            upstream: Some("1.1.1.1:53".to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DnsQuestion {
    pub name: String,
    pub qtype: u16,
    pub qclass: u16,
    pub question_bytes_len: usize,
}

pub struct DnsServer;

impl DnsServer {
    /// Parses the QNAME and question section from a raw DNS query packet.
    pub fn parse_question(buf: &[u8]) -> Option<DnsQuestion> {
        if buf.len() < 12 {
            return None;
        }

        let mut offset = 12;
        let mut labels = Vec::new();

        while offset < buf.len() {
            let len = buf[offset] as usize;
            if len == 0 {
                offset += 1;
                break;
            }
            if len & 0xc0 == 0xc0 {
                // Compression pointer in query question is invalid in standard queries
                return None;
            }
            offset += 1;
            if offset + len > buf.len() {
                return None;
            }
            let label_str = String::from_utf8_lossy(&buf[offset..offset + len]);
            labels.push(label_str.to_string());
            offset += len;
        }

        if offset + 4 > buf.len() {
            return None;
        }

        let qtype = u16::from_be_bytes([buf[offset], buf[offset + 1]]);
        let qclass = u16::from_be_bytes([buf[offset + 2], buf[offset + 3]]);
        let question_bytes_len = offset + 4 - 12;

        Some(DnsQuestion {
            name: labels.join(".").to_lowercase(),
            qtype,
            qclass,
            question_bytes_len,
        })
    }

    /// Builds a DNS response packet for a wildcard TLD match (or empty NOERROR for non-address types).
    pub fn build_wildcard_response(
        req: &[u8],
        question: &DnsQuestion,
        ipv4: Ipv4Addr,
        ipv6: Ipv6Addr,
    ) -> Vec<u8> {
        let mut resp = Vec::with_capacity(128);

        // 1. Transaction ID (preserve original)
        resp.push(req[0]);
        resp.push(req[1]);

        // 2. Flags: QR=1 (response), AA=1 (authoritative), RD=preserved, RA=1, RCODE=0
        let req_flags = u16::from_be_bytes([req[2], req[3]]);
        let rd_bit = req_flags & 0x0100; // Recursion Desired
        let resp_flags: u16 = 0x8400 | rd_bit | 0x0080; // QR=1, AA=1, RA=1, RCODE=0
        resp.extend_from_slice(&resp_flags.to_be_bytes());

        // 3. Counts: QDCOUNT = 1, ANCOUNT = (1 if A or AAAA else 0), NSCOUNT = 0, ARCOUNT = 0
        let ancount: u16 = match question.qtype {
            1 | 28 => 1, // A or AAAA
            _ => 0,
        };
        resp.extend_from_slice(&1u16.to_be_bytes()); // QDCOUNT
        resp.extend_from_slice(&ancount.to_be_bytes()); // ANCOUNT
        resp.extend_from_slice(&0u16.to_be_bytes()); // NSCOUNT
        resp.extend_from_slice(&0u16.to_be_bytes()); // ARCOUNT

        // 4. Copy Question section verbatim
        let question_end = 12 + question.question_bytes_len;
        if question_end <= req.len() {
            resp.extend_from_slice(&req[12..question_end]);
        }

        // 5. Answer section (if A or AAAA)
        if question.qtype == 1 {
            // Type A (IPv4)
            resp.extend_from_slice(&[0xc0, 0x0c]); // Pointer to QNAME at byte 12
            resp.extend_from_slice(&1u16.to_be_bytes()); // TYPE A
            resp.extend_from_slice(&1u16.to_be_bytes()); // CLASS IN
            resp.extend_from_slice(&60u32.to_be_bytes()); // TTL 60 seconds
            resp.extend_from_slice(&4u16.to_be_bytes()); // RDLENGTH 4
            resp.extend_from_slice(&ipv4.octets()); // RDATA
        } else if question.qtype == 28 {
            // Type AAAA (IPv6)
            resp.extend_from_slice(&[0xc0, 0x0c]); // Pointer to QNAME at byte 12
            resp.extend_from_slice(&28u16.to_be_bytes()); // TYPE AAAA
            resp.extend_from_slice(&1u16.to_be_bytes()); // CLASS IN
            resp.extend_from_slice(&60u32.to_be_bytes()); // TTL 60 seconds
            resp.extend_from_slice(&16u16.to_be_bytes()); // RDLENGTH 16
            resp.extend_from_slice(&ipv6.octets()); // RDATA
        }

        resp
    }

    /// Builds an NXDOMAIN response (domain not found).
    pub fn build_nxdomain_response(req: &[u8], question: &DnsQuestion) -> Vec<u8> {
        let mut resp = Vec::with_capacity(64);
        resp.push(req[0]);
        resp.push(req[1]);

        let req_flags = u16::from_be_bytes([req[2], req[3]]);
        let rd_bit = req_flags & 0x0100;
        let resp_flags: u16 = 0x8003 | rd_bit; // QR=1, RCODE=3 (NXDOMAIN)
        resp.extend_from_slice(&resp_flags.to_be_bytes());

        resp.extend_from_slice(&1u16.to_be_bytes()); // QDCOUNT
        resp.extend_from_slice(&0u16.to_be_bytes()); // ANCOUNT
        resp.extend_from_slice(&0u16.to_be_bytes()); // NSCOUNT
        resp.extend_from_slice(&0u16.to_be_bytes()); // ARCOUNT

        let question_end = 12 + question.question_bytes_len;
        if question_end <= req.len() {
            resp.extend_from_slice(&req[12..question_end]);
        }
        resp
    }

    /// Forwards an external query to the configured upstream DNS server.
    pub async fn forward_to_upstream(req: &[u8], upstream_addr: &str) -> Option<Vec<u8>> {
        let socket = UdpSocket::bind("0.0.0.0:0").await.ok()?;
        socket.connect(upstream_addr).await.ok()?;
        socket.send(req).await.ok()?;

        let mut buf = [0u8; 512];
        let timeout = Duration::from_millis(1500);

        match tokio::time::timeout(timeout, socket.recv(&mut buf)).await {
            Ok(Ok(n)) => Some(buf[..n].to_vec()),
            _ => None,
        }
    }

    /// Handles a single incoming DNS request and returns the response bytes.
    pub async fn handle_packet(req: &[u8], config: &DnsConfig) -> Option<Vec<u8>> {
        let question = Self::parse_question(req)?;
        let tld_suffix = format!(".{}", config.tld.to_lowercase());

        let matches_tld = question.name == config.tld.to_lowercase()
            || question.name.ends_with(&tld_suffix);

        if matches_tld {
            Some(Self::build_wildcard_response(
                req,
                &question,
                config.target_ipv4,
                config.target_ipv6,
            ))
        } else if let Some(ref upstream) = config.upstream {
            if let Some(forwarded) = Self::forward_to_upstream(req, upstream).await {
                Some(forwarded)
            } else {
                Some(Self::build_nxdomain_response(req, &question))
            }
        } else {
            Some(Self::build_nxdomain_response(req, &question))
        }
    }

    /// Runs the UDP DNS server loop until receiving a cancellation signal.
    pub async fn run_server(
        config: DnsConfig,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) -> Result<(), std::io::Error> {
        let bind_str = format!("{}:{}", config.bind_address, config.port);
        let socket = Arc::new(UdpSocket::bind(&bind_str).await?);
        let mut buf = [0u8; 512];

        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    break;
                }
                res = socket.recv_from(&mut buf) => {
                    if let Ok((len, src)) = res {
                        let req_bytes = buf[..len].to_vec();
                        let sock_clone = socket.clone();
                        let cfg_clone = config.clone();

                        tokio::spawn(async move {
                            if let Some(resp) = Self::handle_packet(&req_bytes, &cfg_clone).await {
                                let _ = sock_clone.send_to(&resp, src).await;
                            }
                        });
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_build_wildcard_a_record() {
        // Construct standard DNS query for 'loja.test' Type A (1) Class IN (1)
        let query = vec![
            0x12, 0x34, // ID
            0x01, 0x00, // Flags (RD=1)
            0x00, 0x01, // QDCOUNT=1
            0x00, 0x00, // ANCOUNT=0
            0x00, 0x00, // NSCOUNT=0
            0x00, 0x00, // ARCOUNT=0
            // QNAME: 4 'l' 'o' 'j' 'a' 4 't' 'e' 's' 't' 0
            0x04, b'l', b'o', b'j', b'a',
            0x04, b't', b'e', b's', b't',
            0x00,
            0x00, 0x01, // QTYPE: A (1)
            0x00, 0x01, // QCLASS: IN (1)
        ];

        let q = DnsServer::parse_question(&query).expect("Question must parse");
        assert_eq!(q.name, "loja.test");
        assert_eq!(q.qtype, 1);
        assert_eq!(q.qclass, 1);

        let resp = DnsServer::build_wildcard_response(
            &query,
            &q,
            Ipv4Addr::new(127, 0, 0, 1),
            Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1),
        );

        // Check Transaction ID preserved
        assert_eq!(resp[0], 0x12);
        assert_eq!(resp[1], 0x34);

        // Check Flags: QR=1 (bit 15 set)
        assert_eq!(resp[2] & 0x80, 0x80);

        // Check ANCOUNT = 1
        assert_eq!(u16::from_be_bytes([resp[6], resp[7]]), 1);

        // Check RDATA is 127.0.0.1 at the end of the packet
        let rdata = &resp[resp.len() - 4..];
        assert_eq!(rdata, &[127, 0, 0, 1]);
    }

    #[test]
    fn test_wildcard_aaaa_record() {
        let query = vec![
            0x56, 0x78,
            0x01, 0x00,
            0x00, 0x01,
            0x00, 0x00,
            0x00, 0x00,
            0x00, 0x00,
            0x03, b'a', b'p', b'p',
            0x04, b't', b'e', b's', b't',
            0x00,
            0x00, 0x1c, // QTYPE: AAAA (28)
            0x00, 0x01,
        ];

        let q = DnsServer::parse_question(&query).expect("Question must parse");
        assert_eq!(q.name, "app.test");
        assert_eq!(q.qtype, 28);

        let resp = DnsServer::build_wildcard_response(
            &query,
            &q,
            Ipv4Addr::new(127, 0, 0, 1),
            Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1),
        );

        assert_eq!(resp[0], 0x56);
        assert_eq!(u16::from_be_bytes([resp[6], resp[7]]), 1);
        let rdata = &resp[resp.len() - 16..];
        assert_eq!(rdata, &Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1).octets());
    }

    #[tokio::test]
    async fn test_handle_packet_wildcard_resolution() {
        let query = vec![
            0xaa, 0xbb,
            0x01, 0x00,
            0x00, 0x01,
            0x00, 0x00,
            0x00, 0x00,
            0x00, 0x00,
            0x06, b'm', b'y', b'p', b'r', b'o', b'j',
            0x04, b't', b'e', b's', b't',
            0x00,
            0x00, 0x01,
            0x00, 0x01,
        ];

        let config = DnsConfig::default();
        let resp = DnsServer::handle_packet(&query, &config).await.expect("Response must be generated");
        assert_eq!(resp[0], 0xaa);
        assert_eq!(resp[1], 0xbb);
        assert_eq!(&resp[resp.len() - 4..], &[127, 0, 0, 1]);
    }
}
