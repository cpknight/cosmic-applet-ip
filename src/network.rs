// SPDX-License-Identifier: GPL-3.0-only
//
// Read local network interfaces + their IPs without shelling out. We use
// `if-addrs` (a thin wrapper around `getifaddrs(3)`) for addresses/up state
// and parse `/proc/net/route` to discover the default-route interface.

use std::fs;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Snapshot of a single network interface, aggregated across address families.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Interface {
    pub name: String,
    pub ipv4: Vec<Ipv4Addr>,
    pub ipv6: Vec<Ipv6Addr>,
    pub is_loopback: bool,
    /// True iff the interface currently has at least one non-link-local address.
    pub has_address: bool,
}

impl Interface {
    /// Preferred, human-readable IPv4 address if any, else a short IPv6.
    pub fn primary_display(&self) -> Option<IpAddr> {
        if let Some(v4) = self.ipv4.iter().find(|a| !a.is_unspecified()) {
            return Some(IpAddr::V4(*v4));
        }
        // Prefer non-link-local IPv6 for display.
        if let Some(v6) = self
            .ipv6
            .iter()
            .find(|a| !is_link_local_v6(a) && !a.is_unspecified())
        {
            return Some(IpAddr::V6(*v6));
        }
        None
    }

    /// True if the interface is up AND has a usable (non-link-local) address.
    pub fn is_connected(&self) -> bool {
        self.has_address
    }
}

fn is_link_local_v6(a: &Ipv6Addr) -> bool {
    let seg = a.segments();
    (seg[0] & 0xffc0) == 0xfe80
}

fn is_link_local_v4(a: &Ipv4Addr) -> bool {
    let o = a.octets();
    o[0] == 169 && o[1] == 254
}

/// Enumerate all interfaces on the host, sorted by "friendliness":
///   1. default-route interface (if any) first
///   2. other non-loopback, connected interfaces (alphabetical)
///   3. non-loopback, unconnected interfaces
///   4. loopback last
pub fn list_interfaces() -> Vec<Interface> {
    let raw = match if_addrs::get_if_addrs() {
        Ok(v) => v,
        Err(err) => {
            tracing::warn!(%err, "failed to enumerate network interfaces");
            return Vec::new();
        }
    };

    let mut by_name: std::collections::BTreeMap<String, Interface> = Default::default();

    for iface in raw {
        let entry = by_name.entry(iface.name.clone()).or_insert(Interface {
            name: iface.name.clone(),
            ipv4: Vec::new(),
            ipv6: Vec::new(),
            is_loopback: iface.is_loopback(),
            has_address: false,
        });

        match iface.ip() {
            IpAddr::V4(a) => {
                if !entry.ipv4.contains(&a) {
                    entry.ipv4.push(a);
                }
                if !a.is_unspecified() && !is_link_local_v4(&a) && !entry.is_loopback {
                    entry.has_address = true;
                }
            }
            IpAddr::V6(a) => {
                if !entry.ipv6.contains(&a) {
                    entry.ipv6.push(a);
                }
                if !a.is_unspecified() && !is_link_local_v6(&a) && !entry.is_loopback {
                    entry.has_address = true;
                }
            }
        }
    }

    let default = default_interface_name();

    let mut list: Vec<Interface> = by_name.into_values().collect();
    list.sort_by_key(|iface| {
        let default_rank = if Some(&iface.name) == default.as_ref() {
            0
        } else {
            1
        };
        let loop_rank = if iface.is_loopback { 2 } else { 0 };
        let connected_rank = if iface.has_address { 0 } else { 1 };
        (default_rank, loop_rank, connected_rank, iface.name.clone())
    });

    list
}

/// Read the kernel routing table and return the interface name owning the
/// default (0.0.0.0) route, if any.
pub fn default_interface_name() -> Option<String> {
    let contents = fs::read_to_string("/proc/net/route").ok()?;
    let mut best: Option<(String, u32)> = None;
    for line in contents.lines().skip(1) {
        let mut it = line.split_whitespace();
        let iface = it.next()?.to_string();
        let dest = it.next()?;
        let _gateway = it.next()?;
        let _flags = it.next()?;
        let _refcnt = it.next()?;
        let _use = it.next()?;
        let metric = it.next().and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);

        if dest == "00000000" {
            match &best {
                Some((_, m)) if metric >= *m => {}
                _ => best = Some((iface, metric)),
            }
        }
    }
    best.map(|(name, _)| name)
}
