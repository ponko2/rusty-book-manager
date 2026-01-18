#[cfg(test)]
mod tests {
    use rstest::{fixture, rstest};

    #[fixture]
    fn 任意のフィクスチャ名() -> i32 {
        24
    }

    #[rstest]
    fn 任意の関数名(任意のフィクスチャ名: i32) {
        assert_eq!(任意のフィクスチャ名 * 2, 48);
    }
}
