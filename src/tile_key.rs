use std::f64::consts::{E, PI};
use std::ops::Add;
use std::string::ToString;

use once_cell::sync::Lazy;

const DEFAULT_ZOOM: usize = 18;

static TILES_PER_EDGE: Lazy<Vec<u16>> = Lazy::new(|| vec![1, 1, 1, 40, 40, 80, 80, 320, 1000, 2000, 2000, 4000, 8000, 16000, 16000, 32000]);

fn get_tiles_per_edge(zoom: usize) -> f64 {
    TILES_PER_EDGE[zoom.min(15).max(3)].into()
}

fn lat2tile(latitude: f64, tiles_per_edge: f64) -> i64 {
    // return (int) Math.floor((1 - Math.log(Math.tan(lat * Math.PI / 180) + 1 / Math.cos(lat * Math.PI / 180)) / Math.PI) / 2 * tilesPerEdge);
    ((1_f64 - ((latitude * PI / 180_f64).tan() + 1_f64 / (latitude * PI / 180_f64).cos()).log(E) / PI) / 2_f64 * tiles_per_edge).floor() as i64
}

fn lng2tile(longitude: f64, tiles_per_edge: f64) -> i64 {
    // return (int) Math.floor((lng + 180) / 360d * tilesPerEdge);
    ((longitude + 180_f64) / 360_f64 * tiles_per_edge).floor() as i64
}

// fn tile2lat(y: i64, tiles_per_edge: f64) -> f64 {
//     // double n = Math.PI - 2 * Math.PI * y / tilesPerEdge;
//     // return 180 / Math.PI * Math.atan(0.5d * (Math.exp(n) - Math.exp(-n)));
//     let n = PI - 2_f64 * PI * (y as f64) / tiles_per_edge;
//     180_f64 / PI * (0.5_f64 * (n.exp() - (-n).exp())).atan()
// }

// fn tile2lng(x: i64, tiles_per_edge: f64) -> f64 {
//     // return x / tilesPerEdge * 360 - 180;
//     (x as f64) / tiles_per_edge * 360_f64 - 180_f64
// }

#[derive(Clone, Copy, Debug)]
pub struct TileKey {
    zoom: usize,
    x: i64,
    y: i64,
    min_level: u8,
    max_level: u8,
    health: u8
}

impl TileKey {
    pub fn new(latitude: f64, longitude: f64) -> Self {
        let tiles_per_edge = get_tiles_per_edge(DEFAULT_ZOOM);

        TileKey {
            zoom: DEFAULT_ZOOM,
		    x: lng2tile(longitude, tiles_per_edge),
		    y: lat2tile(latitude, tiles_per_edge),
		    min_level: 0,
		    max_level: 8,
		    health: 100,
        }
    }
}

impl Add<(i64, i64)> for TileKey {
    type Output = Self;

    fn add(self, other: (i64, i64)) -> Self {
        let mut temp = self.clone();
        temp.x += other.0;
        temp.y += other.1;
        temp
    }
}

impl ToString for TileKey {
    fn to_string(&self) -> String {
        format!("{}_{}_{}_{}_{}_{}", self.zoom, self.x, self.y, self.min_level, self.max_level, self.health)
    }
}
