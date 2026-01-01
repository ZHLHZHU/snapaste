use crate::clipboard_monitor::{ClipboardEvent, ClipboardMonitor};
use crate::components::history_list::HistoryList;
use crate::components::list_item::ListItemProps;

use crate::components::search_box::SearchBox;
use crate::history::ClipboardHistory;
use crate::styles::{Sizes, Theme};
use arboard::Clipboard;
use gpui::*;
use std::sync::mpsc::Receiver;
use std::time::Duration;
use global_hotkey::{GlobalHotKeyManager, GlobalHotKeyEvent, hotkey::{HotKey, Modifiers, Code}};

// 应用程序动作
actions!(
    snapaste,
    [
        MoveUp,
        MoveDown,
        Confirm,
        Escape,
        ClearSearch,
        DeleteItem,
        Backspace,
        Select1,
        Select2,
        Select3,
        Select4,
        Select5,
        Select6,
        Select7,
        Select8,
        Select9,
        ClearHistory,
        QuitApp,
        CloseWindow,
        ConfirmClear,
        CancelClear,
    ]
);

/// Snapaste 主视图
pub struct Snapaste {
    /// 主题
    theme: Theme,
    /// 搜索框
    search_box: SearchBox,
    /// 历史列表
    history_list: HistoryList,
    /// 粘贴板历史
    history: ClipboardHistory,
    /// 粘贴板事件接收器
    clipboard_receiver: Receiver<ClipboardEvent>,
    /// 状态消息
    status_message: Option<SharedString>,
    /// 搜索查询字符串
    search_query: String,
    /// 焦点句柄
    focus_handle: FocusHandle,
    /// 光标可见状态（用于闪烁动画）
    cursor_visible: bool,
    /// 是否显示清除确认对话框
    show_clear_confirm: bool,
}

impl Snapaste {
    pub fn new(clipboard_receiver: Receiver<ClipboardEvent>, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        
        // 启动后台定时器任务，定期轮询粘贴板事件
        // 确保即使窗口没有焦点也能更新数据
        cx.spawn(|this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            let this = this.clone();
            async move {
                loop {
                    // 每 500ms 轮询一次
                    cx.background_executor().timer(Duration::from_millis(500)).await;
                    let result = this.update(&mut cx, |this, cx| {
                        this.poll_clipboard_events(cx);
                        
                        // TODO: 实现窗口失焦自动隐藏
                        // 目前 cx.is_window_focused() 不存在
                    });
                    // 如果更新失败（例如视图已销毁），退出循环
                    if result.is_err() {
                        break;
                    }
                }
            }
        }).detach();
        
        // 启动光标闪烁定时器
        cx.spawn(|this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            let this = this.clone();
            async move {
                loop {
                    cx.background_executor().timer(Duration::from_millis(530)).await;
                    let result = this.update(&mut cx, |this, cx| {
                        this.cursor_visible = !this.cursor_visible;
                        cx.notify();
                    });
                    if result.is_err() {
                        break;
                    }
                }
            }
        }).detach();
        
        Self {
            theme: Theme::dark(),
            search_box: SearchBox::new(),
            history_list: HistoryList::new(),
            history: ClipboardHistory::new(100),
            clipboard_receiver,
            status_message: None,
            search_query: String::new(),
            focus_handle,
            cursor_visible: true,
            show_clear_confirm: false,
        }
    }

    /// 处理粘贴板事件
    fn poll_clipboard_events(&mut self, cx: &mut Context<Self>) {
        // 处理粘贴板事件
        while let Ok(event) = self.clipboard_receiver.try_recv() {
            if self.history.add(event.content.clone()) {
                self.update_list_items();
                self.status_message = Some(format!("📋 新增: {}", Self::truncate(&event.content, 30)).into());
                cx.notify();
            }
        }
    }

    /// 更新列表项

    /// 更新列表项
    fn update_list_items(&mut self) {
        let filtered = self.history.search(&self.search_query);
        
        let selected_idx = self.history_list.selected_index;
        
        // 修正：items 使用 ListItemProps 构建
        let items: Vec<ListItemProps> = filtered
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                ListItemProps::new(
                    Self::truncate(&item.content, 80),
                    item.formatted_time(),
                    idx,
                ).selected(idx == selected_idx)
            })
            .collect();
        
        self.history_list.set_items(items);
    }
    
    /// 处理键盘输入
    fn handle_key_input(&mut self, key: &str, cx: &mut Context<Self>) {
        // 放宽输入限制，支持所有非控制字符（包括中文、Emoji、符号等）
        for c in key.chars() {
            if !c.is_control() {
                self.search_query.push(c);
            }
        }
        self.history_list.selected_index = 0;
        self.update_list_items();
        cx.notify();
    }
    
    /// 删除搜索查询的最后一个字符
    fn backspace(&mut self, cx: &mut Context<Self>) {
        if !self.search_query.is_empty() {
            self.search_query.pop();
            self.history_list.selected_index = 0;
            self.update_list_items();
            cx.notify();
        }
    }

    /// 处理搜索输入
    fn on_search_input(&mut self, query: String, cx: &mut Context<Self>) {
        self.search_box.set_query(query);
        self.history_list.selected_index = 0;
        self.update_list_items();
        cx.notify();
    }

    /// 移动选择向上
    fn move_up(&mut self, cx: &mut Context<Self>) {
        self.history_list.select_previous();
        self.update_list_items();
        cx.notify();
    }

    /// 移动选择向下
    fn move_down(&mut self, cx: &mut Context<Self>) {
        self.history_list.select_next();
        self.update_list_items();
        cx.notify();
    }

    /// 确认选择（复制到粘贴板并自动粘贴）
    fn confirm_selection(&mut self, cx: &mut Context<Self>) {
        let query = self.search_box.query.to_string();
        let filtered = self.history.search(&query);
        
        if let Some(item) = filtered.get(self.history_list.selected_index) {
            match Clipboard::new() {
                Ok(mut clipboard) => {
                    if clipboard.set_text(&item.content).is_ok() {
                        // 隐藏窗口
                        cx.hide();
                        
                        // 模拟 Cmd+V 粘贴
                        // 需要在后台线程执行，避免阻塞 UI，同时给予窗口隐藏的时间
                        cx.spawn(|_, cx: &mut AsyncApp| {
                            let cx = cx.clone();
                            async move {
                                 // 等待窗口隐藏和焦点切换 - 增加到 300ms 以确保稳定性
                                 cx.background_executor().timer(Duration::from_millis(300)).await;
                                 
                                 #[cfg(target_os = "macos")]
                                 {
                                     use std::process::Command;
                                     let _ = Command::new("osascript")
                                         .arg("-e")
                                         .arg("tell application \"System Events\" to keystroke \"v\" using command down")
                                         .output();
                                 }
                            }
                        }).detach();
                        
                        self.status_message = Some(format!("✓ 已复制: {}", Self::truncate(&item.content, 30)).into());
                    } else {
                        self.status_message = Some("✗ 复制失败".into());
                    }
                }
                Err(_) => {
                    self.status_message = Some("✗ 无法访问粘贴板".into());
                }
            }
            cx.notify();
        }
    }

    /// 清空搜索
    fn clear_search(&mut self, cx: &mut Context<Self>) {
        self.search_query.clear();
        self.search_box.clear();
        self.history_list.selected_index = 0;
        self.update_list_items();
        cx.notify();
    }
    
    /// 选择指定索引的项并复制（自动粘贴）
    fn select_item(&mut self, index: usize, cx: &mut Context<Self>) {
        let filtered = self.history.search(&self.search_query);
        if let Some(item) = filtered.get(index) {
            match Clipboard::new() {
                Ok(mut clipboard) => {
                    if clipboard.set_text(&item.content).is_ok() {
                        // 隐藏窗口
                        cx.hide();
                        
                        // 模拟 Cmd+V 粘贴
                        cx.spawn(|_, cx: &mut AsyncApp| {
                            let cx = cx.clone();
                            async move {
                                 cx.background_executor().timer(Duration::from_millis(300)).await;
                                 #[cfg(target_os = "macos")]
                                 {
                                     use std::process::Command;
                                     let _ = Command::new("osascript")
                                         .arg("-e")
                                         .arg("tell application \"System Events\" to keystroke \"v\" using command down")
                                         .output();
                                 }
                            }
                        }).detach();
                        
                        self.status_message = Some(format!("✓ 已复制: {}", Self::truncate(&item.content, 30)).into());
                    }
                }
                Err(_) => {}
            }
            cx.notify();
        }
    }
    
    /// 清空历史记录
    fn clear_history(&mut self, cx: &mut Context<Self>) {
        self.history.clear();
        self.update_list_items();
        self.status_message = Some("✓ 历史记录已清空".into());
        self.show_clear_confirm = false;
        cx.notify();
    }
    
    /// 显示清除确认对话框
    fn show_clear_confirm(&mut self, cx: &mut Context<Self>) {
        self.show_clear_confirm = true;
        cx.notify();
    }
    
    /// 取消清除
    fn cancel_clear(&mut self, cx: &mut Context<Self>) {
        self.show_clear_confirm = false;
        cx.notify();
    }

    /// 截断字符串
    fn truncate(s: &str, max_len: usize) -> String {
        let s = s.replace('\n', " ").replace('\r', "");
        if s.chars().count() <= max_len {
            s
        } else {
            format!("{}...", s.chars().take(max_len).collect::<String>())
        }
    }

    /// 渲染搜索框
    fn render_search_box(&self, _cx: &mut Context<Self>) -> impl IntoElement {
        let theme = &self.theme;
        let query = &self.search_query;
        // 移除不用的 display_text 变量，逻辑已移至 relative div 内部
        // let display_text: SharedString = ...
        
        div()
            .w_full()
            .h(px(Sizes::SEARCH_HEIGHT))
            .px(px(Sizes::PADDING))
            .py(px(Sizes::PADDING_SM))
            .bg(theme.surface)
            .border_b_1()
            .border_color(theme.border)
            .child(
                div()
                    .w_full()
                    .h_full()
                    .px(px(Sizes::PADDING))
                    .bg(theme.background)
                    .rounded(px(Sizes::RADIUS_SM))
                    .border_1()
                    .border_color(if query.is_empty() { theme.border } else { theme.primary })
                    .flex()
                    .items_center()
                    .child(
                        div()
                            .text_size(px(Sizes::FONT_SIZE))
                            .text_color(theme.text_secondary)
                            .mr(px(8.0))
                            .child("🔍")
                    )
                    .cursor_text() // 鼠标放上去显示文本编辑指针
                    .child(
                        div()
                            .flex_1()
                            .text_size(px(Sizes::FONT_SIZE))
                            .text_color(if query.is_empty() { theme.text_secondary } else { theme.text_primary })
                            .relative()
                            .child(if query.is_empty() { SharedString::from("输入搜索...") } else { SharedString::from(query.clone()) })
                            // 闪烁光标独立渲染，避免移位
                            .child(
                                div()
                                    .absolute()
                                    .top(px(0.0))
                                    .left(if query.is_empty() { px(0.0) } else { px(query.chars().count() as f32 * 9.0) }) // 估算偏移
                                    .child(if self.cursor_visible { "▏" } else { "" })
                            )
                    )
            )
    }

    /// 渲染列表项（紧凑单行样式）
    fn render_list_item(&self, item: &ListItemProps, index: usize) -> Div {
        let theme = &self.theme;
        let bg_color = if item.selected { theme.selected } else { theme.surface };
        
        // 快捷键提示 (⌘1 - ⌘9)
        let shortcut: SharedString = if index < 9 {
            format!("⌘{}", index + 1).into()
        } else {
            "".into()
        };
        
        div()
            .w_full()
            .h(px(Sizes::LIST_ITEM_HEIGHT))
            .px(px(Sizes::PADDING))
            .bg(bg_color)
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_between()
            .child(
                div()
                    .flex_1()
                    .text_size(px(Sizes::FONT_SIZE))
                    .text_color(theme.text_primary)
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .child(item.content.clone())
            )
            .child(
                div()
                    .ml(px(8.0))
                    .text_size(px(Sizes::FONT_SIZE_SM))
                    .text_color(theme.text_secondary)
                    .child(shortcut)
            )
    }

    /// 渲染历史列表
    fn render_history_list(&self, _cx: &mut Context<Self>) -> Div {
        let theme = &self.theme;
        let items = self.history_list.items.clone();
        
        if items.is_empty() {
            div()
                .flex_1()
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .text_size(px(Sizes::FONT_SIZE))
                        .text_color(theme.text_secondary)
                        .child(if self.search_box.query.is_empty() {
                            "暂无粘贴板历史"
                        } else {
                            "未找到匹配项"
                        })
                )
        } else {
            div()
                .flex_1()
                .overflow_hidden()
                .children(items.iter().enumerate().map(|(idx, item)| self.render_list_item(item, idx)))
        }
    }

    /// 渲染状态栏
    fn render_status_bar(&self, _cx: &mut Context<Self>) -> impl IntoElement {
        let theme = &self.theme;
        let item_count = self.history_list.item_count();
        
        div()
            .w_full()
            .h(px(32.0))
            .px(px(Sizes::PADDING))
            .bg(theme.surface)
            .border_t_1()
            .border_color(theme.border)
            .flex()
            .items_center()
            .justify_between()
            .child(
                div()
                    .text_size(px(Sizes::FONT_SIZE_SM))
                    .text_color(theme.text_secondary)
                    .child(format!("{} 条记录", item_count))
            )
            .child(
                div()
                    .text_size(px(Sizes::FONT_SIZE_SM))
                    .text_color(theme.text_secondary)
                    .child(
                        self.status_message.clone().unwrap_or_else(|| "↑↓ 导航 | Enter 复制 | Esc 清除".into())
                    )
            )
    }
    
    /// 渲染底部菜单
    fn render_bottom_menu(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = &self.theme;
        
        div()
            .w_full()
            .border_t_1()
            .border_color(theme.border)
            .bg(theme.surface)
            .child(
                div()
                    .id("clear-button")
                    .w_full()
                    .h(px(Sizes::MENU_ITEM_HEIGHT))
                    .px(px(Sizes::PADDING))
                    .flex()
                    .items_center()
                    .justify_between()
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.hover))
                    .on_click(cx.listener(|this, _event, _window, cx| {
                        this.show_clear_confirm(cx);
                    }))
                    .child(
                        div()
                            .text_size(px(Sizes::FONT_SIZE))
                            .text_color(theme.text_primary)
                            .child("清除")
                    )
                    .child(
                        div()
                            .text_size(px(Sizes::FONT_SIZE_SM))
                            .text_color(theme.text_secondary)
                            .child("⌥⌘⌫")
                    )
            )
            .child(
                div()
                    .id("close-button")
                    .w_full()
                    .h(px(Sizes::MENU_ITEM_HEIGHT))
                    .px(px(Sizes::PADDING))
                    .flex()
                    .items_center()
                    .justify_between()
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.hover))
                    .on_click(cx.listener(|_this, _event, _window, cx| {
                        cx.hide();
                    }))
                    .child(
                        div()
                            .text_size(px(Sizes::FONT_SIZE))
                            .text_color(theme.text_primary)
                            .child("退出")
                    )
                    .child(
                        div()
                            .text_size(px(Sizes::FONT_SIZE_SM))
                            .text_color(theme.text_secondary)
                            .child("⌘Q")
                    )
            )
    }
    
    /// 渲染确认对话框
    fn render_confirm_dialog(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = &self.theme;
        
        div()
            .absolute()
            .inset_0()
            .bg(hsla(0.0, 0.0, 0.0, 0.5))
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .w(px(280.0))
                    .bg(theme.surface)
                    .rounded(px(Sizes::RADIUS))
                    .border_1()
                    .border_color(theme.border)
                    .p(px(Sizes::PADDING * 2.0))
                    .flex()
                    .flex_col()
                    .gap(px(Sizes::PADDING))
                    .child(
                        div()
                            .text_size(px(Sizes::FONT_SIZE))
                            .text_color(theme.text_primary)
                            .child("确定要清空所有历史记录吗？")
                    )
                    .child(
                        div()
                            .text_size(px(Sizes::FONT_SIZE_SM))
                            .text_color(theme.text_secondary)
                            .child("此操作不可撤销")
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(Sizes::PADDING))
                            .justify_end()
                            .child(
                                div()
                                    .id("cancel-clear")
                                    .px(px(12.0))
                                    .py(px(6.0))
                                    .bg(theme.background)
                                    .rounded(px(Sizes::RADIUS_SM))
                                    .border_1()
                                    .border_color(theme.border)
                                    .cursor_pointer()
                                    .hover(|s| s.bg(theme.hover))
                                    .on_click(cx.listener(|this, _event, _window, cx| {
                                        this.cancel_clear(cx);
                                    }))
                                    .child(
                                        div()
                                            .text_size(px(Sizes::FONT_SIZE_SM))
                                            .text_color(theme.text_primary)
                                            .child("取消")
                                    )
                            )
                            .child(
                                div()
                                    .id("confirm-clear")
                                    .px(px(12.0))
                                    .py(px(6.0))
                                    .bg(theme.error)
                                    .rounded(px(Sizes::RADIUS_SM))
                                    .cursor_pointer()
                                    .hover(|s| s.opacity(0.9))
                                    .on_click(cx.listener(|this, _event, _window, cx| {
                                        this.clear_history(cx);
                                    }))
                                    .child(
                                        div()
                                            .text_size(px(Sizes::FONT_SIZE_SM))
                                            .text_color(theme.text_primary)
                                            .child("确认清除")
                                    )
                            )
                    )
            )
    }
}

impl Render for Snapaste {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 轮询粘贴板事件
        self.poll_clipboard_events(cx);
        
        let theme = &self.theme;
        let show_dialog = self.show_clear_confirm;
        
        let mut main_view = div()
            .size_full()
            .bg(theme.background)
            .rounded(px(Sizes::RADIUS))
            .flex()
            .flex_col()
            .track_focus(&self.focus_handle)
            .child(self.render_search_box(cx))
            .child(self.render_history_list(cx))
            .child(self.render_bottom_menu(cx))
            .on_action(cx.listener(|this, _: &MoveUp, _window, cx| {
                this.move_up(cx);
            }))
            .on_action(cx.listener(|this, _: &MoveDown, _window, cx| {
                this.move_down(cx);
            }))
            .on_action(cx.listener(|this, _: &Confirm, _window, cx| {
                this.confirm_selection(cx);
            }))
            .on_action(cx.listener(|this, _: &Escape, _window, cx| {
                if this.show_clear_confirm {
                    this.cancel_clear(cx);
                } else {
                    this.clear_search(cx);
                }
            }))
            .on_action(cx.listener(|this, _: &Backspace, _window, cx| {
                this.backspace(cx);
            }))
            .on_action(cx.listener(|this, _: &Select1, _window, cx| { this.select_item(0, cx); }))
            .on_action(cx.listener(|this, _: &Select2, _window, cx| { this.select_item(1, cx); }))
            .on_action(cx.listener(|this, _: &Select3, _window, cx| { this.select_item(2, cx); }))
            .on_action(cx.listener(|this, _: &Select4, _window, cx| { this.select_item(3, cx); }))
            .on_action(cx.listener(|this, _: &Select5, _window, cx| { this.select_item(4, cx); }))
            .on_action(cx.listener(|this, _: &Select6, _window, cx| { this.select_item(5, cx); }))
            .on_action(cx.listener(|this, _: &Select7, _window, cx| { this.select_item(6, cx); }))
            .on_action(cx.listener(|this, _: &Select8, _window, cx| { this.select_item(7, cx); }))
            .on_action(cx.listener(|this, _: &Select9, _window, cx| { this.select_item(8, cx); }))
            .on_action(cx.listener(|this, _: &ClearHistory, _window, cx| { this.clear_history(cx); }))
            .on_action(cx.listener(|_, _: &QuitApp, _window, cx| { cx.quit(); }))
            .on_action(cx.listener(|_, _: &CloseWindow, _window, cx| { cx.hide(); }))
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                let modifiers = event.keystroke.modifiers;
                let is_plain = !modifiers.control && !modifiers.alt && !modifiers.platform && !modifiers.function;
                
                if is_plain {
                    if let Some(char_str) = &event.keystroke.key_char {
                        this.handle_key_input(char_str, cx);
                    }
                }
            }));
        
        // 如果显示确认对话框，添加遮罩层
        if show_dialog {
            main_view = main_view.child(self.render_confirm_dialog(cx));
        }
        
        main_view
    }
}

/// 全局应用控制器
struct AppController {
    tray: Option<crate::tray::TrayManager>,
    hotkey_manager: Option<GlobalHotKeyManager>,
    window_handle: Option<WindowHandle<Snapaste>>,
}

impl AppController {
    fn new(hotkey_manager: Option<GlobalHotKeyManager>, _cx: &mut Context<Self>) -> Self {
        // 创建托盘
        let tray = crate::tray::TrayManager::new().ok();
        if tray.is_some() {
            println!("✓ 系统托盘已创建");
        }
        
        Self {
            tray,
            hotkey_manager,
            window_handle: None,
        }
    }
    
    fn open_window(&mut self, cx: &mut Context<Self>) {
        // 如果窗口已存在且有效，激活它
        if let Some(handle) = &self.window_handle {
             // WindowHandle::update 闭包接受: (&mut V, &mut WindowContext)
             if handle.update(cx, |_, _window, _cx| {
                 // cx.activate_window();
             }).is_ok() {
                 cx.activate(true);
                 return;
             }
        }
        
        // 创建新的粘贴板监控器
        let mut monitor = ClipboardMonitor::new();
        monitor.start();
        let receiver = if let Some(rx) = monitor.take_receiver() {
            rx
        } else {
            eprintln!("错误: 无法获取剪贴板接收器");
            return;
        };

        // 弹出菜单样式的窗口选项
        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(Sizes::WINDOW_WIDTH), px(Sizes::WINDOW_HEIGHT)),
                cx,
            ))),
            titlebar: None, // 无标题栏
            window_background: WindowBackgroundAppearance::Blurred,
            ..Default::default()
        };
        
        // 我们需要传递 &mut AppContext
        let view: anyhow::Result<WindowHandle<Snapaste>> = cx.open_window(window_options, |_, cx| {
             cx.new(|cx| {
                 Snapaste::new(receiver, cx)
             })
        });
        
        if let Ok(view) = view {
            self.window_handle = Some(view);
            cx.activate(true);
        } else {
            if let Err(e) = view {
                eprintln!("无法打开窗口: {}", e);
            }
        }
    }

    fn handle_menu(&mut self, event: muda::MenuEvent, cx: &mut Context<Self>) {
        let id = event.id.0.as_str();
        use crate::tray::{SHOW_ID, QUIT_ID};
        
        if id == SHOW_ID {
            self.open_window(cx);
        } else if id == QUIT_ID {
            cx.quit();
        }
    }
}

/// 运行 GUI 应用
/// 运行 GUI 应用
pub fn run_gui() -> anyhow::Result<()> {
    // 创建 GPUI 应用
    Application::new().run(move |cx: &mut App| {
        // 设置键绑定
        cx.bind_keys([
            KeyBinding::new("up", MoveUp, None),
            KeyBinding::new("down", MoveDown, None),
            KeyBinding::new("enter", Confirm, None),
            KeyBinding::new("escape", Escape, None),
            KeyBinding::new("backspace", Backspace, None),
            // ⌘1-⌘9 快捷键
            KeyBinding::new("cmd-1", Select1, None),
            KeyBinding::new("cmd-2", Select2, None),
            KeyBinding::new("cmd-3", Select3, None),
            KeyBinding::new("cmd-4", Select4, None),
            KeyBinding::new("cmd-5", Select5, None),
            KeyBinding::new("cmd-6", Select6, None),
            KeyBinding::new("cmd-7", Select7, None),
            KeyBinding::new("cmd-8", Select8, None),
            KeyBinding::new("cmd-9", Select9, None),
            // 其他快捷键
            KeyBinding::new("alt-cmd-backspace", ClearHistory, None),
            KeyBinding::new("cmd-q", QuitApp, None),
        ]);
        
        // 设置 Dock 菜单
        cx.set_dock_menu(vec![
            gpui::MenuItem::action("显示 Snapaste", MoveUp),
        ]);

        // 注册全局热键 Ctrl+/
        let hotkey_manager = GlobalHotKeyManager::new().ok();
        if let Some(ref manager) = hotkey_manager {
            let hotkey = HotKey::new(Some(Modifiers::CONTROL), Code::Slash);
            if let Err(e) = manager.register(hotkey) {
                eprintln!("无法注册全局热键 Ctrl+/: {:?}", e);
            } else {
                println!("✓ 全局热键 Ctrl+/ 已注册");
            }
        }
        
        // 创建全局控制器，并传入 hotkey_manager 以保持其生命周期
        let app_controller = cx.new(|cx| AppController::new(hotkey_manager, cx));
        
        // 首次打开窗口
        app_controller.update(cx, |controller, cx| {
             controller.open_window(cx);
        });
        
        // 启动后台任务监听托盘菜单和全局热键（轮询）
        cx.spawn(|cx: &mut AsyncApp| {
            let cx = cx.clone();
            async move {
                let menu_receiver = muda::MenuEvent::receiver();
                let hotkey_receiver = GlobalHotKeyEvent::receiver();
                loop {
                    // 使用 try_recv 避免阻塞 - 菜单事件
                    while let Ok(event) = menu_receiver.try_recv() {
                        let _ = cx.update(|cx| {
                            app_controller.update(cx, |controller, cx| {
                                controller.handle_menu(event, cx);
                            });
                        });
                    }

                    // 使用 try_recv 避免阻塞 - 全局热键事件
                    while let Ok(event) = hotkey_receiver.try_recv() {
                        println!("⚡ 收到全局热键事件: {:?}", event);
                        let _ = cx.update(|cx| {
                            app_controller.update(cx, |controller, cx| {
                                controller.open_window(cx);
                            });
                        });
                    }

                    // 等待 200ms
                    cx.background_executor().timer(std::time::Duration::from_millis(200)).await;
                }
            }
        }).detach();
    });
    
    Ok(())
}
