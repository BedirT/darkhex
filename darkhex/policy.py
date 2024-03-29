import typing
import darkhex.utils.util as util
import pyspiel
import os
import darkhex.check as CHECK
from darkhex import logger as log


class Policy:

    def __init__(self,
                 policy,
                 board_size: typing.Tuple[int, int],
                 initial_state: pyspiel.State,
                 is_perfect_recall: bool = False,
                 is_best_response: bool = False) -> None:
        """
        Initialize the policy.
        
        Args:
            policy: The policy.
            board_size (typing.Tuple[int, int]): The board size.
            initial_state (pyspiel.State): The initial state.
            is_perfect_recall (bool): Whether the policy is perfect recall.
            is_best_response (bool): Whether the policy is best response.
        """
        if isinstance(policy, str):
            # setup all the parameters using the policy data
            self._load_policy(policy, is_best_response)
            CHECK.EQUAL_OR_N(self.board_size, board_size)
            # Todo: CHECK.EQUAL_OR_N(self.initial_state, initial_state)
        else:
            assert initial_state is not None, "Initial state must be provided"
            assert board_size is not None, "Board size must be provided"
            self.initial_state = initial_state
            # Todo: CHECK.INFO_STATE(self.initial_state)
            # Todo: CHECK.BOARD_SIZE(board_size, initial_state)
            self.board_size = board_size
            self.policy = policy
            self.is_perfect_recall = is_perfect_recall
            self.is_best_response = is_best_response
        self.num_rows = self.board_size[0]
        self.num_cols = self.board_size[1]
        self.num_cells = self.num_rows * self.num_cols

    def get_action_probabilities(self,
                                 info_state: str) -> typing.Dict[int, float]:
        """
        Get the action probability dictionary for the given state.
        Args:
            info_state: The info state.
        
        Returns:
            The action probability dictionary.
        """
        raise NotImplementedError

    def get_action(self, info_state: str) -> int:
        """
        Take an action for the given state.
        Args:
            info_state: The info state.
        
        Returns:
            The action.
        """
        a_p = self.get_action_probabilities(info_state)
        return max(a_p, key=a_p.get)

    def _load_policy(self, policy_name: str, is_best_response: bool) -> None:
        """
        Load the Policy data from the file.
        Args:
            policy_name (str): The policy_name name/folder path.
        """
        if policy_name not in os.listdir(util.PathVars.policies):
            path = policy_name
        else:
            if is_best_response:
                path = util.PathVars.policies + policy_name + "/best_response.pkl"
            else:
                path = util.PathVars.policies + policy_name + "/policy.pkl"
        data = util.load_file(path)
        log.debug(f"Loaded data from path: {path} | {data}")
        self.policy = data.policy
        self.initial_state = data.initial_state
        self.board_size = data.board_size
        self.is_perfect_recall = data.is_perfect_recall
        self.is_best_response = is_best_response
        if data.player in [0, 1]:
            self.player = data.player

    def save_policy_to_file(self,
                            policy_name: str,
                            is_best_response: bool = False) -> None:
        """
        Save the policy as a pickle to the file.

        Args:
            policy_name (str): The policy name.
            is_best_response (bool): Whether the policy is best response.
        """
        data = util.dotdict(
            policy=self.policy,
            initial_state=self.initial_state,
            board_size=self.board_size,
            player=self.player if hasattr(self, "player") else None,
            is_perfect_recall=self.is_perfect_recall,
            is_best_response=is_best_response)
        if policy_name.find("/") != -1 and policy_name.find(
                ".") != -1:  # policy_name is a path
            path = policy_name
        else:
            if is_best_response:
                path = util.PathVars.policies + policy_name + "/best_response.pkl"
            else:
                path = util.PathVars.policies + policy_name + "/policy.pkl"
        util.save_file(data, path)
        log.info("Saved policy to path: " + path)


class TabularPolicy(Policy):

    def __init__(
        self,
        policy,
        board_size: typing.Tuple[int] = None,
        initial_state: pyspiel.State = None,
        is_perfect_recall: bool = False,
        is_best_response: bool = False,
    ):
        """
        Setup a tabular policy. Any two player policy that has a tabular representation can be used.

        Args:
            policy (str or dict[str, dict[int, float]]): The policy name or a dictionary of action probability dictionary.
            board_size (list): The size of the board.
            initial_state (pyspiel.State): The initial state of the board.
            is_perfect_recall (bool): Whether the policy is perfect recall.
            is_best_response (bool): Whether the policy is best response.
        """
        super().__init__(policy, board_size, initial_state, is_perfect_recall,
                         is_best_response)

    def get_action_probabilities(self,
                                 info_state: str) -> typing.Dict[int, float]:
        """
        Get the action probability dictionary for the given state.
        Args:
            info_state: The info state.
        
        Returns:
            The action probability dictionary.
        """
        return self.policy[info_state]


class SinglePlayerTabularPolicy(TabularPolicy):

    def __init__(
        self,
        policy,
        board_size: typing.Tuple[int] = None,
        initial_state: pyspiel.State = None,
        player: int = None,
        is_perfect_recall: bool = False,
        is_best_response: bool = False,
    ):
        """
        Setup a single player tabular policy. Any single player policy that has a tabular representation can be used.

        Args:
            policy (str or dict[str, dict[int, float]]): The policy name or a dictionary of action probability dictionary.
            board_size (list): The size of the board.
            initial_state (pyspiel.State): The initial state of the board.
            player (int): The player the policy belongs to.
            is_perfect_recall (bool): Whether the policy is perfect recall.
            is_best_response (bool): Whether the policy is a best response policy.
        """
        super().__init__(policy, board_size, initial_state, is_perfect_recall,
                         is_best_response)
        if not hasattr(self, 'player'):
            self.player = player
        CHECK.PLAYER(self.player)
        self.opponent = 1 - self.player

    def get_action_probabilities(self,
                                 info_state: str) -> typing.Dict[int, float]:
        """
        Get the action probability dictionary for the given state.
        Args:
            info_state: The info state.
        
        Returns:
            The action probability dictionary.
        """
        # todo:
        # CHECK.STATE_PLAYER(info_state, self.player)
        return self.policy[info_state]


class PyspielSolverPolicy(Policy):

    def __init__(self,
                 solver=None,
                 path=None,
                 board_size: typing.Tuple[int] = None,
                 initial_state: pyspiel.State = None,
                 is_perfect_recall: bool = False):
        """
        Setup a pyspiel policy that uses a solver. A policy file that has a type where average
        policy can be accessed using a solver can be used.

        Args:
            solver (pyspiel.OutcomeSamplingMCCFRSolver or pyspiel.ExternalSamplingMCCFRSolver): 
                A pyspiel solver object.
            path (str): The path to the policy file. Cannot be used with solver.
            board_size (list): The size of the board.
            initial_state (pyspiel.State): The initial state of the board.
            is_perfect_recall (bool): Whether the policy is perfect recall.
        """
        if (solver is None and path is None) or (solver is not None and
                                                 path is not None):
            raise ValueError("Either solver or path must be provided.")
        if solver:
            self.solver = solver
            super().__init__(solver.average_policy(), board_size, initial_state)
        else:
            self._load_policy(path)

    def get_action_probabilities(
            self, pyspiel_state: pyspiel.State) -> typing.Dict[int, float]:
        """
        Get the action probability dictionary for the given pyspiel state.

        Args:
            pyspiel_state: The pyspiel state.

        Returns:
            The action probability dictionary.
        """
        return self.policy.action_probabilities(pyspiel_state)

    def get_action(self, pyspiel_state: pyspiel.State) -> int:
        """
        Get the action for the given pyspiel state.

        Args:
            pyspiel_state: The pyspiel state.

        Returns:
            (int) The action.
        """
        action_probs = self.get_action_probabilities(pyspiel_state)
        return max(action_probs, key=action_probs.get)

    def _load_policy(self, policy_path: str) -> None:
        """
        Load the Policy data from the file.
        Args:
            policy_path (str): The policy path/folder path.
        """
        if policy_path not in os.listdir(util.PathVars.policies):
            path = policy_path
        else:
            path = util.PathVars.policies + policy_path + "/policy.pkl"
        data = util.load_file(path)
        self.solver = data.solver
        self.initial_state = data.initial_state
        self.board_size = data.board_size
        self.policy = self.solver.average_policy()
        self.num_rows = self.board_size[0]
        self.num_cols = self.board_size[1]
        self.num_cells = self.num_rows * self.num_cols

    def save_policy_to_file(self, policy_name: str) -> None:
        """
        Save the policy as a pickle to the file.

        Args:
            policy_name (str): The policy name.
        """
        data = util.dotdict(
            solver=self.solver,
            initial_state=self.initial_state,
            board_size=self.board_size,
        )
        if policy_name.find("/") != -1 and policy_name.find(
                ".") != -1:  # policy_name is a path
            path = policy_name
        else:
            path = util.PathVars.policies + policy_name + "/policy.pkl"
        log.debug(path)
        util.save_file(data, path)
        log.info("Saved policy to path: " + path)


def convert_pyspiel_policy_to_darkhex_policy():
    """
    Convert a pyspiel policy to a darkhex policy.
    """
    pass
