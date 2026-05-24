use chrono::DateTime;

/// 将多种类型统一转为时间戳 (Unix epoch 秒)
pub fn 转化为时间戳(ts: &str) -> Option<i64> {
    // 尝试解析为整数时间戳
    if let Ok(v) = ts.parse::<i64>() {
        return Some(v);
    }
    // 尝试解析为浮点时间戳
    if let Ok(v) = ts.parse::<f64>() {
        return Some(v as i64);
    }
    // 尝试 ISO 格式日期字符串
    if let Ok(dt) = DateTime::parse_from_rfc3339(ts) {
        return Some(dt.timestamp());
    }
    // 尝试 "YYYY-MM-DD HH:MM:SS" 格式
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(ts, "%Y-%m-%d %H:%M:%S") {
        return Some(dt.and_utc().timestamp());
    }
    None
}

/// 将多种类型转为时间戳数字 (i64)
pub fn 转化为时间戳_数字(ts: &str) -> Option<i64> {
    转化为时间戳(ts)
}
