//! Las acciones confirmadas consumen un turno; el renderizado nunca avanza la simulación.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    Move { dx: i32, dy: i32 },
    Wait,
    Attack,
    SwitchWeapon,
    Equip,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TurnClock {
    turn: u64,
}

impl TurnClock {
    pub fn current(&self) -> u64 {
        self.turn
    }

    pub fn commit(&mut self, _action: Action) {
        self.turn = self.turn.saturating_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_confirmed_action_consumes_a_turn() {
        let mut clock = TurnClock::default();
        for (index, action) in [
            Action::Move { dx: 1, dy: 0 },
            Action::Wait,
            Action::Attack,
            Action::SwitchWeapon,
            Action::Equip,
        ]
        .into_iter()
        .enumerate()
        {
            clock.commit(action);
            assert_eq!(clock.current(), index as u64 + 1);
        }
    }
}
