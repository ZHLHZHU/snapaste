use muda::{Menu, MenuItem, PredefinedMenuItem};
use tray_icon::{
    TrayIcon, TrayIconBuilder,
    menu::MenuEvent,
    Icon,
};
use std::sync::mpsc::{self, Receiver, Sender};

/// 托盘事件
#[derive(Debug, Clone)]
pub enum TrayEvent {
    /// 显示主窗口
    ShowWindow,
    /// 显示设置窗口
    ShowSettings,
    /// 退出应用
    Quit,
}

pub const SHOW_ID: &str = "show";
pub const SETTINGS_ID: &str = "settings";
pub const QUIT_ID: &str = "quit";

/// 托盘管理器
pub struct TrayManager {
    _tray_icon: TrayIcon,
    event_sender: Sender<TrayEvent>,
    event_receiver: Option<Receiver<TrayEvent>>,
    show_item_id: muda::MenuId,
    settings_item_id: muda::MenuId,
    quit_item_id: muda::MenuId,
}

impl TrayManager {
    /// 创建托盘管理器
    pub fn new() -> anyhow::Result<Self> {
        let (event_sender, event_receiver) = mpsc::channel();
        
        // 创建菜单
        let menu = Menu::new();
        
        let show_item = MenuItem::with_id(
            muda::MenuId::new(SHOW_ID),
            "显示 Snapaste",
            true,
            None
        );
        let show_item_id = show_item.id().clone();
        
        let settings_item = MenuItem::with_id(
            muda::MenuId::new(SETTINGS_ID),
            "设置...",
            true,
            None
        );
        let settings_item_id = settings_item.id().clone();
        
        let quit_item = MenuItem::with_id(
            muda::MenuId::new(QUIT_ID),
            "退出",
            true,
            None
        );
        let quit_item_id = quit_item.id().clone();
        
        menu.append(&show_item)?;
        menu.append(&settings_item)?;
        menu.append(&PredefinedMenuItem::separator())?;
        menu.append(&quit_item)?;
        
        // 创建托盘图标 - 使用一个简单的图标
        let icon = create_default_icon()?;
        
        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Snapaste - 粘贴板历史")
            .with_icon(icon)
            .build()?;
        
        Ok(Self {
            _tray_icon: tray_icon,
            event_sender,
            event_receiver: Some(event_receiver),
            show_item_id,
            settings_item_id,
            quit_item_id,
        })
    }
    
    /// 获取事件接收器
    pub fn take_receiver(&mut self) -> Option<Receiver<TrayEvent>> {
        self.event_receiver.take()
    }
    
    /// 处理菜单事件（需要在主线程调用）
    pub fn process_menu_events(&self) {
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == self.show_item_id {
                let _ = self.event_sender.send(TrayEvent::ShowWindow);
            } else if event.id == self.settings_item_id {
                let _ = self.event_sender.send(TrayEvent::ShowSettings);
            } else if event.id == self.quit_item_id {
                let _ = self.event_sender.send(TrayEvent::Quit);
            }
        }
    }
}

/// 创建默认的托盘图标（一个简单的剪贴板图标）
fn create_default_icon() -> anyhow::Result<Icon> {
    // 创建一个 22x22 的简单图标
    let size = 22u32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];
    
    // 绘制一个简单的剪贴板形状
    for y in 0..size {
        for x in 0..size {
            let idx = ((y * size + x) * 4) as usize;
            
            // 剪贴板主体 (圆角矩形)
            let in_body = x >= 3 && x < size - 3 && y >= 5 && y < size - 2;
            
            // 剪贴板夹子
            let in_clip = x >= 7 && x < size - 7 && y >= 2 && y < 7;
            
            // 剪贴板内容线条
            let in_line1 = x >= 6 && x < size - 6 && y >= 10 && y < 12;
            let in_line2 = x >= 6 && x < size - 6 && y >= 14 && y < 16;
            
            if in_clip {
                // 夹子 - 深灰色
                rgba[idx] = 80;     // R
                rgba[idx + 1] = 80; // G
                rgba[idx + 2] = 80; // B
                rgba[idx + 3] = 255; // A
            } else if in_line1 || in_line2 {
                // 线条 - 白色
                rgba[idx] = 255;
                rgba[idx + 1] = 255;
                rgba[idx + 2] = 255;
                rgba[idx + 3] = 255;
            } else if in_body {
                // 主体 - 蓝色
                rgba[idx] = 66;      // R
                rgba[idx + 1] = 133; // G
                rgba[idx + 2] = 244; // B
                rgba[idx + 3] = 255; // A
            } else {
                // 透明
                rgba[idx] = 0;
                rgba[idx + 1] = 0;
                rgba[idx + 2] = 0;
                rgba[idx + 3] = 0;
            }
        }
    }
    
    Icon::from_rgba(rgba, size, size)
        .map_err(|e| anyhow::anyhow!("Failed to create icon: {}", e))
}

impl Default for TrayManager {
    fn default() -> Self {
        Self::new().expect("Failed to create tray manager")
    }
}
