
/// What is the character state.
/// This is meant to be used as a state machine.
enum CharacterState {
    Idle,
    CastingLeft {
        charged_time: f32,
    },
    CastingRight {
        charged_time: f32,
    },
    CastingBoth {
        left_charged_time: f32,
        right_charged_time: f32,
    },
    Teleporting {
        tp_timer: f32,
    },
}

/// The current state of the player: state machine, objects, mana...
pub(crate) struct PlayerState {
    right_hand: Option<()>, // todo : object
    left_hand: Option<()>, // todo : object
    mana: f32,
    state: CharacterState,
    grounded: bool,
}