use anyhow::{Context, Result};
use headless_chrome::{protocol::cdp::Page, Browser, LaunchOptions};
use std::fs;

/// 清理文件名中的非法字符
fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            _ => c,
        })
        .collect()
}

/// 截取网站截图
pub async fn capture_screenshot(url: &str) -> Result<()> {
    // 启动Chrome浏览器，设置窗口大小以美化截图比例
    let browser = Browser::new(
        LaunchOptions::default_builder()
            .headless(true)
            .window_size(Some((414, 896))) // iPhone XR 的尺寸，更美观的移动设备比例
            .build()
            .context("无法启动浏览器")?,
    )?;

    let tab = browser.new_tab()?;

    // 设置移动设备参数
    tab.set_user_agent(
        "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.0 Mobile/15E148 Safari/604.1",
        None,
        None,
    )?;

    println!("正在访问: {}", url);

    // 导航到网页
    tab.navigate_to(url)?;
    tab.wait_until_navigated()?;

    // 等待页面完全加载
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // 生成文件名
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let domain = url
        .split("//")
        .nth(1)
        .unwrap_or(url)
        .split('/')
        .next()
        .unwrap_or("unknown");
    let filename = format!(
        "screenshots/{}_{}.png",
        timestamp,
        sanitize_filename(domain)
    );

    // 截取全页截图
    let png_data =
        tab.capture_screenshot(Page::CaptureScreenshotFormatOption::Png, None, None, true)?;

    // 保存截图
    fs::write(&filename, png_data).context("无法保存截图")?;

    println!("已保存: {}", filename);
    Ok(())
}
