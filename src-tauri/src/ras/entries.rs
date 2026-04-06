use super::error::RasError;
use super::types::RasEntry;
use std::mem;
use windows::Win32::NetworkManagement::Rras::*;

/// 枚举系统中所有 RAS 连接条目
pub fn list_ras_entries() -> Result<Vec<RasEntry>, RasError> {
    unsafe {
        let mut count: u32 = 0;
        let entry_size = mem::size_of::<RASENTRYNAMEW>() as u32;
        let mut buf_size: u32 = entry_size;

        // 第一次调用获取缓冲区大小
        let mut dummy = RASENTRYNAMEW::default();
        dummy.dwSize = entry_size;

        let ret = RasEnumEntriesW(
            None,
            None,
            Some(&mut dummy),
            &mut buf_size,
            &mut count,
        );

        // 成功且 count=0 → 无条目
        if ret == 0 && count == 0 {
            return Ok(Vec::new());
        }

        // 成功且 count=1，dummy 中已有数据
        if ret == 0 && count == 1 {
            let name = String::from_utf16_lossy(&dummy.szEntryName)
                .trim_end_matches('\0')
                .to_string();
            return Ok(vec![RasEntry { name }]);
        }

        // ERROR_BUFFER_TOO_SMALL (603) 表示需要更大的缓冲区
        if ret != 0 && ret != 603 {
            return Err(RasError::ApiError(
                super::error::win_error_message(ret),
                ret,
            ));
        }

        if count == 0 {
            return Ok(Vec::new());
        }

        // 分配缓冲区
        let num_entries = (buf_size as usize) / (entry_size as usize);
        let mut entries = vec![RASENTRYNAMEW::default(); num_entries.max(count as usize)];
        for entry in entries.iter_mut() {
            entry.dwSize = entry_size;
        }

        // 第二次调用填充数据
        let ret = RasEnumEntriesW(
            None,
            None,
            Some(&mut entries[0]),
            &mut buf_size,
            &mut count,
        );

        if ret != 0 {
            return Err(RasError::ApiError(
                super::error::win_error_message(ret),
                ret,
            ));
        }

        let result: Vec<RasEntry> = entries
            .iter()
            .take(count as usize)
            .map(|e| {
                let name = String::from_utf16_lossy(&e.szEntryName)
                    .trim_end_matches('\0')
                    .to_string();
                RasEntry { name }
            })
            .collect();

        Ok(result)
    }
}
