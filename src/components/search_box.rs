use gpui::*;

/// 搜索框组件
pub struct SearchBox {
    /// 当前搜索查询
    pub query: SharedString,
    /// 是否获得焦点
    pub focused: bool,
}

impl SearchBox {
    pub fn new() -> Self {
        Self {
            query: SharedString::default(),
            focused: true,
        }
    }

    pub fn set_query(&mut self, query: impl Into<SharedString>) {
        self.query = query.into();
    }

    pub fn clear(&mut self) {
        self.query = SharedString::default();
    }
}

impl Default for SearchBox {
    fn default() -> Self {
        Self::new()
    }
}
