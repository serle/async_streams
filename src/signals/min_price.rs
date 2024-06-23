use super::AsyncStockSignal;

pub struct MinPrice;


///
/// Find the minimum in a series of f64
///
impl AsyncStockSignal for MinPrice {
    type SignalType = f64;
    async fn calculate(&self, series: &[f64]) -> Option<Self::SignalType> {
        if series.is_empty() {
            None
        } else {
            Some(series.iter().fold(f64::MAX, |acc, q| acc.min(*q)))
        }
    }
}
