import pyspiel
import pickle as pkl

# todo: use file version if exists
# todo: make dumping to file optional

def get_all_information_states(game: pyspiel.Game, include_terminal_states=True) -> list:
    """Get all information states for the given game."""
    info_states = {}
    state_data = {}
    state = game.new_initial_state()
    _get_all_info_states(state, info_states, state_data, include_terminal_states)
    with open("darkhex/data/state_data2x2.pkl", "wb") as f:
        pkl.dump(state_data, f)
    return info_states

def _get_all_info_states(state: pyspiel.State, info_states: dict, state_data: dict,
                         include_terminal_states) -> None:
    """Calculate information states recursively for the state. Fill in the
    info_states."""
    r = -1
    if state.is_terminal():
        if include_terminal_states:
            r = 0 if state.returns()[0] > 0 else 1
        else:
            return
    info_tuple = (state.information_state_string(0),
                  state.information_state_string(1),)
    data = [(state.legal_actions(0), state.legal_actions(1)), r] 
                                    # if 0 its not a terminal state
    if info_tuple not in info_states:
        info_states[info_tuple] = data
        if not state.is_terminal():
            if state.information_state_string(state.current_player()) not in state_data:
                state_data[state.information_state_string(state.current_player())] = state
    else:
        return
    if state.is_terminal():
        return
    for action in state.legal_actions():
        new_state = state.child(action)
        _get_all_info_states(new_state, info_states, state_data, include_terminal_states)


if __name__ == "__main__":
    get_all_information_states(pyspiel.load_game("dark_hex_ir(num_rows=2,num_cols=2)"))
