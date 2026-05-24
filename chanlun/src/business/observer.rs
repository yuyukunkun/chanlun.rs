use crate::algorithm::bi::笔;
use crate::algorithm::hub::中枢;
use crate::algorithm::segment::线段;
use crate::config::缠论配置;
use crate::kline::bar::K线;
use crate::kline::chan_kline::缠论K线;
use crate::structure::dash_line::虚线;
use crate::structure::fractal_obj::分型;
use crate::types::相对方向;
use crate::utils::datetime;
use std::rc::Rc;

/// 观察者 — 单周期分析器，持有所有层级序列，接收K线流式输入后逐层计算
pub struct 观察者 {
    pub 符号: String,
    pub 周期: i64,
    pub 配置: 缠论配置,

    // K线序列
    pub 普通K线序列: Vec<Rc<K线>>,
    pub 缠论K线序列: Vec<Rc<缠论K线>>,

    // 分型与笔
    pub 分型序列: Vec<Rc<分型>>,
    pub 笔序列: Vec<Rc<虚线>>,
    pub 笔_中枢序列: Vec<Rc<中枢>>,

    // 线段
    pub 线段序列: Vec<Rc<虚线>>,
    pub 中枢序列: Vec<Rc<中枢>>,

    // 扩展线段（笔级）
    pub 扩展线段序列: Vec<Rc<虚线>>,
    pub 扩展中枢序列: Vec<Rc<中枢>>,

    // 扩展线段（线段级）
    pub 扩展线段序列_线段: Vec<Rc<虚线>>,
    pub 扩展中枢序列_线段: Vec<Rc<中枢>>,

    // 线段之线段
    pub 线段_线段序列: Vec<Rc<虚线>>,
    pub 线段_中枢序列: Vec<Rc<中枢>>,

    // 扩展线段之扩展线段
    pub 扩展线段序列_扩展线段: Vec<Rc<虚线>>,
    pub 扩展中枢序列_扩展线段: Vec<Rc<中枢>>,

    // 终止时间戳
    终止时间戳: Option<i64>,
}

impl 观察者 {
    pub fn new(符号: String, 周期: i64, 配置: 缠论配置) -> Self {
        let 终止时间戳 = if 配置.手动终止 != "1970-01-01 00:00:00" && !配置.手动终止.is_empty()
        {
            datetime::转化为时间戳(&配置.手动终止)
        } else {
            None
        };

        let mut instance = Self {
            符号: 符号.clone(),
            周期,
            配置,
            普通K线序列: Vec::new(),
            缠论K线序列: Vec::new(),
            分型序列: Vec::new(),
            笔序列: Vec::new(),
            笔_中枢序列: Vec::new(),
            线段序列: Vec::new(),
            中枢序列: Vec::new(),
            扩展线段序列: Vec::new(),
            扩展中枢序列: Vec::new(),
            扩展线段序列_线段: Vec::new(),
            扩展中枢序列_线段: Vec::new(),
            线段_线段序列: Vec::new(),
            线段_中枢序列: Vec::new(),
            扩展线段序列_扩展线段: Vec::new(),
            扩展中枢序列_扩展线段: Vec::new(),
            终止时间戳,
        };
        instance.配置.标识 = 符号;
        instance
    }

    /// 标识
    pub fn 标识(&self) -> String {
        format!("{}:{}", self.符号, self.周期)
    }

    /// 当前K线
    pub fn 当前K线(&self) -> Option<&Rc<K线>> {
        self.普通K线序列.last()
    }

    /// 当前缠K
    pub fn 当前缠K(&self) -> Option<&Rc<缠论K线>> {
        self.缠论K线序列.last()
    }

    /// 重置基础序列
    pub fn 重置基础序列(&mut self) {
        self.普通K线序列.clear();
        self.缠论K线序列.clear();
        self.分型序列.clear();
        self.笔序列.clear();
        self.笔_中枢序列.clear();
        self.线段序列.clear();
        self.中枢序列.clear();
        self.扩展线段序列.clear();
        self.扩展中枢序列.clear();
        self.扩展线段序列_线段.clear();
        self.扩展中枢序列_线段.clear();
        self.线段_线段序列.clear();
        self.线段_中枢序列.clear();
        self.扩展线段序列_扩展线段.clear();
        self.扩展中枢序列_扩展线段.clear();
    }

    /// 增加原始K线 — 单根K线投喂入口
    pub fn 增加原始K线(&mut self, 普K: K线) {
        if let Some(终止) = self.终止时间戳 {
            if 普K.时间戳 > 终止 {
                return;
            }
        }
        self.__处理数据(普K);
    }

    /// 核心数据处理管道
    fn __处理数据(&mut self, 普K: K线) {
        // Step 1: 缠论K线分析 (普K is consumed by 分析 as &mut)
        let (_, 当前分型) = 缠论K线::分析(
            普K,
            &mut self.缠论K线序列,
            &mut self.普通K线序列,
            &self.配置,
        );
        let 当前分型 = match 当前分型 {
            Some(fx) => fx,
            None => return,
        };

        // Step 2: 笔分析
        if self.配置.分析笔 {
            笔::分析(
                当前分型,
                &mut self.分型序列,
                &mut self.笔序列,
                &self.缠论K线序列,
                &self.普通K线序列,
                &self.配置,
            );
        }
        if self.分型序列.is_empty() {
            return;
        }

        // Step 3: 笔中枢分析
        if self.配置.分析笔中枢 {
            中枢::分析(&self.笔序列, &mut self.笔_中枢序列, true, "", 0);
        }
        if self.笔序列.is_empty() {
            return;
        }

        // Step 4: 线段分析
        if self.配置.分析线段 {
            线段::分析(
                &self.笔序列,
                &mut self.线段序列,
                &self.配置,
                0,
                &[相对方向::向上, 相对方向::向下],
            );
        }
        if self.配置.分析线段中枢 {
            中枢::分析(&self.线段序列, &mut self.中枢序列, true, "", 0);
        }

        // Step 5: 扩展线段（笔级）
        if self.配置.分析扩展线段 {
            线段::扩展分析(&self.笔序列, &mut self.扩展线段序列, &self.配置);
        }
        if self.配置.分析线段中枢 {
            中枢::分析(&self.扩展线段序列, &mut self.扩展中枢序列, true, "", 0);
        }

        // Step 6: 扩展线段（线段级）
        if self.配置.分析扩展线段 {
            线段::扩展分析(&self.线段序列, &mut self.扩展线段序列_线段, &self.配置);
        }
        if self.配置.分析线段中枢 {
            中枢::分析(
                &self.扩展线段序列_线段,
                &mut self.扩展中枢序列_线段,
                true,
                "",
                0,
            );
        }

        // Step 7: 线段之线段
        if self.配置.分析线段 {
            线段::分析(
                &self.线段序列,
                &mut self.线段_线段序列,
                &self.配置,
                0,
                &[
                    相对方向::向下,
                    相对方向::向上,
                    相对方向::顺,
                    相对方向::逆,
                    相对方向::同,
                ],
            );
        }
        if self.配置.分析线段中枢 {
            中枢::分析(&self.线段_线段序列, &mut self.线段_中枢序列, true, "", 0);
        }

        // Step 8: 扩展线段之扩展线段
        if self.配置.分析扩展线段 {
            线段::扩展分析(
                &self.扩展线段序列,
                &mut self.扩展线段序列_扩展线段,
                &self.配置,
            );
        }
        if self.配置.分析线段中枢 {
            中枢::分析(
                &self.扩展线段序列_扩展线段,
                &mut self.扩展中枢序列_扩展线段,
                true,
                "",
                0,
            );
        }
    }

    /// 静态重新分析 — 遍历所有缠K重新生成分型/笔/线段
    pub fn 静态重新分析(&mut self) {
        self.分型序列.clear();
        self.笔序列.clear();
        self.笔_中枢序列.clear();
        self.线段序列.clear();
        self.中枢序列.clear();
        self.扩展线段序列.clear();
        self.扩展中枢序列.clear();
        self.扩展线段序列_线段.clear();
        self.扩展中枢序列_线段.clear();
        self.线段_线段序列.clear();
        self.线段_中枢序列.clear();
        self.扩展线段序列_扩展线段.clear();
        self.扩展中枢序列_扩展线段.clear();

        for i in 1..self.缠论K线序列.len() - 1 {
            let 当前分型 = 分型::new(
                Some(Rc::clone(&self.缠论K线序列[i - 1])),
                Rc::clone(&self.缠论K线序列[i]),
                Some(Rc::clone(&self.缠论K线序列[i + 1])),
            );
            笔::分析(
                Rc::new(当前分型),
                &mut self.分型序列,
                &mut self.笔序列,
                &self.缠论K线序列,
                &self.普通K线序列,
                &self.配置,
            );
        }

        if self.配置.分析笔中枢 {
            中枢::分析(&self.笔序列, &mut self.笔_中枢序列, true, "", 0);
        }

        if self.配置.分析线段 {
            线段::分析(
                &self.笔序列,
                &mut self.线段序列,
                &self.配置,
                0,
                &[相对方向::向上, 相对方向::向下],
            );
        }
        if self.配置.分析线段中枢 {
            中枢::分析(&self.线段序列, &mut self.中枢序列, true, "", 0);
        }

        if self.配置.分析扩展线段 {
            线段::扩展分析(&self.笔序列, &mut self.扩展线段序列, &self.配置);
        }
        if self.配置.分析线段中枢 {
            中枢::分析(&self.扩展线段序列, &mut self.扩展中枢序列, true, "", 0);
        }

        if self.配置.分析扩展线段 {
            线段::扩展分析(&self.线段序列, &mut self.扩展线段序列_线段, &self.配置);
        }
        if self.配置.分析线段中枢 {
            中枢::分析(
                &self.扩展线段序列_线段,
                &mut self.扩展中枢序列_线段,
                true,
                "",
                0,
            );
        }

        if self.配置.分析线段 {
            线段::分析(
                &self.线段序列,
                &mut self.线段_线段序列,
                &self.配置,
                0,
                &[
                    相对方向::向下,
                    相对方向::向上,
                    相对方向::顺,
                    相对方向::逆,
                    相对方向::同,
                ],
            );
        }
        if self.配置.分析线段中枢 {
            中枢::分析(&self.线段_线段序列, &mut self.线段_中枢序列, true, "", 0);
        }
    }

    /// 测试_保存数据 — 输出各序列数据文本到文件
    pub fn 测试_保存数据(&self, root: Option<&str>) {
        let 笔序列_文本数据: Vec<String> = self.笔序列.iter().map(|b| b.获取数据文本()).collect();
        let 线段序列_文本数据: Vec<String> =
            self.线段序列.iter().map(|s| s.获取数据文本()).collect();
        let 扩展线段序列_数据文本: Vec<String> =
            self.扩展线段序列.iter().map(|s| s.获取数据文本()).collect();
        let 扩展线段序列_线段_数据文本: Vec<String> = self
            .扩展线段序列_线段
            .iter()
            .map(|s| s.获取数据文本())
            .collect();
        let 线段_线段序列_数据文本: Vec<String> = self
            .线段_线段序列
            .iter()
            .map(|s| s.获取数据文本())
            .collect();
        let 扩展线段序列_扩展线段_数据文本: Vec<String> = self
            .扩展线段序列_扩展线段
            .iter()
            .map(|s| s.获取数据文本())
            .collect();

        let 笔_中枢序列_数据文本: Vec<String> =
            self.笔_中枢序列.iter().map(|h| h.获取数据文本()).collect();
        let 中枢序列_数据文本: Vec<String> =
            self.中枢序列.iter().map(|h| h.获取数据文本()).collect();
        let 扩展中枢序列_数据文本: Vec<String> =
            self.扩展中枢序列.iter().map(|h| h.获取数据文本()).collect();
        let 扩展中枢序列_线段_数据文本: Vec<String> = self
            .扩展中枢序列_线段
            .iter()
            .map(|h| h.获取数据文本())
            .collect();
        let 线段_中枢序列_数据文本: Vec<String> = self
            .线段_中枢序列
            .iter()
            .map(|h| h.获取数据文本())
            .collect();
        let 扩展中枢序列_扩展线段_数据文本: Vec<String> = self
            .扩展中枢序列_扩展线段
            .iter()
            .map(|h| h.获取数据文本())
            .collect();

        // 确定根目录
        let 根目录 = match root {
            Some(r) => std::path::PathBuf::from(r),
            None => std::env::current_dir().unwrap_or_default(),
        };

        // 生成子目录名称
        let 起始时间 = self.普通K线序列.first().map(|k| k.时间戳).unwrap_or(0);
        let 结束时间 = self.普通K线序列.last().map(|k| k.时间戳).unwrap_or(0);
        let 目录标识 = format!("Rust_{}:{}_{}_{}", self.符号, self.周期, 起始时间, 结束时间);

        let 保存路径 = 根目录.join(&目录标识);
        if let Err(e) = std::fs::create_dir_all(&保存路径) {
            eprintln!("创建目录失败: {} -> {}", 保存路径.display(), e);
            return;
        }

        // 缠K data for debugging
        let 缠K序列_数据文本: Vec<String> = self
            .缠论K线序列
            .iter()
            .map(|ck| {
                format!(
                    "缠K, {}, {}, {:?}, {}, {}, {}, {}, {}",
                    ck.序号,
                    ck.时间戳,
                    ck.分型,
                    ck.方向,
                    ck.高,
                    ck.低,
                    ck.原始起始序号,
                    ck.原始结束序号
                )
            })
            .collect();
        let 分型序列_数据文本: Vec<String> = self
            .分型序列
            .iter()
            .enumerate()
            .map(|(i, fx)| {
                format!(
                    "分型, {}, {}, {:?}, {}, {}, {}",
                    i, fx.时间戳, fx.结构, fx.分型特征值, fx.中.时间戳, fx.中.低,
                )
            })
            .collect();

        let 数据映射: Vec<(&str, &[String])> = vec![
            ("笔序列_文本数据", &笔序列_文本数据),
            ("线段序列_文本数据", &线段序列_文本数据),
            ("扩展线段序列_数据文本", &扩展线段序列_数据文本),
            ("扩展线段序列_线段_数据文本", &扩展线段序列_线段_数据文本),
            ("线段_线段序列_数据文本", &线段_线段序列_数据文本),
            (
                "扩展线段序列_扩展线段_数据文本",
                &扩展线段序列_扩展线段_数据文本,
            ),
            ("笔_中枢序列_数据文本", &笔_中枢序列_数据文本),
            ("中枢序列_数据文本", &中枢序列_数据文本),
            ("扩展中枢序列_数据文本", &扩展中枢序列_数据文本),
            ("扩展中枢序列_线段_数据文本", &扩展中枢序列_线段_数据文本),
            ("线段_中枢序列_数据文本", &线段_中枢序列_数据文本),
            (
                "扩展中枢序列_扩展线段_数据文本",
                &扩展中枢序列_扩展线段_数据文本,
            ),
            ("缠K序列_数据文本", &缠K序列_数据文本),
            ("分型序列_数据文本", &分型序列_数据文本),
        ];

        for (文件名, 数据列表) in &数据映射 {
            let 文件路径 = 保存路径.join(format!("{}.txt", 文件名));
            let 内容 = 数据列表.join("\n") + "\n";
            if let Err(e) = std::fs::write(&文件路径, &内容) {
                eprintln!("写入文件失败: {} -> {}", 文件路径.display(), e);
            }
        }

        println!("全部数据拆分保存完成，目录：{}", 保存路径.display());
    }

    /// 读取数据文件 — 从 .nb 文件加载数据
    pub fn 读取数据文件(
        文件路径: &str, 配置: Option<缠论配置>
    ) -> Result<Self, String> {
        let 配置 = 配置.unwrap_or_default();

        // Parse filename: btcusd-300-1631772074-1632222374.nb
        let path = std::path::Path::new(文件路径);
        let name = path
            .file_stem()
            .and_then(|n| n.to_str())
            .ok_or("invalid filename")?;
        let parts: Vec<&str> = name.split('-').collect();
        if parts.len() < 4 {
            return Err(format!("invalid filename format: {}", name));
        }
        let 符号 = parts[0].to_string();
        let 周期: i64 = parts[1]
            .parse()
            .map_err(|e| format!("parse period: {}", e))?;

        let mut 实例 = Self::new(符号, 周期, 配置);

        let data = std::fs::read(文件路径).map_err(|e| format!("read file: {}", e))?;
        let size = 48; // 6 × 8 bytes (big-endian double)
        for i in 0..data.len() / size {
            let offset = i * size;
            if let Some(k线) = K线::from_bytes(&data[offset..offset + size], 周期, "nb") {
                实例.增加原始K线(k线);
            }
        }

        Ok(实例)
    }
}
