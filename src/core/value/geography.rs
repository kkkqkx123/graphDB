//! 地理空间类型模块
//!
//! 本模块定义了地理空间类型及其相关操作，支持点、线、多边形等几何形状。

use bincode::{Decode, Encode};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

/// 地理形状类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Encode, Decode)]
pub enum GeoShape {
    Point,
    LineString,
    Polygon,
}

/// 地理坐标
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Encode, Decode)]
pub struct Coordinate {
    pub x: f64,
    pub y: f64,
}

impl std::hash::Hash for Coordinate {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
    }
}

impl Eq for Coordinate {}

impl Coordinate {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn normalize(&mut self) {
        self.x = ((self.x + 180.0) % 360.0) - 180.0;
        self.y = self.y.clamp(-90.0, 90.0);
    }

    pub fn is_valid(&self) -> bool {
        self.x >= -180.0 && self.x <= 180.0 && self.y >= -90.0 && self.y <= 90.0
    }
}

/// 线
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Encode, Decode)]
pub struct LineString {
    pub coordinates: Vec<Coordinate>,
}

impl LineString {
    pub fn new() -> Self {
        Self {
            coordinates: Vec::new(),
        }
    }

    pub fn add_point(&mut self, coord: Coordinate) {
        self.coordinates.push(coord);
    }

    pub fn length(&self) -> f64 {
        let mut total = 0.0;
        for window in self.coordinates.windows(2) {
            total += Self::haversine_distance(
                window[0].y, window[0].x,
                window[1].y, window[1].x,
            );
        }
        total
    }

    pub fn is_valid(&self) -> bool {
        self.coordinates.len() >= 2
            && self.coordinates.iter().all(|c| c.is_valid())
    }

    fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
        const R: f64 = 6371.0;
        const DEG_TO_RAD: f64 = std::f64::consts::PI / 180.0;

        let dlat = (lat2 - lat1) * DEG_TO_RAD;
        let dlon = (lon2 - lon1) * DEG_TO_RAD;

        let a = (dlat / 2.0).sin().powi(2)
            + lat1 * DEG_TO_RAD * lat2 * DEG_TO_RAD * (dlon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        R * c
    }
}

/// 多边形
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Encode, Decode)]
pub struct Polygon {
    pub rings: Vec<Vec<Coordinate>>,
}

impl Polygon {
    pub fn new() -> Self {
        Self { rings: Vec::new() }
    }

    pub fn add_ring(&mut self, ring: Vec<Coordinate>) {
        self.rings.push(ring);
    }

    pub fn area(&self) -> f64 {
        if self.rings.is_empty() {
            return 0.0;
        }

        let exterior = &self.rings[0];
        let mut area = 0.0;
        let n = exterior.len();

        for i in 0..n {
            let j = (i + 1) % n;
            area += exterior[i].x * exterior[j].y;
            area -= exterior[j].x * exterior[i].y;
        }

        area.abs() / 2.0
    }

    pub fn contains_point(&self, point: &Coordinate) -> bool {
        if self.rings.is_empty() {
            return false;
        }

        let exterior = &self.rings[0];
        Self::point_in_polygon(point, exterior)
    }

    fn point_in_polygon(point: &Coordinate, polygon: &[Coordinate]) -> bool {
        let mut inside = false;
        let n = polygon.len();

        for i in 0..n {
            let j = (i + 1) % n;
            let xi = polygon[i].x;
            let yi = polygon[i].y;
            let xj = polygon[j].x;
            let yj = polygon[j].y;

            if ((yi > point.y) != (yj > point.y))
                && (point.x < (xj - xi) * (point.y - yi) / (yj - yi) + xi)
            {
                inside = !inside;
            }
        }

        inside
    }

    pub fn is_valid(&self) -> bool {
        !self.rings.is_empty()
            && self.rings.iter().all(|ring| {
                ring.len() >= 3 && ring.iter().all(|c| c.is_valid())
            })
    }

    /// 转换为 WKT 格式
    pub fn as_wkt(&self) -> String {
        let rings: Vec<String> = self.rings
            .iter()
            .map(|ring| {
                let coords: Vec<String> = ring
                    .iter()
                    .map(|c| format!("{} {}", c.x, c.y))
                    .collect();
                format!("({})", coords.join(", "))
            })
            .collect();
        format!("POLYGON({})", rings.join(", "))
    }
}

/// 地理信息表示
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Encode, Decode)]
pub struct GeographyValue {
    pub latitude: f64,
    pub longitude: f64,
}

impl std::hash::Hash for GeographyValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.latitude.to_bits().hash(state);
        self.longitude.to_bits().hash(state);
    }
}

impl GeographyValue {
    /// 计算两点之间的 Haversine 距离（单位：公里）
    pub fn distance(&self, other: &GeographyValue) -> f64 {
        const EARTH_RADIUS_KM: f64 = 6371.0;
        const DEG_TO_RAD: f64 = std::f64::consts::PI / 180.0;

        let lat1 = self.latitude * DEG_TO_RAD;
        let lat2 = other.latitude * DEG_TO_RAD;
        let delta_lat = (other.latitude - self.latitude) * DEG_TO_RAD;
        let delta_lon = (other.longitude - self.longitude) * DEG_TO_RAD;

        let a = (delta_lat / 2.0).sin().powi(2)
            + lat1.cos() * lat2.cos() * (delta_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        EARTH_RADIUS_KM * c
    }

    /// 计算两点之间的方位角（单位：度）
    pub fn bearing(&self, other: &GeographyValue) -> f64 {
        const DEG_TO_RAD: f64 = std::f64::consts::PI / 180.0;
        const RAD_TO_DEG: f64 = 180.0 / std::f64::consts::PI;

        let lat1 = self.latitude * DEG_TO_RAD;
        let lat2 = other.latitude * DEG_TO_RAD;
        let delta_lon = (other.longitude - self.longitude) * DEG_TO_RAD;

        let y = delta_lon.sin() * lat2.cos();
        let x = lat1.cos() * lat2.sin() - lat1.sin() * lat2.cos() * delta_lon.cos();

        let bearing = y.atan2(x) * RAD_TO_DEG;
        (bearing + 360.0) % 360.0
    }

    /// 检查点是否在指定矩形区域内
    pub fn in_bbox(
        &self,
        min_lat: f64,
        max_lat: f64,
        min_lon: f64,
        max_lon: f64,
    ) -> bool {
        self.latitude >= min_lat
            && self.latitude <= max_lat
            && self.longitude >= min_lon
            && self.longitude <= max_lon
    }

    /// 估算地理值的内存使用大小
    pub fn estimated_size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}

impl Default for GeographyValue {
    fn default() -> Self {
        GeographyValue {
            latitude: 0.0,
            longitude: 0.0,
        }
    }
}

/// 扩展的地理类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Encode, Decode)]
pub enum Geography {
    Point(GeographyValue),
    LineString(LineString),
    Polygon(Polygon),
}

impl Geography {
    pub fn shape(&self) -> GeoShape {
        match self {
            Geography::Point(_) => GeoShape::Point,
            Geography::LineString(_) => GeoShape::LineString,
            Geography::Polygon(_) => GeoShape::Polygon,
        }
    }

    pub fn as_wkt(&self) -> String {
        match self {
            Geography::Point(geo) => {
                format!("POINT({} {})", geo.longitude, geo.latitude)
            }
            Geography::LineString(ls) => {
                let coords: Vec<String> = ls.coordinates
                    .iter()
                    .map(|c| format!("{} {}", c.x, c.y))
                    .collect();
                format!("LINESTRING({})", coords.join(", "))
            }
            Geography::Polygon(poly) => poly.as_wkt(),
        }
    }

    pub fn from_wkt(wkt: &str) -> Result<Self, String> {
        let wkt = wkt.trim();
        
        if wkt.starts_with("POINT") {
            Self::parse_point_wkt(wkt)
        } else if wkt.starts_with("LINESTRING") {
            Self::parse_linestring_wkt(wkt)
        } else if wkt.starts_with("POLYGON") {
            Self::parse_polygon_wkt(wkt)
        } else {
            Err("不支持的 WKT 格式".to_string())
        }
    }

    fn parse_point_wkt(wkt: &str) -> Result<Self, String> {
        let re = Regex::new(r"POINT\s*\(\s*([-\d.]+)\s+([-\d.]+)\s*\)").unwrap();
        
        if let Some(caps) = re.captures(wkt) {
            let lon = caps.get(1).unwrap().as_str().parse::<f64>().unwrap();
            let lat = caps.get(2).unwrap().as_str().parse::<f64>().unwrap();
            return Ok(Geography::Point(GeographyValue {
                latitude: lat,
                longitude: lon,
            }));
        }
        
        Err("无效的 POINT WKT 格式".to_string())
    }

    fn parse_linestring_wkt(wkt: &str) -> Result<Self, String> {
        let re = Regex::new(r"LINESTRING\s*\(\s*(.*?)\s*\)").unwrap();
        
        if let Some(caps) = re.captures(wkt) {
            let coords_str = caps.get(1).unwrap().as_str();
            let coords: Result<Vec<Coordinate>, _> = coords_str
                .split(',')
                .map(|s| {
                    let parts: Vec<&str> = s.trim().split_whitespace().collect();
                    if parts.len() == 2 {
                        let x = parts[0].parse::<f64>().map_err(|_| "无效的坐标".to_string())?;
                        let y = parts[1].parse::<f64>().map_err(|_| "无效的坐标".to_string())?;
                        Ok(Coordinate::new(x, y))
                    } else {
                        Err("无效的坐标格式".to_string())
                    }
                })
                .collect();
            
            return coords.map(|coordinates| {
                Geography::LineString(LineString { coordinates })
            });
        }
        
        Err("无效的 LINESTRING WKT 格式".to_string())
    }

    fn parse_polygon_wkt(wkt: &str) -> Result<Self, String> {
        use regex::Regex;
        let re = Regex::new(r"POLYGON\s*\(\s*(.*?)\s*\)").unwrap();
        
        if let Some(caps) = re.captures(wkt) {
            let rings_str = caps.get(1).unwrap().as_str();
            let rings: Result<Vec<Vec<Coordinate>>, _> = rings_str
                .split("),(")
                .map(|s| {
                    let s = s.trim_start_matches('(').trim_end_matches(')');
                    let coords: Result<Vec<Coordinate>, _> = s
                        .split(',')
                        .map(|coord_str| {
                            let parts: Vec<&str> = coord_str.trim().split_whitespace().collect();
                            if parts.len() == 2 {
                                let x = parts[0].parse::<f64>().map_err(|_| "无效的坐标".to_string())?;
                                let y = parts[1].parse::<f64>().map_err(|_| "无效的坐标".to_string())?;
                                Ok(Coordinate::new(x, y))
                            } else {
                                Err("无效的坐标格式".to_string())
                            }
                        })
                        .collect();
                    coords
                })
                .collect();
            
            return rings.map(|rings| {
                Geography::Polygon(Polygon { rings })
            });
        }
        
        Err("无效的 POLYGON WKT 格式".to_string())
    }
}
