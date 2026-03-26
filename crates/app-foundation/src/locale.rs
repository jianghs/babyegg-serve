use std::str::FromStr;

/// 统一语言枚举。
/// 先支持中英文，后续可以继续扩展。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    /// 简体中文。
    ZhCn,
    /// 美式英文。
    EnUs,
}

impl Locale {
    /// 返回标准语言标签字符串。
    pub fn as_str(&self) -> &'static str {
        match self {
            Locale::ZhCn => "zh-CN",
            Locale::EnUs => "en-US",
        }
    }
}

impl FromStr for Locale {
    type Err = ();

    /// 从环境变量或请求中的字符串解析语言区域。
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "zh" | "zh-cn" | "cn" => Ok(Locale::ZhCn),
            "en" | "en-us" => Ok(Locale::EnUs),
            _ => Err(()),
        }
    }
}
