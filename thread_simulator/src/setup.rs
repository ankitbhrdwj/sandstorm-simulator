extern crate im;
extern crate rand;

use rand::Rng;
use im::HashMap;

pub fn setup_map() -> HashMap<u64, u64> {
    let capacity:usize = 250000000;
    let mut map = HashMap::new();
                                                           
    let mut rng = rand::thread_rng();

    while map.len() != capacity {
        map.insert(rng.gen_range(0, u64::max_value()), rng.gen_range(0, u64::max_value()));
        if map.len() % 1000000 == 0 {
            println!("Current size of map is: {}", map.len());
        }
    }

    return map;
}