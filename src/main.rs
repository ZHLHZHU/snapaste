mod app_state;
mod clipboard_monitor;
mod config;
mod components;
mod history;
mod styles;
mod tray;
mod ui;

use app_state::AppState;
use arboard::Clipboard;
use clap::Parser;
use clipboard_monitor::ClipboardMonitor;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

/// Snapaste - 粘贴板历史管理工具
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// 使用命令行界面模式
    #[arg(short, long)]
    cli: bool,
}

fn main() {
    let args = Args::parse();

    // 单例检查
    let _lock_file = match one_instance_check() {
        Some(f) => f,
        None => {
            eprintln!("错误: Snapaste 已经在运行中");
            return;
        }
    };

    if args.cli {
        run_cli();
    } else {
        run_gui();
    }
}

/// 检查是否只有一个实例在运行
fn one_instance_check() -> Option<std::fs::File> {
    use directories::ProjectDirs;
    use fs2::FileExt;
    use std::fs::File;

    let proj_dirs = ProjectDirs::from("com", "snapaste", "snapaste")?;
    let data_dir = proj_dirs.data_local_dir();
    std::fs::create_dir_all(data_dir).ok()?;

    let lock_file_path = data_dir.join("snapaste.lock");
    let file = File::create(lock_file_path).ok()?;

    match file.try_lock_exclusive() {
        Ok(_) => Some(file),
        Err(_) => None,
    }
}

/// 运行 GUI 模式
fn run_gui() {
    println!("=== Snapaste - 粘贴板历史管理工具 ===");
    println!("正在启动 GUI 模式...\n");

    #[cfg(target_os = "linux")]
    {
        if let Err(e) = gtk::init() {
            eprintln!("无法初始化 GTK: {}", e);
            return;
        }
    }

    if let Err(e) = ui::run_gui() {
        eprintln!("GUI 启动失败: {}", e);
        println!("尝试使用 --cli 参数启动命令行模式");
    }
}

/// 运行 CLI 模式
fn run_cli() {
    println!("=== Snapaste - 粘贴板历史管理工具 ===");
    println!("正在启动 CLI 模式...\n");

    // 创建应用状态
    let app_state = Arc::new(Mutex::new(AppState::new()));

    // 创建粘贴板监控器
    let monitor = ClipboardMonitor::new();
    monitor.start();

    println!("✓ 粘贴板监控已启动");
    println!("✓ 正在监听粘贴板变化...\n");
    println!("命令:");
    println!("  list  - 显示历史记录");
    println!("  search <关键词> - 搜索历史记录");
    println!("  copy <编号> - 复制指定编号的历史记录");
    println!("  clear - 清空历史记录");
    println!("  quit  - 退出程序\n");

    // 初始化过滤项
    app_state.lock().unwrap().init_filtered_items();

    // 主循环
    loop {
        // 检查粘贴板事件
        while let Some(event) = monitor.try_recv() {
            let mut state = app_state.lock().unwrap();
            state.add_clipboard_content(event.content.clone());
            println!("📋 新增粘贴板内容: {}", truncate(&event.content, 50));
        }

        // 读取用户输入
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            continue;
        }

        let input = input.trim();
        let parts: Vec<&str> = input.split_whitespace().collect();

        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "list" => {
                let state = app_state.lock().unwrap();
                let items = state.filtered_items();

                if items.is_empty() {
                    println!("暂无历史记录");
                } else {
                    println!("\n历史记录 (共 {} 条):", items.len());
                    println!("{:-<80}", "");
                    for (idx, item) in items.iter().enumerate() {
                        println!(
                            "[{}] {} | {}",
                            idx,
                            item.formatted_time(),
                            truncate(&item.content, 60)
                        );
                    }
                    println!("{:-<80}\n", "");
                }
            }
            "search" => {
                if parts.len() < 2 {
                    println!("用法: search <关键词>");
                    continue;
                }

                let query = parts[1..].join(" ");
                let mut state = app_state.lock().unwrap();
                state.set_search_query(query.clone());

                let items = state.filtered_items();
                println!("\n搜索结果 '{}' (共 {} 条):", query, items.len());
                println!("{:-<80}", "");
                for (idx, item) in items.iter().enumerate() {
                    println!(
                        "[{}] {} | {}",
                        idx,
                        item.formatted_time(),
                        truncate(&item.content, 60)
                    );
                }
                println!("{:-<80}\n", "");
            }
            "copy" => {
                if parts.len() < 2 {
                    println!("用法: copy <编号>");
                    continue;
                }

                if let Ok(idx) = parts[1].parse::<usize>() {
                    let state = app_state.lock().unwrap();
                    let items = state.filtered_items();

                    if let Some(item) = items.get(idx) {
                        match Clipboard::new() {
                            Ok(mut clipboard) => {
                                if clipboard.set_text(&item.content).is_ok() {
                                    println!("✓ 已复制到粘贴板: {}", truncate(&item.content, 50));
                                } else {
                                    println!("✗ 复制失败");
                                }
                            }
                            Err(e) => println!("✗ 无法访问粘贴板: {}", e),
                        }
                    } else {
                        println!("✗ 无效的编号");
                    }
                } else {
                    println!("✗ 编号必须是数字");
                }
            }
            "clear" => {
                let mut state = app_state.lock().unwrap();
                state.history.clear();
                state.init_filtered_items();
                println!("✓ 历史记录已清空");
            }
            "quit" | "exit" => {
                println!("再见!");
                break;
            }
            _ => {
                println!("未知命令: {}", parts[0]);
            }
        }
    }
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
