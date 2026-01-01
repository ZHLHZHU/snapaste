use crate::components::list_item::ListItemProps;
// use gpui::*; // Removed unused import

/// 历史列表组件
pub struct HistoryList {
    /// 列表项
    pub items: Vec<ListItemProps>,
    /// 当前选中索引
    pub selected_index: usize,
    /// 滚动偏移
    pub scroll_offset: f32,
}

impl HistoryList {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected_index: 0,
            scroll_offset: 0.0,
        }
    }

    pub fn set_items(&mut self, items: Vec<ListItemProps>) {
        self.items = items;
        // 重置选中索引如果超出范围
        if self.selected_index >= self.items.len() {
            self.selected_index = self.items.len().saturating_sub(1);
        }
    }

    pub fn select_next(&mut self) {
        if self.selected_index < self.items.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    pub fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn selected_item(&self) -> Option<&ListItemProps> {
        self.items.get(self.selected_index)
    }

    pub fn item_count(&self) -> usize {
        self.items.len()
    }
}

impl Default for HistoryList {
    fn default() -> Self {
        Self::new()
    }
}
