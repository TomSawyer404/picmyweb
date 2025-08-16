use anyhow::Result;
use std::fs;

// 引入自定义模块
mod screenshot;
mod url_handler;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建截图目录
    fs::create_dir_all("screenshots")?;

    // 获取网站列表
    let websites = url_handler::get_websites()?;

    if websites.is_empty() {
        println!("没有找到要截图的网站");
        return Ok(());
    }

    println!("开始访问网站并截图...");

    for url in &websites {
        let result = async { screenshot::capture_screenshot(url).await }.await;

        if let Err(e) = result {
            println!("访问 {} 失败: {:?}, 跳过该网站", url, e);
            continue;
        }
    }

    println!("所有截图完成！");
    Ok(())
}
