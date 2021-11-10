use super::Point;

pub(super) struct SectionCollector {
    len_each: u16,
    current: u8,
    buffer: Vec<Point>,
}

impl SectionCollector {
    pub fn new(len_each: u16) -> Self {
        Self {
            len_each,
            current: 0,
            buffer: Vec::new(),
        }
    }

    pub fn push(&mut self, p: Point) -> Option<(u8, Vec<Point>)> {
        // 计算分区序号，如果不属于当前分区则更新分区
        let result = Some((p.dir / self.len_each) as u8)
            .filter(|i| *i != self.current)
            .map(|i| {
                let capacity = self.buffer.capacity();
                (
                    std::mem::replace(&mut self.current, i),
                    std::mem::replace(&mut self.buffer, Vec::with_capacity(capacity)),
                )
            });
        if p.len > 0 {
            self.buffer.push(p);
        }
        result
    }
}
