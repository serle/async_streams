use super::AsyncStockSignal;

pub struct MaxPrice;

///
/// Find the maximum in a series of f64
///
impl AsyncStockSignal for MaxPrice {
    type SignalType = f64;
    async fn calculate(&self, series: &[f64]) -> Option<Self::SignalType> {
        if series.is_empty() {
            None
        } else {
            Some(series.iter().fold(f64::MIN, |acc, q| acc.max(*q)))
        }
    }
}
