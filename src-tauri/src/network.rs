use std::net::{TcpStream, SocketAddr};
use std::time::Duration;
use windows::Win32::NetworkManagement::IpHelper::{
    FreeMibTable, GetIfTable2,
};
use windows::Win32::NetworkManagement::Ndis::{
    IF_OPER_STATUS, IfOperStatusUp,
};

/// Ethernet CSMA/CD adapter type (value 6)
const IF_TYPE_ETHERNET_CSMACD: u32 = 6;

/// 检查是否有至少一个以太网适配器物理链路 UP（网线已插入）
pub fn has_physical_link() -> bool {
    unsafe {
        let mut table = std::ptr::null_mut();
        let ret = GetIfTable2(&mut table);
        if ret.is_err() {
            return false;
        }

        let table_ref = &*table;
        let count = table_ref.NumEntries as usize;

        let entries = std::slice::from_raw_parts(
            table_ref.Table.as_ptr(),
            count,
        );

        let mut found = false;
        for entry in entries {
            // 只关注以太网适配器 (type 6)
            if entry.Type == IF_TYPE_ETHERNET_CSMACD
                && entry.OperStatus == IF_OPER_STATUS(IfOperStatusUp.0)
            {
                found = true;
                break;
            }
        }

        FreeMibTable(table as *const _);
        found
    }
}

/// 检查系统是否已有可用的互联网连接
/// 通过尝试 TCP 连接公共 DNS 服务器来判断
pub fn has_internet_connectivity() -> bool {
    let targets: &[SocketAddr] = &[
        "223.5.5.5:53".parse().unwrap(),   // 阿里 DNS
        "114.114.114.114:53".parse().unwrap(), // 114 DNS
    ];

    let timeout = Duration::from_secs(2);

    for addr in targets {
        if TcpStream::connect_timeout(addr, timeout).is_ok() {
            return true;
        }
    }

    false
}
