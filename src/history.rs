use chrono::{DateTime, Local};
use crate::config::StorageConfig;

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ContentType {
    Text,
    Image,
    File,
}

/// 粘贴板历史项
#[derive(Debug, Clone)]
pub struct ClipboardItem {
    pub id: usize,
    pub content: String,
    pub timestamp: DateTime<Local>,
    pub first_copied_at: DateTime<Local>,
    pub copy_count: usize,
    pub content_type: ContentType,
}

impl ClipboardItem {
    /// 创建新的历史项
    pub fn new(id: usize, content: String, content_type: ContentType) -> Self {
        let now = Local::now();
        Self {
            id,
            content,
            timestamp: now,
            first_copied_at: now,
            copy_count: 1,
            content_type,
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
    /// 添加新项(如果不是重复的)
    pub fn add(&mut self, content: String, config: &StorageConfig) -> bool {
        // 检查内容类型是否被允许
        // 目前暂只处理文本，后续可扩展
        if !config.store_text {
            return false;
        }

        // 检查是否已存在
        if let Some(idx) = self.items.iter().position(|item| item.content == content) {
            // 已存在，更新时间戳和复制次数，并移动到开头
            let mut item = self.items.remove(idx);
            item.timestamp = Local::now();
            item.copy_count += 1;
            self.items.insert(0, item);
            return true;
        }

        // 创建新项
        let item = ClipboardItem::new(self.next_id, content, ContentType::Text);
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

    /// 根据查询过滤并排序历史项
    pub fn search(&self, query: &str, sort_order: &crate::config::SortOrder) -> Vec<ClipboardItem> {
        let mut results = if query.is_empty() {
            self.items.clone()
        } else {
            let query_lower = query.to_lowercase();
            self.items
                .iter()
                .filter(|item| item.content.to_lowercase().contains(&query_lower))
                .cloned()
                .collect()
        };

        match sort_order {
            crate::config::SortOrder::LastCopied => {
                results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            }
            crate::config::SortOrder::FirstCopied => {
                results.sort_by(|a, b| b.first_copied_at.cmp(&a.first_copied_at));
            }
            crate::config::SortOrder::CopyCount => {
                results.sort_by(|a, b| b.copy_count.cmp(&a.copy_count));
            }
        }

        results
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

    /// 设置最大历史项数量并截断
    pub fn set_max_items(&mut self, max_items: usize) {
        self.max_items = max_items;
        if self.items.len() > self.max_items {
            self.items.truncate(self.max_items);
        }
    }
}
