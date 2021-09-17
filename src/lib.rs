pub mod feed;
/// 府県天気予報（Ｒ１）
pub mod fuken_r1;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }// 気象庁のxmlデータを読むラッパをつくる
}
