mod price_diff;
mod windowed_sma;
mod max_price;
mod min_price;

//--------------------------------------------------------------------------------------------------
pub use price_diff::PriceDifference;
pub use windowed_sma::WindowedSMA;
pub use max_price::MaxPrice;
pub use min_price::MinPrice;
//--------------------------------------------------------------------------------------------------

///
/// A trait to provide a common interface for all signal calculations.
///
pub trait AsyncStockSignal {

    ///
    /// The signal's data type.
    ///
    type SignalType;

    ///
    /// Calculate the signal on the provided series.
    ///
    /// # Returns
    ///
    /// The signal (using the provided type) or `None` on error/invalid data.
    ///
    async fn calculate(&self, series: &[f64]) -> Option<Self::SignalType>;
}

