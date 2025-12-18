use chrono::{DateTime, Local};

/// 粘贴板历史项
#[derive(Debug, Clone)]
pub struct ClipboardItem {
    pub id: usize,
    pub content: String,
    pub timestamp: DateTime<Local>,
}

impl ClipboardItem {
    /// 创建新的历史项
    pub fn new(id: usize, content: String) -> Self {
        Self {
            id,
            content,
            timestamp: Local::now(),
        }
    }

    /// 获取内容预览(截断长文本)
    pub fn preview(&self, max_len: usize) -> String {
        if self.content.len() <= max_len {
            self.content.clone()
        } else {
            format!("{}...", &self.content[..max_len])
        }
    }

    /// 格式化时间戳
    pub fn formatted_time(&self) -> String {
        self.timestamp.format("%H:%M:%S").to_string()
    }
}

/// 历史记录管理器
pub struct ClipboardHistory {
    items: Vec<ClipboardItem>,
    max_items: usize,
    next_id: usize,
}

impl ClipboardHistory {
    /// 创建新的历史记录管理器
    pub fn new(max_items: usize) -> Self {
        Self {
            items: Vec::new(),
            max_items,
            next_id: 0,
        }
    }

    /// 添加新项(如果不是重复的)
    pub fn add(&mut self, content: String) -> bool {
        // 去重:检查是否与最近一项相同
        if let Some(last) = self.items.first() {
            if last.content == content {
                return false;
            }
        }

        // 创建新项
        let item = ClipboardItem::new(self.next_id, content);
        self.next_id += 1;

        // 插入到开头
        self.items.insert(0, item);

        // 限制数量
        if self.items.len() > self.max_items {
            self.items.truncate(self.max_items);
        }

        true
    }

    /// 获取所有历史项
    pub fn items(&self) -> &[ClipboardItem] {
        &self.items
    }

    /// 根据查询过滤历史项
    pub fn search(&self, query: &str) -> Vec<ClipboardItem> {
        if query.is_empty() {
            return self.items.clone();
        }

        let query_lower = query.to_lowercase();
        self.items
            .iter()
            .filter(|item| item.content.to_lowercase().contains(&query_lower))
            .cloned()
            .collect()
    }

    /// 根据 ID 获取项
    pub fn get_by_id(&self, id: usize) -> Option<&ClipboardItem> {
        self.items.iter().find(|item| item.id == id)
    }

    /// 清空历史
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// 获取历史项数量
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}
