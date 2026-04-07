; NSIS Hooks - 将快捷方式显示名改为中文
; productName 保持英文（NetworkDialer）以确保 CI 签名/文件名兼容
; 通过钩子在安装后将快捷方式重命名为中文

!macro NSIS_HOOK_POSTINSTALL
  ; 升级兼容：清理旧版中文注册表项（旧版 productName 为中文时注册的）
  DeleteRegKey SHCTX "Software\Microsoft\Windows\CurrentVersion\Uninstall\网络拨号连接管理器"
  DeleteRegKey SHCTX "Software\networkline\网络拨号连接管理器"

  ; 先删除可能存在的旧版中文快捷方式（防止重复）
  Delete "$DESKTOP\网络拨号连接管理器.lnk"
  Delete "$SMPROGRAMS\网络拨号连接管理器.lnk"

  ; 将新建的英文快捷方式重命名为中文
  IfFileExists "$DESKTOP\${PRODUCTNAME}.lnk" 0 +2
    Rename "$DESKTOP\${PRODUCTNAME}.lnk" "$DESKTOP\网络拨号连接管理器.lnk"

  IfFileExists "$SMPROGRAMS\${PRODUCTNAME}.lnk" 0 +2
    Rename "$SMPROGRAMS\${PRODUCTNAME}.lnk" "$SMPROGRAMS\网络拨号连接管理器.lnk"

  ; 更新注册表显示名为中文（控制面板卸载列表）
  WriteRegStr SHCTX "${UNINSTKEY}" "DisplayName" "网络拨号连接管理器"
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ; 卸载时删除中文名快捷方式
  Delete "$DESKTOP\网络拨号连接管理器.lnk"
  Delete "$SMPROGRAMS\网络拨号连接管理器.lnk"
!macroend
