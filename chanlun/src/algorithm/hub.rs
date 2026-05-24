use crate::structure::dash_line::虚线;
use crate::structure::fractal_obj::分型;
use crate::types::相对方向;
use std::rc::Rc;

/// 中枢 — 三段虚线重叠区间构成的价格中枢
#[derive(Debug, Clone)]
pub struct 中枢 {
    pub 序号: i64,
    pub 标识: String,
    pub 级别: i64,
    pub 基础序列: Vec<Rc<虚线>>,
    pub 第三买卖线: Option<Rc<虚线>>,
    pub 本级_第三买卖线: Option<Rc<虚线>>,
}

impl 中枢 {
    pub fn new(序号: i64, 标识: String, 级别: i64, 基础序列: Vec<Rc<虚线>>) -> Self {
        Self {
            序号,
            标识,
            级别,
            基础序列: 基础序列.into_iter().take(3).collect(),
            第三买卖线: None,
            本级_第三买卖线: None,
        }
    }

    pub fn 添加虚线(&mut self, 实线: Rc<虚线>) {
        self.基础序列.push(实线);
        self.本级_第三买卖线 = None;
        self.第三买卖线 = None;
    }

    pub fn 图表标题(&self) -> String {
        format!(
            "{}:{}:{}:{}",
            self.文().中.标识, self.文().中.周期, self.标识, self.序号
        )
    }

    pub fn 离开段(&self) -> Rc<虚线> {
        Rc::clone(&self.基础序列[self.基础序列.len() - 1])
    }

    pub fn 方向(&self) -> 相对方向 {
        self.基础序列[0].方向().翻转()
    }

    pub fn 高(&self) -> f64 {
        self.基础序列[..3]
            .iter()
            .map(|x| x.高())
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0)
    }

    pub fn 低(&self) -> f64 {
        self.基础序列[..3]
            .iter()
            .map(|x| x.低())
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0)
    }

    pub fn 高高(&self) -> f64 {
        self.基础序列
            .iter()
            .map(|x| x.高())
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0)
    }

    pub fn 低低(&self) -> f64 {
        self.基础序列
            .iter()
            .map(|x| x.低())
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0)
    }

    pub fn 文(&self) -> Rc<分型> {
        Rc::clone(&self.基础序列[0].文)
    }

    pub fn 武(&self) -> Rc<分型> {
        Rc::clone(&self.基础序列[self.基础序列.len() - 1].武)
    }

    pub fn 设置第三买卖线(&mut self, 线: Rc<虚线>) {
        self.第三买卖线 = Some(线);
    }

    pub fn 获取数据文本(&self) -> String {
        let 第三买卖线_str = match &self.第三买卖线 {
            Some(x) => format!("{}", x),
            None => "None".to_string(),
        };
        let 本级_第三买卖线_str = match &self.本级_第三买卖线 {
            Some(x) => format!("{}", x),
            None => "None".to_string(),
        };
        format!(
            "{}, {}, {}, 文:({},{}), 武:({},{}), {}, {}",
            self.标识,
            self.序号,
            self.级别,
            self.文().时间戳,
            crate::utils::format_f64_g(self.文().分型特征值),
            self.武().时间戳,
            crate::utils::format_f64_g(self.武().分型特征值),
            第三买卖线_str,
            本级_第三买卖线_str,
        )
    }

    /// 校验中枢合法性
    pub fn 校验合法性(&self, 虚线序列: &[Rc<虚线>], _中枢序列: &[Rc<中枢>]) -> bool {
        // 1. 检查基础序列中的元素是否仍在虚线序列中
        for 元素 in &self.基础序列 {
            if !虚线序列.iter().any(|x| Rc::as_ptr(x) == Rc::as_ptr(元素)) {
                return false;
            }
        }
        // 2. 检查前三根重叠是否有效
        if self.基础序列.len() >= 3 {
            let 高 = self.高();
            let 低 = self.低();
            if 高 <= 低 {
                return false;
            }
        }
        true
    }

    // ---- 关联函数 ----

    /// 基础检查 — 三根虚线是否能形成中枢
    pub fn 基础检查(左: &虚线, 中: &虚线, 右: &虚线) -> bool {
        if !左.之后是(中) || !中.之后是(右) {
            return false;
        }
        let 高 = [左.高(), 中.高(), 右.高()]
            .iter()
            .copied()
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0);
        let 低 = [左.低(), 中.低(), 右.低()]
            .iter()
            .copied()
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0);
        高 > 低
    }

    /// 创建中枢
    pub fn 创建(左: Rc<虚线>, 中: Rc<虚线>, 右: Rc<虚线>, 级别: i64, 标识: &str) -> Self {
        Self::new(0, format!("{}中枢<{}>", 标识, 中.标识), 级别, vec![左, 中, 右])
    }

    /// 从序列中获取中枢
    pub fn 从序列中获取中枢(
        虚线序列: &[Rc<虚线>],
        起始方向: 相对方向,
        标识: &str,
    ) -> Option<Rc<中枢>> {
        for i in 2..虚线序列.len() {
            let 左 = &虚线序列[i - 2];
            let 中 = &虚线序列[i - 1];
            let 右 = &虚线序列[i];
            if Self::基础检查(左, 中, 右) && 左.方向() == 起始方向 {
                let 中枢 = Self::创建(Rc::clone(左), Rc::clone(中), Rc::clone(右), 0, 标识);
                return Some(Rc::new(中枢));
            }
        }
        None
    }

    /// 向中枢序列尾部添加
    pub fn 向中枢序列尾部添加(中枢序列: &mut Vec<Rc<中枢>>, 待添加中枢: Rc<中枢>) {
        // Dedup
        if let Some(前一个) = 中枢序列.last() {
            if 前一个.高() == 待添加中枢.高() && 前一个.低() == 待添加中枢.低() {
                return;
            }
        }
        // Set序号: Python — if中枢序列: 待添加中枢.序号 = 中枢序列[-1].序号 + 1 else: stays 0
        if let Some(前一个) = 中枢序列.last() {
            let 新序号 = 前一个.序号 + 1;
            // Clone to mutate since Rc is immutable
            let mut cloned = (*待添加中枢).clone();
            cloned.序号 = 新序号;
            中枢序列.push(Rc::new(cloned));
        } else {
            中枢序列.push(待添加中枢);
        }
    }

    /// 从中枢序列尾部弹出
    pub fn 从中枢序列尾部弹出(中枢序列: &mut Vec<Rc<中枢>>, 待弹出: &Rc<中枢>) -> Option<Rc<中枢>> {
        if 中枢序列.last().map(|x| Rc::as_ptr(x)) == Some(Rc::as_ptr(待弹出)) {
            中枢序列.pop()
        } else {
            None
        }
    }

    /// 中枢分析 — 从虚线序列生成中枢序列（增量算法）
    ///
    /// 每收到新的虚线序列数据后调用，更新中枢序列
    pub fn 分析(
        虚线序列: &[Rc<虚线>],
        中枢序列: &mut Vec<Rc<中枢>>,
        跳过首部: bool,
        标识: &str,
        层级: i64,
    ) {
        if 虚线序列.len() < 3 {
            return;
        }

        // 初始化第一个中枢
        if 中枢序列.is_empty() {
            for i in 1..虚线序列.len() - 1 {
                let 左 = &虚线序列[i - 1];
                let 中 = &虚线序列[i];
                let 右 = &虚线序列[i + 1];

                if Self::基础检查(左, 中, 右) {
                    // Python: 序号 = 虚线序列.index(左)
                    let 序号 = 虚线序列.iter().position(|x| Rc::as_ptr(x) == Rc::as_ptr(左)).unwrap_or(i - 1);
                    if 跳过首部 && (左.序号 == 0 || 序号 == 0) {
                        continue;
                    }
                    if 序号 >= 2 {
                        let 前 = &虚线序列[序号 - 2];
                        let 同向相对关系 = crate::types::相对方向::分析(前.高(), 前.低(), 左.高(), 左.低());
                        if 同向相对关系.是否向上() && 左.方向().是否向上() {
                            continue;
                        }
                        if 同向相对关系.是否向下() && 左.方向().是否向下() {
                            continue;
                        }
                    }
                    let 新中枢 = Rc::new(Self::创建(
                        Rc::clone(左),
                        Rc::clone(中),
                        Rc::clone(右),
                        中.级别,
                        标识,
                    ));
                    Self::向中枢序列尾部添加(中枢序列, 新中枢);
                    // Python: return 中枢递归分析(虚线序列, 中枢序列, ...)
                    Self::分析(虚线序列, 中枢序列, 跳过首部, 标识, 层级);
                    return;
                }
            }
            return;
        }

        // 增量更新
        let 当前中枢_idx = 中枢序列.len() - 1;
        let 当前中枢 = Rc::clone(&中枢序列[当前中枢_idx]);

        if !当前中枢.校验合法性(虚线序列, 中枢序列) {
            Self::从中枢序列尾部弹出(中枢序列, &当前中枢);
            Self::分析(虚线序列, 中枢序列, 跳过首部, 标识, 层级);
            return;
        }

        // 找到当前中枢最后一个元素在虚线序列中的位置
        let 最后元素 = &当前中枢.基础序列[当前中枢.基础序列.len() - 1];
        let 起始索引 = match 虚线序列
            .iter()
            .position(|x| Rc::as_ptr(x) == Rc::as_ptr(最后元素))
        {
            Some(idx) => idx + 1,
            None => return,
        };

        let 中枢高 = 当前中枢.高();
        let 中枢低 = 当前中枢.低();
        let mut 候选序列: Vec<Rc<虚线>> = Vec::new();

        let mut 新当前中枢 = (*当前中枢).clone();

        for i in 起始索引..虚线序列.len() {
            let 当前虚线 = Rc::clone(&虚线序列[i]);

            // 检查是否超出中枢范围（缺口）
            if crate::types::相对方向::分析(中枢高, 中枢低, 当前虚线.高(), 当前虚线.低()).是否缺口() {
                候选序列.push(当前虚线.clone());

                // Python: if 当前中枢.基础序列[-1].之后是(当前虚线):
                if 新当前中枢.基础序列.last().unwrap().之后是(&当前虚线) {
                    新当前中枢.设置第三买卖线(当前虚线.clone());
                }
            } else {
                if 候选序列.is_empty() {
                    // 仍在范围内：延伸中枢
                    新当前中枢.添加虚线(当前虚线);
                } else {
                    候选序列.push(当前虚线);
                }
            }

            // 候选序列积满3个：尝试创建新中枢
            while 候选序列.len() >= 3 {
                let 起始方向 = 新当前中枢.基础序列.last().unwrap().方向().翻转();
                match Self::从序列中获取中枢(&候选序列, 起始方向, 标识) {
                    Some(新中枢) => {
                        中枢序列[当前中枢_idx] = Rc::new(新当前中枢.clone());
                        Self::向中枢序列尾部添加(中枢序列, 新中枢);
                        return;
                    }
                    None => {
                        候选序列.remove(0); // 滑动窗口
                    }
                }
            }
        }

        // 更新当前中枢
        中枢序列[当前中枢_idx] = Rc::new(新当前中枢);
    }
}

impl std::fmt::Display for 中枢 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let 序列_str = self
            .基础序列
            .iter()
            .map(|d| format!("{}", d))
            .collect::<Vec<_>>()
            .join(", ");
        write!(
            f,
            "{}({}, {}, 元素数量: {}, [{}], {} ===>>> {})",
            self.标识,
            crate::utils::format_f64_g(self.高()),
            crate::utils::format_f64_g(self.低()),
            self.基础序列.len(),
            序列_str,
            self.文(),
            self.武(),
        )
    }
}
