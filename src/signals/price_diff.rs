use super::AsyncStockSignal;

pub struct PriceDifference {}

///
/// Calculates the absolute and relative difference between the beginning and ending of an f64 series. The relative difference is relative to the beginning.
///
/// # Returns
///
/// A tuple `(absolute, relative)` difference.
///
impl AsyncStockSignal for PriceDifference {
    type SignalType = (f64, f64);
    async fn calculate(&self, series: &[f64]) -> Option<Self::SignalType> {
        if !series.is_empty() {
            // unwrap is safe here even if first == last
            let (first, last) = (series.first().unwrap(), series.last().unwrap());
            let abs_diff = last - first;
            let first = if *first == 0.0 { 1.0 } else { *first };
            let rel_diff = abs_diff / first;
            Some((abs_diff, rel_diff))
        } else {
            None
        }
    }
}
