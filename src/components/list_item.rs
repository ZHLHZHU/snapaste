use gpui::*;

/// 列表项组件属性
#[derive(Clone)]
pub struct ListItemProps {
    /// 内容预览
    pub content: SharedString,
    /// 时间戳
    pub timestamp: SharedString,
    /// 是否选中
    pub selected: bool,
    /// 索引
    pub index: usize,
}

impl ListItemProps {
    pub fn new(content: impl Into<SharedString>, timestamp: impl Into<SharedString>, index: usize) -> Self {
        Self {
            content: content.into(),
            timestamp: timestamp.into(),
            selected: false,
            index,
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}
