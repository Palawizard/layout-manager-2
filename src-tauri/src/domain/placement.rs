use serde::{Deserialize, Serialize};

use super::{
    geometry::NormalizedBounds,
    layout::WindowPlacement,
    monitor::{MonitorFallback, MonitorId, MonitorSelector},
    window::WindowState,
};
use crate::error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlacementPreset {
    FullScreen,
    LeftHalf,
    RightHalf,
    TopHalf,
    BottomHalf,
    TopLeftQuarter,
    TopRightQuarter,
    BottomLeftQuarter,
    BottomRightQuarter,
    Custom,
}

impl PlacementPreset {
    #[must_use]
    pub const fn all_standard() -> &'static [Self] {
        &[
            Self::FullScreen,
            Self::LeftHalf,
            Self::RightHalf,
            Self::TopHalf,
            Self::BottomHalf,
            Self::TopLeftQuarter,
            Self::TopRightQuarter,
            Self::BottomLeftQuarter,
            Self::BottomRightQuarter,
        ]
    }

    #[must_use]
    pub fn to_bounds(self) -> NormalizedBounds {
        match self {
            Self::FullScreen => bounds(0.0, 0.0, 1.0, 1.0),
            Self::LeftHalf => bounds(0.0, 0.0, 0.5, 1.0),
            Self::RightHalf => bounds(0.5, 0.0, 0.5, 1.0),
            Self::TopHalf => bounds(0.0, 0.0, 1.0, 0.5),
            Self::BottomHalf => bounds(0.0, 0.5, 1.0, 0.5),
            Self::TopLeftQuarter => bounds(0.0, 0.0, 0.5, 0.5),
            Self::TopRightQuarter => bounds(0.5, 0.0, 0.5, 0.5),
            Self::BottomLeftQuarter => bounds(0.0, 0.5, 0.5, 0.5),
            Self::BottomRightQuarter => bounds(0.5, 0.5, 0.5, 0.5),
            Self::Custom => bounds(0.0, 0.0, 1.0, 1.0),
        }
    }

    pub fn detect(bounds: NormalizedBounds) -> Result<Self, AppError> {
        for preset in Self::all_standard() {
            if preset.to_bounds() == bounds {
                return Ok(*preset);
            }
        }
        Ok(Self::Custom)
    }
}

pub fn build_window_placement(
    monitor_id: MonitorId,
    fallback: MonitorFallback,
    preset: PlacementPreset,
    custom_bounds: Option<NormalizedBounds>,
    state: WindowState,
) -> Result<WindowPlacement, AppError> {
    let bounds = match preset {
        PlacementPreset::Custom => custom_bounds.ok_or_else(|| {
            AppError::Validation("La zone personnalisée est invalide.".to_owned())
        })?,
        preset => preset.to_bounds(),
    };
    NormalizedBounds::new(bounds.x, bounds.y, bounds.width, bounds.height)?;
    Ok(WindowPlacement {
        monitor_selector: MonitorSelector {
            preferred_id: monitor_id,
            fallback,
        },
        bounds,
        state,
    })
}

const fn bounds(x: f64, y: f64, width: f64, height: f64) -> NormalizedBounds {
    NormalizedBounds {
        x,
        y,
        width,
        height,
    }
}

#[cfg(test)]
mod tests {
    use super::{PlacementPreset, build_window_placement};
    use crate::domain::{
        geometry::NormalizedBounds,
        monitor::{MonitorFallback, MonitorId},
        window::WindowState,
    };

    #[test]
    fn converts_every_standard_preset() {
        for preset in PlacementPreset::all_standard() {
            let bounds = preset.to_bounds();
            assert!(NormalizedBounds::new(bounds.x, bounds.y, bounds.width, bounds.height).is_ok());
            assert_eq!(PlacementPreset::detect(bounds).expect("preset"), *preset);
        }
    }

    #[test]
    fn builds_a_custom_placement() {
        let custom = NormalizedBounds::new(0.1, 0.2, 0.3, 0.4).expect("custom");
        let placement = build_window_placement(
            MonitorId("display-1".to_owned()),
            MonitorFallback::Primary,
            PlacementPreset::Custom,
            Some(custom),
            WindowState::Maximized,
        )
        .expect("placement");
        assert_eq!(placement.bounds, custom);
        assert_eq!(placement.state, WindowState::Maximized);
    }
}
