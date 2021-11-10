use driver::{Driver, MultipleDeviceDriver};
use std::time::{Duration, Instant};

mod section_collector;

use section_collector::SectionCollector;

pub extern crate driver;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Config {
    pub len_meter: u16,
    pub dir_round: u16,
    pub zipped_size: usize,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Point {
    pub len: u16,
    pub dir: u16,
}

pub trait LidarDriver: 'static + Send + Sized {
    type Key;

    /// 列出操作系统上的设备键-
    fn keys() -> Vec<Self::Key>;

    /// 启动确认超时
    fn open_timeout() -> Duration;

    /// 解析超时
    fn parse_timeout() -> Duration;

    /// 一整圈对应的角度值
    fn max_dir() -> u16;

    fn new(t: &Self::Key) -> Option<Self>;

    /// 从设备中接收数据到缓冲区
    fn receive(&mut self) -> bool;

    /// 从缓冲区解析一点
    ///
    /// 缓冲区空时返回 [`None`]
    fn parse(&mut self) -> Option<Point>;
}

pub struct Lidar<T: LidarDriver> {
    /// 内部实际驱动，用于支持设备特异性功能
    pub inner: T,
    last_time: Instant,
    section: SectionCollector,
    /// 点过滤器
    pub filter: fn(Point) -> bool,
}

impl<T: LidarDriver> MultipleDeviceDriver for Lidar<T> {
    type Command = fn(Point) -> bool;

    fn send(&mut self, command: Self::Command) {
        self.filter = command;
    }
}

impl<T: LidarDriver> Driver for Lidar<T> {
    type Pacemaker = ();
    type Key = T::Key;
    type Event = (u8, Vec<Point>);

    fn keys() -> Vec<Self::Key> {
        T::keys()
    }

    fn open_timeout() -> Duration {
        T::open_timeout()
    }

    fn new(t: &Self::Key) -> Option<(Self::Pacemaker, Self)> {
        T::new(t).map(|t| {
            (
                (),
                Self {
                    inner: t,
                    last_time: Instant::now(),
                    section: SectionCollector::new(T::max_dir() / 8),
                    filter: |_| true,
                },
            )
        })
    }

    fn join<F>(&mut self, mut f: F) -> bool
    where
        F: FnMut(&mut Self, Option<(std::time::Instant, Self::Event)>) -> bool,
    {
        let mut time = Instant::now();
        loop {
            if let Some(mut p) = self.inner.parse() {
                time = self.last_time;
                // 过滤
                if p.len != 0 && !(self.filter)(p) {
                    p.len = 0;
                }
                if let Some(section) = self.section.push(p) {
                    if !f(self, Some((time, section))) {
                        // 如果回调指示不要继续阻塞，立即退出
                        return true;
                    }
                }
            }
            // 解析超时
            else if self.last_time > time + T::parse_timeout() {
                return false;
            }
            // 成功接收
            else if self.inner.receive() {
                self.last_time = Instant::now();
            }
            // 接收失败
            else {
                return false;
            }
        }
    }
}
