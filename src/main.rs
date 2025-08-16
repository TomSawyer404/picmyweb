#![windows_subsystem = "windows"]

use anyhow::Result;

mod gui;
mod screenshot;
mod url_handler;

#[tokio::main]
async fn main() -> Result<()> {
    gui::run_gui().map_err(|e| anyhow::anyhow!("GUI错误: {}", e))
}
