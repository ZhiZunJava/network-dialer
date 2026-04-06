use serde::Serialize;

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum RasError {
    #[error("RAS API 错误: {0} (代码: {1})")]
    ApiError(String, u32),

    #[error("没有找到连接条目: {0}")]
    EntryNotFound(String),

    #[error("连接已断开")]
    NotConnected,

    #[error("操作失败: {0}")]
    OperationFailed(String),

    #[error("配置错误: {0}")]
    ConfigError(String),
}

impl Serialize for RasError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Windows API 错误码转可读消息
/// 参考: https://learn.microsoft.com/en-us/windows/win32/rras/routing-and-remote-access-error-codes
pub fn win_error_message(code: u32) -> String {
    match code {
        600 => "操作挂起".to_string(),
        601 => "检测到无效的端口句柄".to_string(),
        602 => "指定的端口已打开".to_string(),
        603 => "缓冲区太小".to_string(),
        604 => "指定了错误的信息".to_string(),
        605 => "无法设置端口信息".to_string(),
        606 => "指定的端口未连接".to_string(),
        607 => "检测到无效事件".to_string(),
        608 => "设备不存在".to_string(),
        609 => "设备类型不存在".to_string(),
        610 => "缓冲区无效".to_string(),
        611 => "路由不可用".to_string(),
        612 => "路由尚未分配".to_string(),
        613 => "指定了无效的压缩".to_string(),
        614 => "缓冲区不足".to_string(),
        615 => "找不到指定的端口".to_string(),
        616 => "异步请求挂起".to_string(),
        617 => "端口或设备正在断开".to_string(),
        618 => "指定的端口未打开".to_string(),
        619 => "指定的端口未连接（连接已终止或尚未建立）".to_string(),
        620 => "无法确定端点".to_string(),
        621 => "无法打开电话簿文件".to_string(),
        622 => "无法加载电话簿文件".to_string(),
        623 => "找不到电话簿条目".to_string(),
        624 => "无法写入电话簿文件".to_string(),
        625 => "在电话簿中发现无效信息".to_string(),
        626 => "无法加载字符串".to_string(),
        627 => "找不到密钥".to_string(),
        628 => "端口已断开（远端关闭了连接）".to_string(),
        629 => "端口被远程计算机断开".to_string(),
        630 => "端口因硬件故障而断开".to_string(),
        631 => "用户已断开连接".to_string(),
        632 => "结构大小不正确".to_string(),
        633 => "端口已在使用中或未配置为拨出".to_string(),
        634 => "无法在远程网络中注册计算机".to_string(),
        635 => "发生未知错误".to_string(),
        636 => "连接到端口的设备不正确".to_string(),
        637 => "无法将字符串转换".to_string(),
        638 => "请求超时".to_string(),
        639 => "没有可用的异步网络".to_string(),
        640 => "NetBIOS 错误".to_string(),
        641 => "服务器无法分配 NetBIOS 资源".to_string(),
        642 => "某个 NetBIOS 名称已在远程网络中注册".to_string(),
        643 => "服务器端的网络适配器故障".to_string(),
        644 => "将无法接收网络弹出消息".to_string(),
        645 => "发生内部身份验证错误".to_string(),
        646 => "不允许在此时段登录".to_string(),
        647 => "帐户已禁用".to_string(),
        648 => "密码已过期".to_string(),
        649 => "帐户没有远程访问权限".to_string(),
        651 => "调制解调器（或其他连接设备）报告了错误".to_string(),
        652 => "来自设备的响应无法识别".to_string(),
        653 => "在设备 .INF 文件中找不到需要的宏".to_string(),
        654 => "设备 .INF 文件中的命令引用了未定义的宏".to_string(),
        655 => "在设备 .INF 文件中找不到 <message> 宏".to_string(),
        656 => "设备 .INF 文件中的 <defaultoff> 宏包含未定义的宏".to_string(),
        657 => "无法打开设备 .INF 文件".to_string(),
        658 => "设备名太长".to_string(),
        659 => "媒体 .INI 文件引用了未知的设备名".to_string(),
        660 => "设备 .INF 文件不包含对该命令的响应".to_string(),
        661 => "设备 .INF 文件缺少命令".to_string(),
        662 => "试图设置未列出的宏".to_string(),
        663 => "媒体 .INI 文件引用了未知的设备类型".to_string(),
        664 => "无法分配内存".to_string(),
        665 => "端口不是为远程访问配置的".to_string(),
        666 => "调制解调器（或其他连接设备）没有工作".to_string(),
        667 => "无法读取媒体 .INI 文件".to_string(),
        668 => "连接已断开（链路已终止）".to_string(),
        669 => "媒体 .INI 文件中的使用参数无效".to_string(),
        670 => "无法从媒体 .INI 文件读取节名".to_string(),
        671 => "无法从媒体 .INI 文件读取设备类型".to_string(),
        672 => "无法从媒体 .INI 文件读取设备名".to_string(),
        673 => "无法从媒体 .INI 文件读取使用".to_string(),
        676 => "线路忙".to_string(),
        677 => "人应答而非调制解调器".to_string(),
        678 => "没有应答".to_string(),
        679 => "无法检测到载波".to_string(),
        680 => "没有拨号音".to_string(),
        691 => "用户名或密码错误".to_string(),
        692 => "端口硬件故障".to_string(),
        695 => "尚未启动状态机".to_string(),
        696 => "状态机已启动".to_string(),
        697 => "响应循环未完成".to_string(),
        699 => "设备响应导致缓冲区溢出".to_string(),
        700 => "设备 .INF 文件中的扩展命令太长".to_string(),
        701 => "设备移至 COM 驱动程序不支持的 BPS 速率".to_string(),
        703 => "需要用户交互的对话框".to_string(),
        704 => "回调号码无效".to_string(),
        705 => "授权状态无效".to_string(),
        707 => "发生 X.25 诊断提示".to_string(),
        708 => "帐户已过期".to_string(),
        709 => "更改域上的密码时出错".to_string(),
        710 => "与调制解调器通信时检测到串行过度运行错误".to_string(),
        711 => "RasMan 初始化失败，请检查事件日志".to_string(),
        712 => "双向端口正在初始化".to_string(),
        713 => "没有可用的活动 ISDN 线路".to_string(),
        714 => "没有足够的 ISDN 通道可用于拨叫".to_string(),
        715 => "电话线路质量太差".to_string(),
        716 => "远程访问 IP 配置不可用".to_string(),
        717 => "静态 IP 地址池中没有可用的 IP 地址".to_string(),
        718 => "等待响应超时（PPP 协商超时）".to_string(),
        719 => "PPP 被远程计算机终止".to_string(),
        720 => "没有可用的 PPP 控制协议".to_string(),
        721 => "远程 PPP 对等体无响应".to_string(),
        722 => "PPP 数据包无效".to_string(),
        723 => "电话号码（包括前缀和后缀）太长".to_string(),
        726 => "IPX 协议无法在多个端口上同时拨出".to_string(),
        728 => "找不到用于拨号的 IP 适配器".to_string(),
        729 => "必须先安装 SLIP 才能使用 SLIP 协议".to_string(),
        731 => "协议配置不正确".to_string(),
        732 => "PPP 协商未收敛".to_string(),
        733 => "PPP 控制协议不可用".to_string(),
        734 => "PPP 链接控制协议已终止".to_string(),
        735 => "请求的地址被服务器拒绝".to_string(),
        736 => "远程计算机终止了控制协议".to_string(),
        737 => "检测到环回".to_string(),
        738 => "服务器未分配地址".to_string(),
        739 => "远程服务器要求的身份验证协议不能使用存储的密码".to_string(),
        740 => "检测到无效的拨号规则".to_string(),
        741 => "本地计算机不支持所需的数据加密类型".to_string(),
        742 => "远程计算机不支持所需的数据加密类型".to_string(),
        743 => "远程服务器要求数据加密".to_string(),
        744 => "无法使用远程服务器要求的 IPX 网络号".to_string(),
        749 => "电话号码无效".to_string(),
        750 => "错误的模块".to_string(),
        751 => "回调号码中包含无效字符".to_string(),
        752 => "处理脚本时遇到语法错误".to_string(),
        753 => "连接无法断开，因为它是由多协议路由器创建的".to_string(),
        754 => "系统找不到多链接包".to_string(),
        755 => "系统无法自动拨号，因为该连接使用了自定义拨号程序".to_string(),
        756 => "此连接已在拨号中（请勿重复拨号）".to_string(),
        757 => "无法自动启动远程访问连接服务".to_string(),
        758 => "Internet 连接共享已在该连接上启用".to_string(),
        760 => "启用路由功能时出现错误".to_string(),
        761 => "为该连接启用 Internet 连接共享时出现错误".to_string(),
        763 => "无法启用 ICS，LAN 连接已配置自动 DHCP".to_string(),
        764 => "未安装智能卡读卡器".to_string(),
        765 => "无法启用 ICS，网络上已有 IP 分配连接".to_string(),
        766 => "找不到任何证书".to_string(),
        767 => "无法启用 ICS，所选网络适配器未启用 IP".to_string(),
        768 => "VPN 服务器不可达".to_string(),
        769 => "VPN 服务器拒绝了连接".to_string(),
        770 => "网络忙".to_string(),
        771 => "远程计算机的网络硬件不兼容".to_string(),
        772 => "目标号码已更改".to_string(),
        773 => "临时故障，请重试".to_string(),
        774 => "呼叫被远端阻止".to_string(),
        775 => "目标启用了\"请勿打扰\"".to_string(),
        776 => "目标计算机的调制解调器故障".to_string(),
        777 => "远端计算机断开连接".to_string(),
        778 => "找不到可以建立此连接的计算机".to_string(),
        779 => "不允许通话".to_string(),
        780 => "VPN 隧道失败".to_string(),
        781 => "找不到有效的加密证书".to_string(),
        783 => "无法启用 ICS/ICF".to_string(),
        784 => "连接正在使用，无法删除".to_string(),
        786 => "L2TP 连接失败，没有可用的安全层".to_string(),
        787 => "L2TP 安全层无法验证远程计算机".to_string(),
        788 => "L2TP 安全层初始协商遇到处理错误".to_string(),
        789 => "L2TP 安全层遇到处理错误".to_string(),
        790 => "L2TP 证书验证失败".to_string(),
        791 => "L2TP 安全策略未找到".to_string(),
        792 => "L2TP 安全协商超时".to_string(),
        793 => "L2TP 安全协商出错".to_string(),
        794 => "未找到属性帧协议".to_string(),
        795 => "PEAP 身份验证失败".to_string(),
        _ => format!("错误代码 {}", code),
    }
}

/// 判断错误码是否为可恢复的临时错误（适合自动重试）
#[allow(dead_code)]
pub fn is_retryable_error(code: u32) -> bool {
    matches!(
        code,
        619 | 628 | 629 | 630 | 668 | 676 | 678 | 680 | 715 | 718
            | 721 | 738 | 768 | 770 | 773 | 777
    )
}

/// 判断错误码是否为不应重试的致命错误（如认证失败、配置错误）
#[allow(dead_code)]
pub fn is_fatal_error(code: u32) -> bool {
    matches!(
        code,
        608 | 623 | 647 | 648 | 649 | 691 | 756
    )
}
