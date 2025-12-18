use crate::history::{ClipboardHistory, ClipboardItem};

/// 应用程序状态
pub struct AppState {
    /// 历史记录管理器
    pub history: ClipboardHistory,
    /// 当前搜索查询
    pub search_query: String,
    /// 当前选中的索引
    pub selected_index: usize,
    /// 过滤后的历史项
    filtered_items: Vec<ClipboardItem>,
}

impl AppState {
    /// 创建新的应用状态
    pub fn new() -> Self {
        Self {
            history: ClipboardHistory::new(100),
            search_query: String::new(),
            selected_index: 0,
            filtered_items: Vec::new(),
        }
    }

    /// 添加新的粘贴板内容
    pub fn add_clipboard_content(&mut self, content: String) {
        if self.history.add(content) {
            self.update_filtered_items();
        }
    }

    /// 更新搜索查询
    pub fn set_search_query(&mut self, query: String) {
        self.search_query = query;
        self.selected_index = 0;
        self.update_filtered_items();
    }

    /// 更新过滤后的项
    fn update_filtered_items(&mut self) {
        self.filtered_items = self.history.search(&self.search_query);
    }

    /// 获取过滤后的历史项
    pub fn filtered_items(&self) -> &[ClipboardItem] {
        &self.filtered_items
    }

    /// 向上移动选择
    pub fn move_selection_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// 向下移动选择
    pub fn move_selection_down(&mut self) {
        let max_index = self.filtered_items.len().saturating_sub(1);
        if self.selected_index < max_index {
            self.selected_index += 1;
        }
    }

    /// 获取当前选中的项
    pub fn selected_item(&self) -> Option<&ClipboardItem> {
        self.filtered_items.get(self.selected_index)
    }

    /// 重置选择
    pub fn reset_selection(&mut self) {
        self.selected_index = 0;
    }

    /// 初始化过滤项
    pub fn init_filtered_items(&mut self) {
        self.update_filtered_items();
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
