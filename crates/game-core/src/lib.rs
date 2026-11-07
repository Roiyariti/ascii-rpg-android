//! Estado lógico y proyección elevada 2.5D. Sin dependencias de ventana o GPU.
use glam::{IVec2, Vec2, Vec3};
use serde::{Deserialize, Serialize};
use turn_system::{Action, TurnClock};
use world_generation::Terrain;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub player: IVec2,
    pub clock: TurnClock,
    pub seed: u32,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            player: IVec2::ZERO,
            clock: TurnClock::default(),
            seed: 42,
        }
    }
}

impl GameState {
    /// Rechaza movimientos que no correspondan a una celda cardinal sin consumir turno.
    pub fn apply(&mut self, action: Action) -> bool {
        if let Action::Move { dx, dy } = action {
            if dx.unsigned_abs().saturating_add(dy.unsigned_abs()) != 1 {
                return false;
            }
            let Some(x) = self.player.x.checked_add(dx) else {
                return false;
            };
            let Some(y) = self.player.y.checked_add(dy) else {
                return false;
            };
            self.player = IVec2::new(x, y);
        }
        self.clock.commit(action);
        true
    }

    pub fn player_position(&self) -> Vec3 {
        Vec3::new(
            self.player.x as f32,
            self.player.y as f32,
            Terrain::new(self.seed).height(self.player.x, self.player.y),
        )
    }
}

/// x/y definen el suelo; z es la elevación. La salida se expresa en píxeles.
pub fn project(position: Vec3) -> Vec2 {
    Vec2::new(
        (position.x - position.y) * 22.0,
        (position.x + position.y) * 11.0 - position.z * 10.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejected_move_preserves_position_and_clock() {
        let mut state = GameState::default();
        assert!(!state.apply(Action::Move { dx: 1, dy: 1 }));
        assert!(!state.apply(Action::Move {
            dx: i32::MIN,
            dy: 0
        }));
        assert_eq!(state.player, IVec2::ZERO);
        assert_eq!(state.clock.current(), 0);
        assert!(state.apply(Action::Move { dx: 1, dy: 0 }));
        assert_eq!(state.player, IVec2::X);
        assert_eq!(state.clock.current(), 1);
    }

    #[test]
    fn elevation_moves_up_without_changing_ground_axis() {
        let ground = project(Vec3::new(2.0, 1.0, 0.0));
        let raised = project(Vec3::new(2.0, 1.0, 2.0));
        assert_eq!(ground.x, raised.x);
        assert!(raised.y < ground.y);
    }
}
