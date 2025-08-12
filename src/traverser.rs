use rand::{Rng, SeedableRng};
use sha2::{Sha256, Digest};
use rand_chacha::ChaCha20Rng;

fn string_to_seed_32(s: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    let result = hasher.finalize();
    result.into()
}

pub struct Traverser {
    random: ChaCha20Rng,
    area: Vec<(usize, Vec<u8>)>,
    iteration: usize,
    dimensions: (u32, u32, Vec<u8>),
}

impl Traverser {

    pub fn new(dimensions: (u32, u32, Vec<u8>), key: Option<String>) -> Traverser {

        let key = match key {
            None => {""}
            Some(x) => &*{ x }
        };

        let seed = string_to_seed_32(key);

        let mut area = Vec::with_capacity(dimensions.0 as usize * dimensions.1 as usize);
        for i in 0..(dimensions.0 * dimensions.1) {
            area.push((i as usize, dimensions.2.clone()));
        }

        Traverser{
            area,
            random: ChaCha20Rng::from_seed(seed),
            iteration: 0,
            dimensions
        }

    }

    pub fn next(&mut self) -> Option<(u32, u32, u8)> {
        if self.area.is_empty() {
            return None;
        }

        let index = self.random.random_range(0..self.area.len());

        let color_vertex = self.area.get_mut(index).unwrap();

        let color;

        if color_vertex.1.len() == 1 {
            color = color_vertex.1.pop().unwrap();
        }
        else {
            let color_index = self.random.random_range(0..color_vertex.1.len());
            color = color_vertex.1.remove(color_index);
        }

        let value = color_vertex.0 as u32;

        if color_vertex.1.len() == 0 {
            self.area.remove(index);
        }

        self.iteration += 1;
        Some((value % self.dimensions.0, value / self.dimensions.0, color))
    }
}