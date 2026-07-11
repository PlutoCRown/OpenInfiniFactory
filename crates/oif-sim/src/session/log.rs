use std::collections::VecDeque;

const MAX_ENTRIES: usize = 2000;

/// 单条模拟调试日志
#[derive(Clone, Debug)]
struct LogEntry {
    turn: u64,
    message: String,
}

/// 模拟调试日志缓冲
#[derive(Default)]
pub struct SimulationDebugLog {
    pub enabled: bool,
    entries: VecDeque<LogEntry>,
}

impl SimulationDebugLog {
    /// 开关日志采集
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// 清空缓冲
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// 写入一条日志（未启用时忽略）
    pub fn log(&mut self, turn: u64, message: impl Into<String>) {
        if !self.enabled {
            return;
        }
        let message = message.into();
        eprintln!("[sim turn={turn}] {message}");
        self.entries.push_back(LogEntry { turn, message });
        while self.entries.len() > MAX_ENTRIES {
            self.entries.pop_front();
        }
    }

    /// 最近条目的 JSON 串
    pub fn recent_json(&self, limit: usize) -> String {
        let limit = limit.clamp(1, MAX_ENTRIES);
        let entries: Vec<_> = self
            .entries
            .iter()
            .rev()
            .take(limit)
            .map(|entry| {
                format!(
                    r#"{{"turn":{},"message":{}}}"#,
                    entry.turn,
                    serde_json::to_string(&entry.message).unwrap_or_else(|_| "\"\"".into())
                )
            })
            .collect();
        format!(r#"{{"entries":[{}]}}"#, entries.join(","))
    }
}
