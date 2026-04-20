//! Geospatial Types Module
//!
//! This module defines the types of geographic spatial points and the related operations.

use oxicode::{Decode, Encode};
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Geographic Information Representation
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
    /// Calculate the Haversine distance between two points (unit: kilometers)
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

    /// Calculate the azimuth angle between two points (unit: degrees)
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

    /// Check whether the checkpoint is within the specified rectangular area.
    pub fn in_bbox(&self, min_lat: f64, max_lat: f64, min_lon: f64, max_lon: f64) -> bool {
        self.latitude >= min_lat
            && self.latitude <= max_lat
            && self.longitude >= min_lon
            && self.longitude <= max_lon
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

impl GeographyValue {
    /// Estimate the memory usage of the geography value
    pub fn estimated_size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}

/// Geographic type (only point types are supported)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Encode, Decode)]
pub enum Geography {
    Point(GeographyValue),
}

impl Geography {
    /// Parse geographic data from the WKT format (only POINT types are supported)
    pub fn from_wkt(wkt: &str) -> Result<Self, String> {
        let wkt = wkt.trim();

        if wkt.starts_with("POINT") {
            Self::parse_point_wkt(wkt)
        } else {
            Err("不支持的 WKT 格式，目前只支持 POINT".to_string())
        }
    }

    fn parse_point_wkt(wkt: &str) -> Result<Self, String> {
        let re = Regex::new(r"POINT\s*\(\s*([-\d.]+)\s+([-\d.]+)\s*\)")
            .map_err(|_| "无效的正则表达式".to_string())?;

        if let Some(caps) = re.captures(wkt) {
            let lon = caps
                .get(1)
                .ok_or("缺少经度坐标")?
                .as_str()
                .parse::<f64>()
                .map_err(|_| "无效的经度格式")?;
            let lat = caps
                .get(2)
                .ok_or("缺少纬度坐标")?
                .as_str()
                .parse::<f64>()
                .map_err(|_| "无效的纬度格式")?;
            return Ok(Geography::Point(GeographyValue {
                latitude: lat,
                longitude: lon,
            }));
        }

        Err("无效的 POINT WKT 格式".to_string())
    }
}
