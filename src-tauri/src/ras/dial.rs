use super::error::{win_error_message, RasError};
use std::mem;
use windows::Win32::Foundation::BOOL;
use windows::Win32::NetworkManagement::Rras::*;

/// 安全地将 Rust 字符串拷贝到 UTF-16 定长数组（含 null 终止符）
/// 如果字符串过长则返回错误
fn copy_str_to_wide(src: &str, dst: &mut [u16], field_name: &str) -> Result<(), RasError> {
    let wide: Vec<u16> = src.encode_utf16().chain(std::iter::once(0)).collect();
    if wide.len() > dst.len() {
        return Err(RasError::OperationFailed(format!(
            "{} 过长（最大 {} 字符，实际 {} 字符）",
            field_name,
            dst.len() - 1,
            src.chars().count()
        )));
    }
    dst[..wide.len()].copy_from_slice(&wide);
    Ok(())
}

/// 从系统中加载已保存的拨号参数（包括凭据）
/// 当用户未手动输入用户名/密码时，使用此函数获取系统存储的凭据
fn load_saved_dial_params(entry_name: &str) -> Result<(RASDIALPARAMSW, bool), RasError> {
    unsafe {
        let mut dial_params = RASDIALPARAMSW::default();
        dial_params.dwSize = mem::size_of::<RASDIALPARAMSW>() as u32;

        // 设置条目名称
        copy_str_to_wide(entry_name, &mut dial_params.szEntryName, "连接名称")?;

        let mut has_password = BOOL::default();

        let ret = RasGetEntryDialParamsW(
            None, // 使用默认电话簿
            &mut dial_params,
            &mut has_password,
        );

        if ret != 0 {
            return Err(RasError::ApiError(win_error_message(ret), ret));
        }

        Ok((dial_params, has_password.as_bool()))
    }
}

/// 发起 RAS 拨号连接
/// 返回连接句柄
/// 如果 username/password 为空，将自动加载系统已保存的凭据
pub fn ras_dial(entry_name: &str, username: &str, password: &str) -> Result<HRASCONN, RasError> {
    unsafe {
        let dial_params;

        if username.is_empty() && password.is_empty() {
            // 用户未提供凭据，从系统加载已保存的拨号参数
            // RasGetEntryDialParamsW 会填充条目名称、用户名、密码等所有已保存字段
            match load_saved_dial_params(entry_name) {
                Ok((params, _has_password)) => {
                    dial_params = params;
                }
                Err(_) => {
                    // 加载失败，回退到仅使用条目名称
                    let mut params = RASDIALPARAMSW::default();
                    params.dwSize = mem::size_of::<RASDIALPARAMSW>() as u32;
                    copy_str_to_wide(entry_name, &mut params.szEntryName, "连接名称")?;
                    dial_params = params;
                }
            }
        } else {
            // 用户手动提供了凭据
            let mut params = RASDIALPARAMSW::default();
            params.dwSize = mem::size_of::<RASDIALPARAMSW>() as u32;

            copy_str_to_wide(entry_name, &mut params.szEntryName, "连接名称")?;

            if !username.is_empty() {
                copy_str_to_wide(username, &mut params.szUserName, "用户名")?;
            }

            if !password.is_empty() {
                copy_str_to_wide(password, &mut params.szPassword, "密码")?;
            }

            dial_params = params;
        }

        let mut h_conn = HRASCONN::default();

        let ret = RasDialW(
            None,
            None,
            &dial_params,
            0,
            None,
            &mut h_conn,
        );

        if ret != 0 {
            return Err(RasError::ApiError(win_error_message(ret), ret));
        }

        // 验证连接是否真正建立
        // RasDialW 返回 0 只表示拨号请求已提交，需要轮询确认状态
        match wait_for_connection(h_conn) {
            Ok(()) => Ok(h_conn),
            Err(e) => {
                // 连接验证失败，主动挂断残留连接
                let _ = RasHangUpW(h_conn);
                wait_hangup_complete(h_conn);
                Err(e)
            }
        }
    }
}

/// 等待 RasDial 连接完成（轮询 RasGetConnectStatus）
/// 最多等待 60 秒
fn wait_for_connection(h_conn: HRASCONN) -> Result<(), RasError> {
    unsafe {
        let mut status = RASCONNSTATUSW::default();
        status.dwSize = mem::size_of::<RASCONNSTATUSW>() as u32;

        let max_wait = 60; // 最多等待 60 秒
        for _ in 0..max_wait * 4 {
            let ret = RasGetConnectStatusW(h_conn, &mut status);
            if ret != 0 {
                return Err(RasError::ApiError(win_error_message(ret), ret));
            }

            let conn_state = status.rasconnstate;
            if conn_state == RASCS_Connected {
                return Ok(());
            } else if conn_state == RASCS_Disconnected {
                let err_code = status.dwError;
                if err_code != 0 {
                    return Err(RasError::ApiError(win_error_message(err_code), err_code));
                }
                return Err(RasError::NotConnected);
            } else {
                // 仍在连接中，检查是否有子错误
                if status.dwError != 0 {
                    let err_code = status.dwError;
                    return Err(RasError::ApiError(win_error_message(err_code), err_code));
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(250));
        }

        Err(RasError::OperationFailed("连接超时（60秒）".to_string()))
    }
}

/// 挂断 RAS 连接
pub fn ras_hangup(h_conn: HRASCONN) -> Result<(), RasError> {
    unsafe {
        let ret = RasHangUpW(h_conn);
        if ret != 0 {
            return Err(RasError::ApiError(win_error_message(ret), ret));
        }
        // 等待连接完全关闭（轮询而非硬等待）
        wait_hangup_complete(h_conn);
        Ok(())
    }
}

/// 等待挂断完成：轮询 RasGetConnectStatus 直到返回错误（表示句柄已无效=连接已关闭）
/// 最多等待 10 秒
fn wait_hangup_complete(h_conn: HRASCONN) {
    unsafe {
        let mut status = RASCONNSTATUSW::default();
        status.dwSize = mem::size_of::<RASCONNSTATUSW>() as u32;

        for _ in 0..40 {
            // 40 * 250ms = 10s
            let ret = RasGetConnectStatusW(h_conn, &mut status);
            // 当 RasGetConnectStatus 返回非零，说明句柄已失效 → 连接已完全关闭
            if ret != 0 {
                return;
            }
            // 如果已报告 Disconnected 状态
            if status.rasconnstate == RASCS_Disconnected {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
    }
}
