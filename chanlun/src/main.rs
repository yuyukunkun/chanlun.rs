/*
 * MIT License
 *
 * Copyright (c) 2026 YuYuKunKun
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

use chanlun::business::multi_frame::立体分析器;
use chanlun::business::observer::观察者;
use chanlun::config::缠论配置;
use chanlun::kline::bar::K线;
use std::env;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("用法:");
        eprintln!(
            "  {} read <.nb文件路径>     测试_读取数据（单周期分析）",
            args[0]
        );
        eprintln!(
            "  {} synth <.nb文件路径>    测试_周期合成（多周期合成）",
            args[0]
        );
        std::process::exit(1);
    }

    let 命令 = &args[1];

    match 命令.as_str() {
        "read" | "r" => {
            if args.len() < 3 {
                eprintln!("用法: {} read <.nb文件路径>", args[0]);
                std::process::exit(1);
            }
            测试_读取数据(&args[2]);
        }
        "synth" | "s" => {
            if args.len() < 3 {
                eprintln!("用法: {} synth <.nb文件路径>", args[0]);
                std::process::exit(1);
            }
            测试_周期合成(&args[2]);
        }
        // Backward compatibility: if first arg is a file path, treat as "read"
        other if other.ends_with(".nb") => {
            测试_读取数据(other);
        }
        _ => {
            eprintln!("未知命令: {}，请使用 read 或 synth", 命令);
            std::process::exit(1);
        }
    }
}

/// 测试_读取数据 — 单周期分析
fn 测试_读取数据(文件路径: &str) {
    let 启动时间 = Instant::now();

    let 配置 = 缠论配置::default().不推送();
    let 观察员 = 观察者::new("".into(), 0, 缠论配置::default());
    观察员
        .write()
        .读取数据文件(文件路径, 配置)
        .expect("读取数据文件失败");
    let 观察员 = 观察员.read();
    let 消耗用时 = 启动时间.elapsed();
    println!(
        "测试_读取数据 耗时 {:.2?} 普K数量 {}",
        消耗用时,
        观察员.普通K线序列.len()
    );
    println!("符号: {}", 观察员.符号);
    println!("周期: {}", 观察员.周期);
    println!("缠K数量: {}", 观察员.缠论K线序列.len());
    println!("分型数量: {}", 观察员.分型序列.len());
    println!("笔数量: {}", 观察员.笔序列.len());
    println!("笔中枢数量: {}", 观察员.笔_中枢序列.len());
    println!("线段数量: {}", 观察员.线段序列().len());
    println!("中枢数量: {}", 观察员.中枢序列().len());
    println!("扩展线段数量: {}", 观察员.扩展线段序列().len());
    println!("线段_线段序列数量: {}", 观察员.线段_线段序列().len());
    println!(
        "扩展线段_扩展线段数量: {}",
        观察员.扩展线段序列_扩展线段().len()
    );

    println!("\n===== 保存分析数据 =====\n");
    观察员.测试_保存数据(None);
}

/// 测试_周期合成 — 多周期合成分析
fn 测试_周期合成(文件路径: &str) {
    let 启动时间 = Instant::now();

    // Parse filename to extract metadata
    let path = std::path::Path::new(文件路径);
    let name = path
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    let parts: Vec<&str> = name.split('-').collect();
    if parts.len() < 4 {
        eprintln!(
            "invalid filename format: {}, expected 符号-周期-起始-结束",
            name
        );
        std::process::exit(1);
    }
    let 符号 = parts[0].to_string();
    let 周期: i64 = parts[1].parse().unwrap_or(300);
    let 周期组 = vec![周期, 周期 * 5, 周期 * 5 * 6];

    let 配置 = 缠论配置::default().不推送();
    let mut 多级别分析 = 立体分析器::new(符号.clone(), 周期组, Some(配置), None);

    // Read binary file and feed K-lines
    let data = match std::fs::read(文件路径) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("读取文件失败: {}", e);
            std::process::exit(1);
        }
    };

    let size: usize = 48; // 6 × f64 big-endian
    let k线数量 = data.len() / size;
    for i in 0..k线数量 {
        let offset = i * size;
        if let Some(k线) = K线::from_bytes(&data[offset..offset + size], 周期, &符号) {
            多级别分析.投喂K线(k线);
        }
    }

    let 消耗用时 = 启动时间.elapsed();
    println!("测试_周期合成 耗时 {:.2?} 普K数量 {}", 消耗用时, k线数量);
    println!("符号: {}", 符号);
    println!("周期组: {:?}", vec![周期, 周期 * 5, 周期 * 5 * 6]);

    // Display stats per period
    for &p in &[周期, 周期 * 5, 周期 * 5 * 6] {
        if let Some(观察员) = 多级别分析.获取观察者(p) {
            let 观察员 = 观察员.read();
            println!(
                "周期<{}>: 缠K={}, 分型={}, 笔={}, 线段={}, 中枢={}",
                p,
                观察员.缠论K线序列.len(),
                观察员.分型序列.len(),
                观察员.笔序列.len(),
                观察员.线段序列().len(),
                观察员.中枢序列().len(),
            );
        }
    }

    println!("\n===== 保存分析数据 =====\n");
    多级别分析.测试_保存数据(None);
}
