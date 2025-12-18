use arboard::Clipboard;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use std::time::Duration;

/// 粘贴板变化事件
#[derive(Debug, Clone)]
pub struct ClipboardEvent {
    pub content: String,
}

/// 粘贴板监控器
pub struct ClipboardMonitor {
    sender: Sender<ClipboardEvent>,
    receiver: Option<Receiver<ClipboardEvent>>,
}

impl ClipboardMonitor {
    /// 创建新的粘贴板监控器
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        Self { sender, receiver: Some(receiver) }
    }

    /// 启动监控线程
    pub fn start(&self) {
        let sender = self.sender.clone();

        thread::spawn(move || {
            let mut clipboard = match Clipboard::new() {
                Ok(cb) => cb,
                Err(e) => {
                    eprintln!("无法初始化粘贴板: {}", e);
                    return;
                }
            };

            let mut last_content = String::new();

            loop {
                // 每 500ms 检查一次粘贴板
                thread::sleep(Duration::from_millis(500));

                // 尝试读取粘贴板内容
                match clipboard.get_text() {
                    Ok(content) => {
                        // 如果内容发生变化且不为空
                        if !content.is_empty() && content != last_content {
                            last_content = content.clone();

                            // 发送事件
                            if let Err(e) = sender.send(ClipboardEvent { content }) {
                                eprintln!("发送粘贴板事件失败: {}", e);
                                break;
                            }
                        }
                    }
                    Err(_) => {
                        // 粘贴板可能为空或包含非文本内容,忽略
                    }
                }
            }
        });
    }

    /// 获取接收器的引用(用于 CLI 模式)
    pub fn receiver(&self) -> Option<&Receiver<ClipboardEvent>> {
        self.receiver.as_ref()
    }

    /// 取出接收器所有权(用于 GUI 模式,只能调用一次)
    pub fn take_receiver(&mut self) -> Option<Receiver<ClipboardEvent>> {
        self.receiver.take()
    }

    /// 尝试接收粘贴板事件(非阻塞)
    pub fn try_recv(&self) -> Option<ClipboardEvent> {
        self.receiver.as_ref()?.try_recv().ok()
    }
}

impl Default for ClipboardMonitor {
    fn default() -> Self {
        Self::new()
    }
}
