#[derive(Debug, Copy, Clone)]
pub struct WindowInfo {
    logical_size: Size,
    physical_size: PhySize,
    scale: f64,
    scale_recip: f64,
}

impl WindowInfo {
    pub fn from_logical_size(logical_size: Size, scale: f64) -> Self {
        let scale_recip = if scale == 1.0 { 1.0 } else { 1.0 / scale };

        let physical_size = PhySize {
            width: (logical_size.width * scale).round() as u32,
            height: (logical_size.height * scale).round() as u32,
        };

        Self { logical_size, physical_size, scale, scale_recip }
    }

    pub fn from_physical_size(physical_size: PhySize, scale: f64) -> Self {
        let scale_recip = if scale == 1.0 { 1.0 } else { 1.0 / scale };

        let logical_size = Size {
            width: f64::from(physical_size.width) * scale_recip,
            height: f64::from(physical_size.height) * scale_recip,
        };

        Self { logical_size, physical_size, scale, scale_recip }
    }

    pub fn logical_size(&self) -> Size {
        self.logical_size
    }

    pub fn physical_size(&self) -> PhySize {
        self.physical_size
    }

    pub fn scale(&self) -> f64 {
        self.scale
    }

    pub fn scale_recip(&self) -> f64 {
        self.scale_recip
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn to_physical(&self, window_info: &WindowInfo) -> PhyPoint {
        PhyPoint {
            x: (self.x * window_info.scale()).round() as i32,
            y: (self.y * window_info.scale()).round() as i32,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PhyPoint {
    pub x: i32,
    pub y: i32,
}

impl PhyPoint {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn to_logical(&self, window_info: &WindowInfo) -> Point {
        Point {
            x: f64::from(self.x) * window_info.scale_recip(),
            y: f64::from(self.y) * window_info.scale_recip(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

impl Size {
    pub fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }

    #[inline]
    pub fn to_physical(&self, window_info: &WindowInfo) -> PhySize {
        PhySize {
            width: (self.width * window_info.scale()).round() as u32,
            height: (self.height * window_info.scale()).round() as u32,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PhySize {
    pub width: u32,
    pub height: u32,
}

impl PhySize {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    #[inline]
    pub fn to_logical(&self, window_info: &WindowInfo) -> Size {
        Size {
            width: f64::from(self.width) * window_info.scale_recip(),
            height: f64::from(self.height) * window_info.scale_recip(),
        }
    }
}
