use crate::domain::{
    geometry::PixelBounds,
    ports::{NativeError, WindowController},
    window::{NativeWindowHandle, WindowState},
};

pub fn apply_window_placement(
    controller: &impl WindowController,
    handle: NativeWindowHandle,
    bounds: PixelBounds,
    final_state: WindowState,
) -> Result<(), NativeError> {
    controller.place_window(handle, bounds)?;
    controller.set_window_state(handle, final_state)
}

#[cfg(test)]
mod tests {
    use super::apply_window_placement;
    use crate::domain::{
        geometry::PixelBounds,
        ports::fakes::FakeWindowSystem,
        window::{NativeWindowHandle, WindowState},
    };

    #[test]
    fn places_before_applying_the_final_state() {
        let system = FakeWindowSystem::default();
        let handle = NativeWindowHandle(42);
        let bounds = PixelBounds {
            x: -960,
            y: 0,
            width: 960,
            height: 1040,
        };

        apply_window_placement(&system, handle, bounds, WindowState::Maximized)
            .expect("placement succeeds");

        assert_eq!(
            *system.placements.lock().expect("placements"),
            vec![(handle, bounds)]
        );
        assert_eq!(
            *system.states.lock().expect("states"),
            vec![(handle, WindowState::Maximized)]
        );
    }

    #[test]
    fn supports_every_final_state() {
        for state in [
            WindowState::Normal,
            WindowState::Maximized,
            WindowState::Minimized,
        ] {
            let system = FakeWindowSystem::default();
            apply_window_placement(
                &system,
                NativeWindowHandle(1),
                PixelBounds {
                    x: 0,
                    y: 0,
                    width: 800,
                    height: 600,
                },
                state,
            )
            .expect("state succeeds");
            assert_eq!(system.states.lock().expect("states")[0].1, state);
        }
    }
}
