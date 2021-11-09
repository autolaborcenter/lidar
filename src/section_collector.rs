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
            buffer: Vec::with_capacity(len_each as usize / 10),
        }
    }

    pub fn push(&mut self, p: Point) -> Option<(u8, Vec<Point>)> {
        let i = (p.dir / self.len_each) as u8;
        let result = if self.current == i {
            None
        } else {
            Some((
                std::mem::replace(&mut self.current, i),
                std::mem::replace(
                    &mut self.buffer,
                    Vec::with_capacity(self.len_each as usize / 10),
                ),
            ))
        };

        if p.len > 0 {
            self.buffer.push(p);
        }
        result
    }
}
