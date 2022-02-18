//Replacement for std::cmp::min/max
pub mod min_max {

    pub fn get_max_u64(value1: u64, value2: u64) -> u64 {
        if value1 > value2 {
            return value1;
        }
        return value2;
    }
    pub fn get_min_u64(value1: u64, value2: u64) -> u64 {
        if value1 < value2 {
            return value1;
        }
        return value2;
    }
    pub fn get_min_u128(value1: u128, value2: u128) -> u128 {
        if value1 < value2 {
            return value1;
        }
        return value2;
    }
    pub fn get_max_u128(value1: u128, value2: u128) -> u128 {
        if value1 > value2 {
            return value1;
        }
        return value2;
    }
}
