use std::f64::consts::PI;
use std::ops::Add;

const DEFAULT_ZOOM: u8 = 15;

static TILES_PER_EDGE: [u16; 16] = [1, 1, 1, 40, 40, 80, 80, 320, 1000, 2000, 2000, 4000, 8000, 16000, 16000, 32000];

fn get_tiles_per_edge(zoom: u8) -> f64 {
    TILES_PER_EDGE[zoom.min(15).max(3) as usize].into()
}

fn lat2tile(latitude: f64, tiles_per_edge: f64) -> i64 {
    // return (int) Math.floor((1 - Math.log(Math.tan(lat * Math.PI / 180) + 1 / Math.cos(lat * Math.PI / 180)) / Math.PI) / 2 * tilesPerEdge);
    ((1_f64 - ((latitude * PI / 180_f64).tan() + 1_f64 / (latitude * PI / 180_f64).cos()).ln() / PI) / 2_f64
        * tiles_per_edge)
        .floor() as i64
}

fn lng2tile(longitude: f64, tiles_per_edge: f64) -> i64 {
    // return (int) Math.floor((lng + 180) / 360d * tilesPerEdge);
    ((longitude + 180_f64) / 360_f64 * tiles_per_edge).floor() as i64
}

#[allow(dead_code)]
fn tile2lat(y: i64, tiles_per_edge: f64) -> f64 {
    // double n = Math.PI - 2 * Math.PI * y / tilesPerEdge;
    // return 180 / Math.PI * Math.atan(0.5d * (Math.exp(n) - Math.exp(-n)));
    let n = PI - 2_f64 * PI * (y as f64) / tiles_per_edge;
    180_f64 / PI * (0.5_f64 * (n.exp() - (-n).exp())).atan()
}

#[allow(dead_code)]
fn tile2lng(x: i64, tiles_per_edge: f64) -> f64 {
    // return x / tilesPerEdge * 360 - 180;
    (x as f64) / tiles_per_edge * 360_f64 - 180_f64
}

#[derive(Clone, Copy, Debug)]
pub struct TileKey {
    pub zoom: u8,
    pub x: i64,
    pub y: i64,
    pub min_level: u8,
    pub max_level: u8,
    pub health: u8,
}

impl TileKey {
    pub fn new(
        latitude: f64,
        longitude: f64,
        zoom: Option<u8>,
        min_level: Option<u8>,
        max_level: Option<u8>,
        health: Option<u8>,
    ) -> Self {
        let zoom = zoom.unwrap_or(DEFAULT_ZOOM);
        let tiles_per_edge = get_tiles_per_edge(zoom);

        TileKey {
            zoom,
            x: lng2tile(longitude, tiles_per_edge),
            y: lat2tile(latitude, tiles_per_edge),
            min_level: min_level.unwrap_or_default(),
            max_level: max_level.unwrap_or(8),
            health: health.unwrap_or(100),
        }
    }

    #[allow(dead_code)]
    pub fn range(
        (from_lat, from_lng): (f64, f64),
        (to_lat, to_lng): (f64, f64),
        zoom: Option<u8>,
        min_level: Option<u8>,
        max_level: Option<u8>,
        health: Option<u8>,
    ) -> Vec<Self> {
        let zoom = zoom.unwrap_or(DEFAULT_ZOOM);
        let tiles_per_edge = get_tiles_per_edge(zoom);

        let x1 = lng2tile(from_lng, tiles_per_edge);
        let y1 = lat2tile(from_lat, tiles_per_edge);
        let x2 = lng2tile(to_lng, tiles_per_edge);
        let y2 = lat2tile(to_lat, tiles_per_edge);
        let from_x = x1.min(x2);
        let from_y = y1.min(y2);
        let to_x = x1.max(x2);
        let to_y = y1.max(y2);

        (from_x..=to_x)
            .into_iter()
            .flat_map(|x| {
                (from_y..=to_y).into_iter().map(move |y| TileKey {
                    zoom,
                    x,
                    y,
                    min_level: min_level.unwrap_or_default(),
                    max_level: max_level.unwrap_or(8),
                    health: health.unwrap_or(100),
                })
            })
            .collect()
    }
}

impl Add<(i64, i64)> for TileKey {
    type Output = Self;

    fn add(mut self, other: (i64, i64)) -> Self {
        self.x += other.0;
        self.y += other.1;
        self
    }
}

impl std::fmt::Display for TileKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}_{}_{}_{}_{}_{}", self.zoom, self.x, self.y, self.min_level, self.max_level, self.health)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn tile_key() {
        let tk = super::TileKey::new(45.5636024140848, 12.431250000000006, None, None, None, None);
        assert_eq!(tk.x, 17105);
        assert_eq!(tk.y, 11440);

        let tiles_per_edge = super::get_tiles_per_edge(super::DEFAULT_ZOOM);
        assert_eq!(super::tile2lat(tk.y, tiles_per_edge), 45.5636024140848);
        assert_eq!(super::tile2lng(tk.x, tiles_per_edge), 12.431250000000006);
    }

    #[test]
    fn range() {
        let tiles_per_edge = super::get_tiles_per_edge(super::DEFAULT_ZOOM);
        let tks = super::TileKey::range(
            (45.362997, 12.060000000000002),
            (45.76016527904371, 12.939141),
            None,
            None,
            None,
            None,
        );
        assert!(!tks.is_empty());
        for tk in tks {
            let lat = super::tile2lat(tk.y, tiles_per_edge);
            let lng = super::tile2lng(tk.x, tiles_per_edge);
            if !((45.362997..=45.76016527904371).contains(&lat) && (12.060000000000002..=12.939141).contains(&lng)) {
                panic!("{lat},{lng}");
            }
        }
    }
}
