use crate::structure::segment_feat::线段特征;
use crate::types::分型结构;
use std::rc::Rc;

/// 特征分型 — 由三个线段特征元素构成的分型
#[derive(Debug, Clone)]
pub struct 特征分型 {
    pub 左: Rc<线段特征>,
    pub 中: Rc<线段特征>,
    pub 右: Rc<线段特征>,
    pub 结构: 分型结构,
}

impl 特征分型 {
    pub fn new(
        左: Rc<线段特征>, 中: Rc<线段特征>, 右: Rc<线段特征>, 结构: 分型结构
    ) -> Self {
        Self {
            左, 中, 右, 结构
        }
    }
}

impl std::fmt::Display for 特征分型 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "特征分型<{}, {}>", self.结构, self.中)
    }
}
