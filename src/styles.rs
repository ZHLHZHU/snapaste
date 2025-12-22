use gpui::*;

/// 颜色主题
pub struct Theme {
    /// 背景色
    pub background: Hsla,
    /// 表面色（卡片/弹窗背景）
    pub surface: Hsla,
    /// 主色
    pub primary: Hsla,
    /// 文本主色
    pub text_primary: Hsla,
    /// 文本次要色
    pub text_secondary: Hsla,
    /// 边框色
    pub border: Hsla,
    /// 选中背景
    pub selected: Hsla,
    /// 悬停背景
    pub hover: Hsla,
    /// 成功色
    pub success: Hsla,
    /// 错误色
    pub error: Hsla,
}

impl Theme {
    /// 暗色主题
    pub fn dark() -> Self {
        Self {
            background: hsla(220.0 / 360.0, 0.15, 0.08, 1.0),
            surface: hsla(220.0 / 360.0, 0.15, 0.12, 1.0),
            primary: hsla(210.0 / 360.0, 0.80, 0.55, 1.0),
            text_primary: hsla(0.0, 0.0, 0.95, 1.0),
            text_secondary: hsla(0.0, 0.0, 0.60, 1.0),
            border: hsla(220.0 / 360.0, 0.15, 0.20, 1.0),
            selected: hsla(210.0 / 360.0, 0.60, 0.25, 1.0),
            hover: hsla(220.0 / 360.0, 0.15, 0.18, 1.0),
            success: hsla(140.0 / 360.0, 0.60, 0.45, 1.0),
            error: hsla(0.0 / 360.0, 0.70, 0.55, 1.0),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

/// 尺寸常量
pub struct Sizes;

impl Sizes {
    /// 窗口宽度
    pub const WINDOW_WIDTH: f32 = 420.0;
    /// 窗口高度
    pub const WINDOW_HEIGHT: f32 = 480.0;
    /// 内边距
    pub const PADDING: f32 = 8.0;
    /// 小内边距
    pub const PADDING_SM: f32 = 4.0;
    /// 圆角
    pub const RADIUS: f32 = 10.0;
    /// 小圆角
    pub const RADIUS_SM: f32 = 4.0;
    /// 列表项高度（紧凑单行）
    pub const LIST_ITEM_HEIGHT: f32 = 28.0;
    /// 搜索框高度
    pub const SEARCH_HEIGHT: f32 = 36.0;
    /// 字体大小
    pub const FONT_SIZE: f32 = 13.0;
    /// 小字体
    pub const FONT_SIZE_SM: f32 = 11.0;
    /// 菜单项高度
    pub const MENU_ITEM_HEIGHT: f32 = 24.0;
}
