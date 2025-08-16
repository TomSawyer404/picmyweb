use anyhow::{Context, Result};
use std::fs;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::path::Path;
use std::str::FromStr;

/// 从文件中读取网站列表
pub fn read_websites_from_file(file_path: &str) -> Result<Vec<String>> {
    let content = fs::read_to_string(file_path).context("无法读取文件")?;
    let websites: Vec<String> = content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .flat_map(|line| generate_urls(line))
        .collect();
    Ok(websites)
}

/// 为IP地址或IP:port生成http和https两个URL，其他情况只生成https
fn generate_urls(input: &str) -> Vec<String> {
    // 如果已经包含协议，则直接返回
    if input.starts_with("http://") || input.starts_with("https://") {
        return vec![input.to_string()];
    }

    // 检查是否是IP地址 (IPv4 或 IPv6) 或 IP:port 形式
    if is_ip_address(input) || is_ip_with_port(input) {
        // 为IP地址或IP:port生成http和https两个URL
        return vec![format!("http://{}", input), format!("https://{}", input)];
    }

    // 对于域名，只生成http
    vec![format!("http://{}", input)]
}

/// 检查字符串是否是有效的IP地址 (IPv4 或 IPv6)
fn is_ip_address(input: &str) -> bool {
    // 尝试解析 IPv4 地址
    if Ipv4Addr::from_str(input).is_ok() {
        return true;
    }

    // 尝试解析 IPv6 地址
    if Ipv6Addr::from_str(input).is_ok() {
        return true;
    }

    false
}

/// 检查字符串是否是 IP:port 形式
fn is_ip_with_port(input: &str) -> bool {
    // 分割 IP 和端口
    let parts: Vec<&str> = input.split(':').collect();

    // 至少要有 IP 和端口两部分
    if parts.len() < 2 {
        return false;
    }

    // 检查最后一部分是否是数字端口
    let port_parse_result = parts[parts.len() - 1].parse::<u16>();
    if port_parse_result.is_ok() {
        // 检查前面的部分是否是有效的IP地址
        let ip_part = &parts[..parts.len() - 1].join(":");

        // 对于 IPv6，需要特殊处理，因为 IPv6 地址本身包含冒号
        if ip_part.starts_with('[') && ip_part.ends_with(']') {
            // IPv6 地址形式 [IPv6]:port
            let ipv6_str = &ip_part[1..ip_part.len() - 1];
            return Ipv6Addr::from_str(ipv6_str).is_ok();
        } else if parts.len() == 2 {
            // IPv4:port 形式
            return Ipv4Addr::from_str(parts[0]).is_ok();
        } else if parts.len() > 2 {
            // 可能是 IPv6:port 形式，但没有用方括号括起来
            // 尝试将所有部分（除了最后一个端口）重新组合为 IPv6 地址
            let ipv6_str = parts[..parts.len() - 1].join(":");
            return Ipv6Addr::from_str(&ipv6_str).is_ok();
        }
    }

    false
}

/// 获取要截图的网站列表
pub fn get_websites() -> Result<Vec<String>> {
    // 固定从 websites.txt 文件读取
    let file_path = "websites.txt";
    if Path::new(file_path).exists() {
        return read_websites_from_file(file_path);
    } else {
        eprintln!("文件 {} 不存在", file_path);
        return Err(anyhow::anyhow!("文件 {} 不存在", file_path));
    }
}
