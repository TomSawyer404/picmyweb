use eframe::egui;
use std::env;
use std::fs;
use std::sync::{Arc, Mutex};

use crate::{screenshot, url_handler};

const COPYRIGHT: &str = "© 2025 佛子岭日夜加班有限责任公司";

#[derive(Default)]
pub struct PicMyWebApp {
    pub status: Arc<Mutex<String>>,
    pub logs: Arc<Mutex<Vec<String>>>,
    pub is_processing: bool,
    pub file_path: String,
}

impl PicMyWebApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            file_path: String::from("websites.txt"),
            ..Default::default()
        }
    }
}

impl eframe::App for PicMyWebApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading(format!("网站截图工具"));
                ui.label(COPYRIGHT);
                ui.separator();
            });

            // 文件选择区域
            ui.horizontal(|ui| {
                ui.label("📄 网址文件:");
                ui.text_edit_singleline(&mut self.file_path);

                if ui.button("📁 浏览...").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("文本文件", &["txt"])
                        .set_directory(std::env::current_dir().unwrap())
                        .pick_file()
                    {
                        self.file_path = path.display().to_string();
                    }
                }
            });

            // 显示文件状态
            if !self.file_path.is_empty() {
                let file_exists = std::path::Path::new(&self.file_path).exists();
                let (icon, text) = if file_exists {
                    ("✅", "文件已找到")
                } else {
                    ("❌", "文件不存在")
                };
                ui.horizontal(|ui| {
                    ui.label(format!("{} {}", icon, text));
                });
            }

            ui.separator();

            let is_processing = self.is_processing;

            // 开始截图按钮
            ui.horizontal(|ui| {
                let button = ui.add_enabled(
                    !is_processing && !self.file_path.is_empty(),
                    egui::Button::new("🚀 开始截图").min_size(egui::Vec2::new(120.0, 40.0)),
                );

                if button.clicked() {
                    self.is_processing = true;
                    let status = Arc::clone(&self.status);
                    let logs = Arc::clone(&self.logs);
                    let file_path = self.file_path.clone();

                    *status.lock().unwrap() = "正在初始化...".to_string();
                    logs.lock().unwrap().clear();
                    logs.lock().unwrap().push("🚀 开始处理...".to_string());

                    let ctx = ctx.clone();

                    tokio::spawn(async move {
                        let result = process_urls(logs, file_path).await;
                        *status.lock().unwrap() = result;
                        ctx.request_repaint();
                    });
                }

                if self.is_processing {
                    ui.spinner();
                    ui.label("处理中...");
                }
            });

            ui.separator();

            // 状态显示
            let status = self.status.lock().unwrap();
            if !status.is_empty() {
                ui.horizontal(|ui| {
                    ui.label("📊 状态:");
                    ui.label(&*status);
                });
            }

            // 日志显示区域
            egui::ScrollArea::vertical()
                .max_height(250.0)
                .show(ui, |ui| {
                    let logs = self.logs.lock().unwrap();
                    if logs.is_empty() {
                        ui.label("暂无日志...");
                    } else {
                        for log in logs.iter() {
                            ui.label(log);
                        }
                    }
                });

            if !status.is_empty() && !self.is_processing {
                self.is_processing = false;
            }
        });
    }
}

async fn process_urls(logs: Arc<Mutex<Vec<String>>>, file_path: String) -> String {
    // 创建截图目录
    let _ = fs::create_dir_all("screenshots");

    // 检查文件是否存在
    if !std::path::Path::new(&file_path).exists() {
        logs.lock()
            .unwrap()
            .push(format!("❌ 错误：文件不存在 - {}", file_path));
        return format!("文件不存在: {}", file_path);
    }

    match url_handler::get_websites_from_file(&file_path) {
        Ok(websites) => {
            if websites.is_empty() {
                logs.lock()
                    .unwrap()
                    .push("⚠️ 警告：文件中没有找到有效的网址".to_string());
                return "没有找到要截图的网站".to_string();
            }

            let mut success_count = 0;
            let total_count = websites.len();

            logs.lock()
                .unwrap()
                .push(format!("📋 共找到 {} 个待处理网址", total_count));

            for (index, url) in websites.iter().enumerate() {
                let current = index + 1;
                logs.lock().unwrap().push(format!(
                    "[{}/{}] 🔄 正在处理: {}",
                    current, total_count, url
                ));

                match screenshot::capture_screenshot(url).await {
                    Ok(_) => {
                        success_count += 1;
                        logs.lock().unwrap().push(format!(
                            "✅ [{}/{}] 成功截图: {}",
                            current, total_count, url
                        ));
                    }
                    Err(e) => {
                        logs.lock().unwrap().push(format!(
                            "❌ [{}/{}] 截图失败 {}: {}",
                            current, total_count, url, e
                        ));
                    }
                }
            }

            // 完成提示
            logs.lock().unwrap().push("🎉 处理完成！".to_string());
            logs.lock().unwrap().push(format!(
                "📊 统计：成功 {}/{} 个网站",
                success_count, total_count
            ));

            format!("完成! 成功截图 {}/{} 个网站", success_count, total_count)
        }
        Err(e) => {
            logs.lock()
                .unwrap()
                .push(format!("❌ 错误：读取网址文件失败 - {}", e));
            format!("读取网址文件失败: {}", e)
        }
    }
}

pub fn run_gui() -> eframe::Result<()> {
    let system_root = env::var("SystemRoot").expect("无法获取系统根目录");
    let font_path = format!("{}\\Fonts\\msyh.ttc", system_root);

    let font_data = match fs::read(&font_path) {
        Ok(data) => data,
        Err(_) => {
            let fallback_font = format!("{}\\Fonts\\simsun.ttc", system_root);
            fs::read(fallback_font).expect("无法加载字体文件")
        }
    };

    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "Chinese Font".to_owned(),
        egui::FontData::from_owned(font_data),
    );
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "Chinese Font".to_owned());

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(700.0, 650.0)),
        min_window_size: Some(egui::vec2(600.0, 500.0)),
        ..Default::default()
    };

    eframe::run_native(
        &format!("网站截图工具 created by Rust Programming Language"),
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_fonts(fonts);
            Box::new(PicMyWebApp::new(cc))
        }),
    )
}
