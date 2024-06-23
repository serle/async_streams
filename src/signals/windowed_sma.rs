use super::AsyncStockSignal;

pub struct WindowedSMA(usize);

impl WindowedSMA {
    pub(crate) fn new(window_size: usize) -> Self {
        Self(window_size)
    }
}

///
/// Window function to create a simple moving average
///
impl AsyncStockSignal for WindowedSMA {
    type SignalType = Vec<f64>;
    async fn calculate(&self, series: &[f64]) -> Option<Self::SignalType> {
        if !series.is_empty() && self.0 > 1 {
            Some(
                series
                    .windows(self.0)
                    .map(|w| w.iter().sum::<f64>() / w.len() as f64)
                    .collect(),
            )
        } else {
            None
        }
    }
}
