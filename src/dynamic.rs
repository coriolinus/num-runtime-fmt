/// Container for setting certain parameters dynamically.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Dynamic {
    pub width: Option<usize>,
    pub precision: Option<usize>,
    pub spacing: Option<usize>,
}

impl Dynamic {
    /// Construct a `Dynamic` instance, specifying all parameters.
    ///
    /// The parameters accept anything which can become an `Option<usize>`, so
    /// it's possible to use both bare numbers and `None`:
    ///
    /// ```rust
    /// # use num_runtime_fmt::Dynamic;
    /// Dynamic::new(5, 3, None);
    /// ```
    pub fn new<W, P, S>(width: W, precision: P, spacing: S) -> Dynamic
    where
        W: Into<Option<usize>>,
        P: Into<Option<usize>>,
        S: Into<Option<usize>>,
    {
        Dynamic {
            width: width.into(),
            precision: precision.into(),
            spacing: spacing.into(),
        }
    }

    /// Construct a `Dynamic` instance specifying only `width`.
    pub fn width(width: usize) -> Dynamic {
        Dynamic {
            width: Some(width),
            ..Dynamic::default()
        }
    }

    /// Construct a `Dynamic` instance specifying only `precision`.
    pub fn precision(precision: usize) -> Dynamic {
        Dynamic {
            precision: Some(precision),
            ..Dynamic::default()
        }
    }

    /// Construct a `Dynamic` instance specifying only `spacing`.
    pub fn spacing(spacing: usize) -> Dynamic {
        Dynamic {
            spacing: Some(spacing),
            ..Dynamic::default()
        }
    }
}
