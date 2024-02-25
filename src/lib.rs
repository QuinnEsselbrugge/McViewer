
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub mod chunk_parser;
pub mod chunk_renderer;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
