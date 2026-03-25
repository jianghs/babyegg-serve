use std::str::FromStr;

/// 统一语言枚举。
/// 先支持中英文，后续可以继续扩展。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    ZhCn,
    EnUs,
}

impl Locale {
    pub fn as_str(&self) -> &'static str {
        match self {
            Locale::ZhCn => "zh-CN",
            Locale::EnUs => "en-US",
        }
    }
}

impl FromStr for Locale {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "zh" | "zh-cn" | "cn" => Ok(Locale::ZhCn),
            "en" | "en-us" => Ok(Locale::EnUs),
            _ => Err(()),
        }
    }
}
