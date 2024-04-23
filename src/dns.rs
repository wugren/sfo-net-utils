use std::io;
use std::net::IpAddr;

#[cfg(unix)]
fn system_nameservers() -> io::Result<Vec<IpAddr>> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let f = File::open("/etc/resolv.conf")?;
    let reader = BufReader::new(f);

    let mut nameservers = Vec::new();
    for line in reader.lines() {
        let line = line?;

        if let Some(nameserver_str) = line.strip_prefix("nameserver ") {
            let ip: Result<std::net::IpAddr, _> = nameserver_str.parse();

            match ip {
                Ok(ip) => nameservers.push(ip),
                Err(e)  => warn!("Failed to parse nameserver line {:?}: {}", line, e),
            }
        }
    }

    Ok(nameservers)
}


#[cfg(windows)]
#[allow(unused)]
fn system_nameservers() -> io::Result<Vec<IpAddr>> {
    use std::net::{IpAddr, UdpSocket};

    #[derive(Debug, PartialEq)]
    enum ForceIPFamily {
        V4,
        V6,
        None,
    }

    fn get_ipv4() -> io::Result<IpAddr> {
        let s = UdpSocket::bind("0.0.0.0:0")?;
        s.connect("8.8.8.8:53")?;
        let addr = s.local_addr()?;
        Ok(addr.ip())
    }

    fn get_ipv6() -> io::Result<IpAddr> {
        let s = UdpSocket::bind("[::1]:0")?;
        s.connect("[2001:4860:4860::8888]:53")?;
        let addr = s.local_addr()?;
        Ok(addr.ip())
    }

    let force_ip_family: ForceIPFamily = ForceIPFamily::None;
    let ip = match force_ip_family {
        ForceIPFamily::V4 => get_ipv4().ok(),
        ForceIPFamily::V6 => get_ipv6().ok(),
        ForceIPFamily::None => get_ipv6().or(get_ipv4()).ok(),
    };

    let mut dns_list = Vec::new();
    let adapters = ipconfig::get_adapters().map_err(|e| {
        io::Error::new(io::ErrorKind::Other, e)
    })?;
    let active_adapters = adapters.iter().filter(|a| {
        a.oper_status() == ipconfig::OperStatus::IfOperStatusUp && !a.gateways().is_empty()
    });


    if let Some(dns_server) = active_adapters
        .clone()
        .find(|a| ip.map(|ip| a.ip_addresses().contains(&ip)).unwrap_or(false))
    {
        for dns_server in dns_server.dns_servers() {
            dns_list.push(dns_server.clone());
        }
    }

    // Fallback
    if let Some(dns_server) = active_adapters
        .filter(|a| !ip.map(|ip| a.ip_addresses().contains(&ip)).unwrap_or(false))
        .flat_map(|a| a.dns_servers())
        .find(|d| (d.is_ipv4() && force_ip_family != ForceIPFamily::V6) || d.is_ipv6())
    {
        dns_list.push(dns_server.clone());
    }

    Ok(dns_list)
}

#[test]
fn test_dns() {
    let dns = system_nameservers().unwrap();
    println!("dns:{:?}", dns);
}
