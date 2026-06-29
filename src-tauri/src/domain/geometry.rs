use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizedBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl NormalizedBounds {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Result<Self, AppError> {
        let values = [x, y, width, height];
        if values.iter().any(|value| !value.is_finite())
            || x < 0.0
            || y < 0.0
            || width <= 0.0
            || height <= 0.0
            || x + width > 1.0
            || y + height > 1.0
        {
            return Err(AppError::Validation(
                "La zone de la fenêtre est invalide.".to_owned(),
            ));
        }
        Ok(Self {
            x,
            y,
            width,
            height,
        })
    }

    #[must_use]
    pub fn to_pixels(self, work_area: WorkArea) -> PixelBounds {
        PixelBounds {
            x: work_area.x + (self.x * f64::from(work_area.width)).round() as i32,
            y: work_area.y + (self.y * f64::from(work_area.height)).round() as i32,
            width: (self.width * f64::from(work_area.width)).round().max(1.0) as i32,
            height: (self.height * f64::from(work_area.height)).round().max(1.0) as i32,
        }
        .clamp_to(work_area)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkArea {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PixelBounds {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl PixelBounds {
    #[must_use]
    pub fn clamp_to(self, area: WorkArea) -> Self {
        let width = self.width.clamp(1, area.width.max(1));
        let height = self.height.clamp(1, area.height.max(1));
        Self {
            x: self.x.clamp(area.x, area.x + area.width - width),
            y: self.y.clamp(area.y, area.y + area.height - height),
            width,
            height,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{NormalizedBounds, PixelBounds, WorkArea};

    #[test]
    fn converts_normalized_bounds_on_a_negative_monitor() {
        let bounds = NormalizedBounds::new(0.5, 0.0, 0.5, 1.0).expect("valid bounds");
        let area = WorkArea {
            x: -1920,
            y: -40,
            width: 1920,
            height: 1040,
        };

        assert_eq!(
            bounds.to_pixels(area),
            PixelBounds {
                x: -960,
                y: -40,
                width: 960,
                height: 1040
            }
        );
    }

    #[test]
    fn rejects_bounds_outside_the_monitor() {
        assert!(NormalizedBounds::new(0.8, 0.0, 0.5, 1.0).is_err());
    }

    #[test]
    fn clamps_pixels_to_the_work_area() {
        let area = WorkArea {
            x: 100,
            y: 50,
            width: 800,
            height: 600,
        };
        let bounds = PixelBounds {
            x: 0,
            y: 700,
            width: 1000,
            height: 200,
        };

        assert_eq!(
            bounds.clamp_to(area),
            PixelBounds {
                x: 100,
                y: 450,
                width: 800,
                height: 200
            }
        );
    }
}
