use super::error::{win_error_message, RasError};
use super::types::RasConnection;
use std::mem;
use windows::Win32::NetworkManagement::Rras::*;

/// 获取所有活动的 RAS 连接
pub fn list_active_connections() -> Result<Vec<RasConnection>, RasError> {
    unsafe {
        let mut count: u32 = 0;
        let conn_size = mem::size_of::<RASCONNW>() as u32;
        let mut buf_size: u32 = conn_size;

        let mut dummy = RASCONNW::default();
        dummy.dwSize = conn_size;

        let ret = RasEnumConnectionsW(Some(&mut dummy), &mut buf_size, &mut count);

        // 没有活动连接
        if ret == 0 && count == 0 {
            return Ok(Vec::new());
        }

        // 成功且 count=1, dummy 中已有数据
        if ret == 0 && count == 1 {
            let name = String::from_utf16_lossy(&dummy.szEntryName)
                .trim_end_matches('\0')
                .to_string();
            return Ok(vec![RasConnection {
                name,
                handle: dummy.hrasconn.0 as usize,
            }]);
        }

        if ret != 0 && ret != 603 {
            if count == 0 {
                return Ok(Vec::new());
            }
            return Err(RasError::ApiError(win_error_message(ret), ret));
        }

        let num_conns = (buf_size as usize) / (conn_size as usize);
        let mut conns = vec![RASCONNW::default(); num_conns.max(count as usize)];
        for conn in conns.iter_mut() {
            conn.dwSize = conn_size;
        }

        let ret = RasEnumConnectionsW(Some(&mut conns[0]), &mut buf_size, &mut count);

        if ret != 0 {
            return Err(RasError::ApiError(win_error_message(ret), ret));
        }

        let result: Vec<RasConnection> = conns
            .iter()
            .take(count as usize)
            .map(|c| {
                let name = String::from_utf16_lossy(&c.szEntryName)
                    .trim_end_matches('\0')
                    .to_string();
                RasConnection {
                    name,
                    handle: c.hrasconn.0 as usize,
                }
            })
            .collect();

        Ok(result)
    }
}

/// 检查指定名称的连接是否在线
pub fn is_entry_connected(entry_name: &str) -> Result<(bool, Option<HRASCONN>), RasError> {
    let connections = list_active_connections()?;
    for conn in &connections {
        if conn.name == entry_name {
            let h = HRASCONN(conn.handle as *mut _);
            return Ok((true, Some(h)));
        }
    }
    Ok((false, None))
}

/// 检查是否有任何 RAS 连接是活跃的，如果有返回第一个的名称和句柄
pub fn any_active_connection() -> Result<Option<(String, HRASCONN)>, RasError> {
    let connections = list_active_connections()?;
    if let Some(conn) = connections.first() {
        let h = HRASCONN(conn.handle as *mut _);
        return Ok(Some((conn.name.clone(), h)));
    }
    Ok(None)
}
