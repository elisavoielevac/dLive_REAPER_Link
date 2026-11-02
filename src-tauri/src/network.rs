//! MixRack sockets must use an explicitly selected physical Ethernet adapter.
use serde::Serialize;
use std::{
    net::{IpAddr, SocketAddr, TcpStream},
    time::Duration,
};

#[derive(Clone, Debug, Serialize)]
pub struct EthernetInterface {
    pub id: String,
    pub name: String,
    pub bsd_name: String,
    pub addresses: Vec<IpAddr>,
    pub ready: bool,
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use core_foundation::{
        array::{CFArray, CFArrayRef},
        base::TCFType,
        string::{CFString, CFStringRef},
    };
    use std::{
        collections::HashMap,
        ffi::{c_void, CStr},
    };

    #[link(name = "SystemConfiguration", kind = "framework")]
    extern "C" {
        fn SCNetworkInterfaceCopyAll() -> CFArrayRef;
        fn SCNetworkInterfaceGetInterfaceType(interface: *const c_void) -> CFStringRef;
        fn SCNetworkInterfaceGetBSDName(interface: *const c_void) -> CFStringRef;
        fn SCNetworkInterfaceGetLocalizedDisplayName(interface: *const c_void) -> CFStringRef;
        fn SCNetworkInterfaceGetHardwareAddressString(interface: *const c_void) -> CFStringRef;
        static kSCNetworkInterfaceTypeEthernet: CFStringRef;
    }

    // Borrowed SC strings are retained by the wrapper, then released after copying.
    unsafe fn string(value: CFStringRef) -> String {
        if value.is_null() {
            String::new()
        } else {
            CFString::wrap_under_get_rule(value).to_string()
        }
    }

    pub fn interfaces() -> Result<Vec<EthernetInterface>, String> {
        let mut addresses: HashMap<String, (bool, Vec<IpAddr>)> = HashMap::new();
        // getifaddrs owns the linked list until freeifaddrs; copy all values first.
        unsafe {
            let mut head = std::ptr::null_mut();
            if libc::getifaddrs(&mut head) != 0 {
                return Err(std::io::Error::last_os_error().to_string());
            }
            let mut current = head;
            while let Some(entry) = current.as_ref() {
                let name = CStr::from_ptr(entry.ifa_name)
                    .to_string_lossy()
                    .into_owned();
                let state = addresses.entry(name).or_default();
                state.0 = entry.ifa_flags & (libc::IFF_UP | libc::IFF_RUNNING) as u32
                    == (libc::IFF_UP | libc::IFF_RUNNING) as u32;
                if !entry.ifa_addr.is_null() {
                    let ip = match (*entry.ifa_addr).sa_family as i32 {
                        libc::AF_INET => Some(IpAddr::V4(std::net::Ipv4Addr::from(
                            (*(entry.ifa_addr as *const libc::sockaddr_in))
                                .sin_addr
                                .s_addr
                                .to_ne_bytes(),
                        ))),
                        libc::AF_INET6 => Some(IpAddr::V6(std::net::Ipv6Addr::from(
                            (*(entry.ifa_addr as *const libc::sockaddr_in6))
                                .sin6_addr
                                .s6_addr,
                        ))),
                        _ => None,
                    };
                    if let Some(ip) = ip {
                        if !state.1.contains(&ip) {
                            state.1.push(ip);
                        }
                    }
                }
                current = entry.ifa_next;
            }
            libc::freeifaddrs(head);
        }
        unsafe {
            let raw = SCNetworkInterfaceCopyAll();
            if raw.is_null() {
                return Err("Could not list Ethernet interfaces".into());
            }
            let array: CFArray<*const c_void> = CFArray::wrap_under_create_rule(raw);
            let ethernet = string(kSCNetworkInterfaceTypeEthernet);
            let mut result = Vec::new();
            for item in array.iter() {
                let interface = *item;
                // BSD en* names alone cannot distinguish Wi-Fi from Ethernet.
                if string(SCNetworkInterfaceGetInterfaceType(interface)) != ethernet {
                    continue;
                }
                let id =
                    string(SCNetworkInterfaceGetHardwareAddressString(interface)).to_lowercase();
                let bsd_name = string(SCNetworkInterfaceGetBSDName(interface));
                if id.is_empty() || bsd_name.is_empty() {
                    continue;
                }
                let (ready, ips) = addresses.remove(&bsd_name).unwrap_or_default();
                result.push(EthernetInterface {
                    id,
                    name: string(SCNetworkInterfaceGetLocalizedDisplayName(interface)),
                    bsd_name,
                    addresses: ips,
                    ready,
                });
            }
            result.sort_by(|a, b| a.bsd_name.cmp(&b.bsd_name));
            Ok(result)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn wifi_hardware_id_is_rejected_even_if_supplied_directly() {
            let allowed = interfaces().unwrap();
            let mut checked = 0;
            unsafe {
                let raw = SCNetworkInterfaceCopyAll();
                assert!(!raw.is_null());
                let all: CFArray<*const c_void> = CFArray::wrap_under_create_rule(raw);
                for item in all.iter() {
                    if string(SCNetworkInterfaceGetInterfaceType(*item)) != "IEEE80211" {
                        continue;
                    }
                    let id =
                        string(SCNetworkInterfaceGetHardwareAddressString(*item)).to_lowercase();
                    assert!(!allowed.iter().any(|interface| interface.id == id));
                    assert!(super::super::connect(
                        &id,
                        "127.0.0.1:51325".parse().unwrap(),
                        Duration::from_millis(10)
                    )
                    .is_err());
                    checked += 1;
                }
            }
            eprintln!("Verified {checked} Wi-Fi adapters are rejected");
        }
    }
}

pub fn interfaces() -> Result<Vec<EthernetInterface>, String> {
    #[cfg(target_os = "macos")]
    {
        macos::interfaces()
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err("Ethernet interface selection requires macOS".into())
    }
}

fn selected_index(id: &str, destination: IpAddr) -> Result<std::num::NonZeroU32, String> {
    if id.is_empty() {
        return Err("Select an Ethernet interface in Setup and save the scene source. Wi-Fi is not supported.".into());
    }
    let interface = interfaces()?.into_iter().find(|interface| interface.id == id)
        .ok_or("Selected Ethernet adapter is unavailable. Reconnect it or select another Ethernet interface in Setup.")?;
    if !interface.ready
        || !interface
            .addresses
            .iter()
            .any(|ip| ip.is_ipv4() == destination.is_ipv4())
    {
        return Err(format!(
            "Ethernet interface {} is not ready or has no suitable IP address",
            interface.bsd_name
        ));
    }
    let name = std::ffi::CString::new(interface.bsd_name).map_err(|e| e.to_string())?;
    // Resolve the current index each time: indices can change after unplug/replug.
    std::num::NonZeroU32::new(unsafe { libc::if_nametoindex(name.as_ptr()) })
        .ok_or_else(|| "Selected Ethernet interface disappeared".into())
}

pub fn connect(id: &str, address: SocketAddr, timeout: Duration) -> Result<TcpStream, String> {
    // Only simulated unit-test connections may bypass the physical Ethernet requirement.
    #[cfg(test)]
    if id == "test-loopback" && address.ip().is_loopback() {
        return TcpStream::connect_timeout(&address, timeout).map_err(|e| e.to_string());
    }
    let socket = bound_socket(id, address)?;
    socket
        .connect_timeout(&address.into(), timeout)
        .map_err(|e| e.to_string())?;
    Ok(socket.into())
}

fn bound_socket(id: &str, address: SocketAddr) -> Result<socket2::Socket, String> {
    let index = selected_index(id, address.ip())?;
    let socket = socket2::Socket::new(
        socket2::Domain::for_address(address),
        socket2::Type::STREAM,
        Some(socket2::Protocol::TCP),
    )
    .map_err(|e| e.to_string())?;
    if address.is_ipv4() {
        socket.bind_device_by_index_v4(Some(index))
    } else {
        socket.bind_device_by_index_v6(Some(index))
    }
    .map_err(|e| format!("Could not bind MixRack connection to Ethernet: {e}"))?;
    Ok(socket)
}

pub fn check_connection(id: &str, stream: &TcpStream) -> Result<(), String> {
    let address = stream.peer_addr().map_err(|e| e.to_string())?;
    #[cfg(test)]
    if id == "test-loopback" && address.ip().is_loopback() {
        return Ok(());
    }
    let expected = selected_index(id, address.ip())?;
    let socket = socket2::SockRef::from(stream);
    let bound = if address.is_ipv4() {
        socket.device_index_v4()
    } else {
        socket.device_index_v6()
    }
    .map_err(|e| e.to_string())?;
    if bound != Some(expected) {
        return Err("Selected Ethernet interface changed; reconnect required".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ethernet_sockets_have_the_selected_kernel_binding_without_connecting() {
        let mut checked = 0;
        for interface in interfaces().unwrap() {
            if !interface.ready {
                continue;
            }
            for ip in &interface.addresses {
                let address = SocketAddr::new(*ip, 51325);
                let socket = bound_socket(&interface.id, address).unwrap();
                let actual = if ip.is_ipv4() {
                    socket.device_index_v4()
                } else {
                    socket.device_index_v6()
                }
                .unwrap();
                assert_eq!(actual, Some(selected_index(&interface.id, *ip).unwrap()));
                checked += 1;
            }
        }
        eprintln!("Verified {checked} Ethernet socket bindings; no connections opened");
    }

    #[test]
    fn missing_or_unknown_selection_never_uses_default_route() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        for id in ["", "not-an-ethernet-adapter", "test-loopback-not-allowed"] {
            assert!(connect(
                id,
                listener.local_addr().unwrap(),
                Duration::from_millis(100)
            )
            .is_err());
            assert_eq!(
                listener.accept().unwrap_err().kind(),
                std::io::ErrorKind::WouldBlock
            );
        }
    }
}
