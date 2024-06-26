import json
import os.path
import os
import sys
import subprocess
import re
import platform
import shutil
import argparse
import site

from pathlib import Path
from time import sleep
from typing import Callable, ContextManager, List, NoReturn, Optional, Tuple
from datetime import datetime, timedelta
from urllib.request import Request, urlopen
from urllib.parse import urlencode

PIPX_VERSION = "1.3.3"
MIN_PYTHON_VERSION = (3, 10)
MIN_PIP_VERSION = (22, 0)
PACKAGE_FEED_PATH = 'DigitecGalaxus/_packaging/Devinite/pypi/simple/'
PYTHON_CANDIDATES = ["python3.12", "python3.11", "python3.10", "python3", "python"]

DG_CLI_USER_TYPE = os.getenv("DG_CLI_USER_TYPE", "user")
IS_INTERACTIVE_SESSION = DG_CLI_USER_TYPE == "user"
IS_DG_CLI_INTEGRATION_TEST = DG_CLI_USER_TYPE == "dg-cli-integration-test"

__system = platform.system()
IS_WINDOWS = __system == 'Windows'
IS_MACOS = __system == 'Darwin'
IS_DEVCONTAINER = os.getenv("IS_DEVCONTAINER", "false") == "true"

def main():
    autonomous_pat = os.getenv("DG_CLI_FEED_AUTHENTICATION_TOKEN", None)
    azure_devops_pat = None
    authenticated_user = None

    assert_valid_python_setup()

    if autonomous_pat is None and not IS_DG_CLI_INTEGRATION_TEST:
        if not IS_INTERACTIVE_SESSION:
            fail_installation(
                "Cannot install DG CLI in a non interactive environment without a PAT token"
                "Please make sure to set the DG_CLI_FEED_AUTHENTICATION_TOKEN environment variable"
            )

        action_info("In order to download the DG CLI from the package feed, we you need to authenticate with Azure DevOps")

        authenticated_user = authenticate_user()
        azure_devops_pat = acquire_pat_token(authenticated_user)
    else:
        action_info("Using automation PAT token in ENV")
        azure_devops_pat = autonomous_pat

    install_dg_cli_core(azure_devops_pat)

    if authenticated_user is not None:
        initialize_cli(authenticated_user, str(azure_devops_pat))

    action_info(
        'Installation complete. The new CLI command "dg" is ready to roll soon 🎉.\n'
        'To finalize the installation, close and re-spawn (or just reload) all terminal/shell instances '
        'to have everything properly initialized.'
    )


# region Generic utils

def run_all(*actions: Callable[[], None]) -> None:
    """Calls all actions in order. Allows for multiple statements in a lambda."""
    for action in actions:
        action()


def fail_installation(message: str, *additional_info: str) -> NoReturn:
    action_error(message)

    for info in additional_info:
        print(info)

    print(color("Installation failed", TEXT_RED + TEXT_BOLD))
    print("See stacktrace below")
    raise Exception()


def run_process(
    args: List[Path | str],
    failure_message: str | None = None,
    on_failure: Callable[[str, str], None] | None = None,
    require_success: bool = True,
    **kwargs) -> subprocess.CompletedProcess:
    try:
        process_result = subprocess.run(
            args,
            text=True,
            encoding='UTF-8',
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            **kwargs
        )
    except Exception as e:
        fail_installation(str(e))

    if process_result.returncode != 0:
        if on_failure is not None:
            on_failure(process_result.stdout, process_result.stderr)
            return process_result

        if failure_message is not None and require_success:
            fail_installation(
                color(failure_message, TEXT_RED + TEXT_BOLD),
                color("Running a vital command failed", TEXT_RED + TEXT_BOLD),
                "Aborting installation.",
                color("Command: ", TEXT_BOLD) + " ".join(map(lambda x: str(x), args)),
                color("Return code: ", TEXT_BOLD) + str(process_result.returncode),
                "Check the output below for more information.",
                color("STDOUT:", TEXT_BOLD),
                process_result.stdout,
                color("STDERR:", TEXT_BOLD),
                process_result.stderr
            )

        if failure_message is not None:
            action_warning(failure_message)
            print(action("Running a non-vital command failed - continuing with the installation. Please check the output below for more information"))
            print(action("Command: " + " ".join(map(lambda x: str(x), args))))
            print(action("Return code: " + str(process_result.returncode)))
            print(action("Check the output below for more information."))
            print(action("STDOUT:"))
            print(action(process_result.stdout))
            print(action("STDERR:"))
            print(action(process_result.stderr))

    return process_result


# endregion

# region Printing utils

TEXT_RED = "\033[91m"
TEXT_GREEN = "\033[92m"
TEXT_YELLOW = "\033[93m"
TEXT_BLUE = "\033[94m"
TEXT_PURPLE = "\033[95m"
TEXT_CYAN = "\033[96m"

BACKGROUND_RED = "\033[41m"
BACKGROUND_GREEN = "\033[42m"
BACKGROUND_YELLOW = "\033[43m"
BACKGROUND_BLUE = "\033[44m"
BACKGROUND_PURPLE = "\033[45m"
BACKGROUND_CYAN = "\033[46m"

TEXT_BOLD = "\033[1m"
TEXT_UNDERLINE = "\033[4m"


def action_error(text: str):
    print(action(text, TEXT_RED + TEXT_BOLD))


def color(text: str, color_str: Optional[str] = None):
    if color_str is None:
        return text
    return color_str + text + '\033[0m'


__action_depth = 0


def action_indent():
    global __action_depth
    if __action_depth == 0:
        __action_depth = 1
        return

    __action_depth += 2


def action_unindent():
    global __action_depth
    if __action_depth == 0:
        return

    if __action_depth == 1:
        __action_depth = 0
        return

    __action_depth -= 2


def indented_action(text: str, by: int = 1):
    global __action_depth
    return f"{__action_depth * by * ' '}{text}"


def action(text: str, color_str: Optional[str] = None):
    text = text.replace('\n', '\n' + indented_action(''))
    return color(indented_action(text), color_str)


def action_start(text: str) -> ContextManager:
    global __action_depth
    prefix = ""
    if __action_depth != 0:
        prefix = "➡️ "

    action_text = action(f"{prefix}{text} ...", TEXT_BOLD + TEXT_BLUE)
    return ActionContextManager(action_text)


class ActionContextManager(ContextManager):
    def __init__(self, text: str):
        self.text = text

    def __enter__(self):
        print(self.text)
        action_indent()

    def __exit__(self, exc_type, exc_value, traceback):
        action_end()


def action_info(text: str):
    print(action(f"ℹ️ {text}", TEXT_CYAN))


def action_success(text: str):
    global __action_depth
    print(action(f"✅ {text}", TEXT_GREEN))


def action_warning(text: str):
    global __action_depth
    print(action(f"⚠️ {text}", TEXT_YELLOW))


def action_end():
    action_unindent()


# endregion

# region User Authentication

DG_CLI_FEED_PAT_TOKEN_NAME = "DG CLI Devinite Feed"

DEVOPS_USER_IMPERSONATION_SCOPE = "499b84ac-1321-427f-aa17-267ca6975798/.default"
OFFLINE_ACCESS_SCOPE = "offline_access"
USER_AUTHENTICATION_SCOPES = f"{DEVOPS_USER_IMPERSONATION_SCOPE} {OFFLINE_ACCESS_SCOPE}"
DEVICE_CODE_GRANT_TYPE = "urn:ietf:params:oauth:grant-type:device_code"

DEVOPS_API_TOKEN_ENDPOINT = "https://vssps.dev.azure.com/DigitecGalaxus/_apis/Tokens/Pats?api-version=7.0-preview.1"

TOKEN_APP_CLIENT_ID = "6944457e-ec0c-491c-8f73-1d45f229f40e"
TOKEN_APP_TENANT_ID = "35aa8c5b-ac0a-4b15-9788-ff6dfa22901f"
TOKEN_APP_DEVICE_CODE_URI = f"https://login.microsoftonline.com/{TOKEN_APP_TENANT_ID}/oauth2/v2.0/devicecode"
TOKEN_APP_TOKEN_URI = f"https://login.microsoftonline.com/{TOKEN_APP_TENANT_ID}/oauth2/v2.0/token"

HTTPS_CONNECTION_FAILED_ERROR_MESSAGE = "An error ocurred while trying to establish a HTTPs connection." + \
                                        "Some Python installations do not link to the system's CA certificates. " + \
                                        "Please check our documentation here https://backstage.devinite.com/docs/default/System/Dg.Cli/#supportedtested-platforms " + \
                                        "for more information."

NINE_MONTHS_IN_WEEKS = 40


class AuthenticatedUser:
    def __init__(self, access_token: str, refresh_token: str):
        self.access_token = access_token
        self.refresh_token = refresh_token


class TokenPair:
    def __init__(self, access_token: str, refresh_token: str):
        self.access_token = access_token
        self.refresh_token = refresh_token


def authenticate_user() -> AuthenticatedUser:
    device_code, retry_interval = initiate_device_code_auth()
    tokens = acquire_token_with_device_code(device_code, retry_interval)

    action_info("Successfully authenticated.")
    action_info("Acquiring PAT token for package feed.")

    return AuthenticatedUser(tokens.access_token, tokens.refresh_token)


def initiate_device_code_auth() -> Tuple[str, int]:
    data = urlencode({
        'client_id': TOKEN_APP_CLIENT_ID,
        'scope': USER_AUTHENTICATION_SCOPES
    }).encode("utf-8")

    response = None
    try:
        with urlopen(Request(TOKEN_APP_DEVICE_CODE_URI, method="POST", data=data)) as res:
            response = res.read()

    except Exception as e:
        print(color(HTTPS_CONNECTION_FAILED_ERROR_MESSAGE), TEXT_RED)
        fail_installation(str(e))

    response_data = json.loads(response.decode('utf-8'))
    backup_message = f"Please authenticate at {response_data.get('verification_uri')} using the following code: {response_data.get('user_code')}"
    print(color(response_data.get('message', backup_message), TEXT_YELLOW))

    return response_data["device_code"], response_data["interval"]


def acquire_token_with_device_code(device_code: str, retry_interval: int) -> TokenPair:
    tokens = None
    while not tokens:
        sleep(retry_interval)
        print("Waiting for the user to authenticate...")
        tokens = try_acquire_token_with_device_code(device_code)

    return tokens


def try_acquire_token_with_device_code(device_code: str) -> Optional[TokenPair]:
    data = urlencode({
        'grant_type': DEVICE_CODE_GRANT_TYPE,
        'client_id': TOKEN_APP_CLIENT_ID,
        'device_code': device_code,
        'scope': USER_AUTHENTICATION_SCOPES
    }).encode("utf-8")

    try:
        with urlopen(Request(TOKEN_APP_TOKEN_URI, method="POST", data=data)) as res:
            if res.status != 200:
                return None

            token_response = res.read()
            tokens = json.loads(token_response.decode('utf-8'))
            return TokenPair(tokens["access_token"], tokens["refresh_token"])
    except Exception:
        return None


def acquire_pat_token(authenticated_user: AuthenticatedUser) -> str:
    current_date = datetime.now()
    token_expiration_date = current_date + timedelta(weeks=NINE_MONTHS_IN_WEEKS)
    token_expiration = token_expiration_date.strftime(r'%Y-%m-%dT%H:%M:%S.%fZ')

    data = json.dumps({
        "displayName": DG_CLI_FEED_PAT_TOKEN_NAME,
        "allOrgs": False,
        "scope": "vso.packaging",
        "validTo": token_expiration
    }).encode("utf-8")

    with urlopen(Request(DEVOPS_API_TOKEN_ENDPOINT, method="POST", data=data, headers={
        "Authorization": f"Bearer {authenticated_user.access_token}",
        "Content-Type": "application/json"
    })) as res:
        if res.status != 200:
            print(color(f"Failed to acquire PAT token for package feed. {res.status}", TEXT_RED))
            print(res.read().decode('utf-8'))
            fail_installation("Failed to acquire PAT token for package feed.")

        pat_response_data = json.loads(res.read().decode('utf-8'))["patToken"]
        print(color(f"Created PAT token \"{DG_CLI_FEED_PAT_TOKEN_NAME}\" for package feed ", TEXT_GREEN))

        return pat_response_data['token']


# endregion

# region Version checks

def assert_valid_python_setup() -> None:
    with action_start("Checking running Python version"):
        assert_valid_running_python_version()
        action_success("Running Python version compatible")

    with action_start("Checking required Python version"):
        assert_valid_python_version()
        action_success("Python version compatible")

    with action_start("Checking for required Pip version"):
        assert_required_pip_version_installed()
        action_success("Pip version compatible")


def assert_valid_running_python_version() -> None:
    current_version = sys.version_info[:2]
    if current_version < MIN_PYTHON_VERSION:
        fail_installation(
            f"DG CLI requires at least Python {MIN_PYTHON_VERSION[0]}.{MIN_PYTHON_VERSION[1]}. " +
            f"Currently installed version {current_version[0]}.{current_version[1]}"
        )


def assert_valid_python_version() -> None:
    result = run_python(
        ["--version"],
        require_success=True,
        failure_message=f"Failed to get Python version running `{find_python_executable()} --version`."
    )

    version_match = re.match(r"Python (\d+)\.(\d+)\.(\d+)", result.stdout)
    if not version_match:
        fail_installation("Failed to match Python version, please contact team Bender.")

    major, minor, patch = map(int, version_match.groups())
    if (major, minor) < MIN_PYTHON_VERSION:
        fail_installation(
            f"DG CLI requires at least Python {MIN_PYTHON_VERSION[0]}.{MIN_PYTHON_VERSION[1]}. " +
            f"Currently installed version {major}.{minor}"
        )

    print(f"Python version {major}.{minor}.{patch} detected")
    if minor == 12 and patch == 0:
        fail_installation(
            "We noticed a bug in Python 3.12.0, breaking the behavior of shutil.which in some configurations. " +
            "Please install the CLI using Python <= 3.11.x or >= 3.12.1"
        )


def assert_required_pip_version_installed() -> None:
    pip_version_output = run_python(
        ['-m', 'pip', '--version'],
        require_success=True,
        failure_message=f"Failed to get pip version running `{find_python_executable()} -m pip --version`." +
                        f"Make sure pip is installed."
    )

    regex_match = re.search(r"pip (\d+)\.(\d+)\.*", pip_version_output.stdout)
    if not regex_match:
        fail_installation("Failed to match pip version, please contact team Bender.")

    major, minor = map(int, regex_match.groups())
    if (major, minor) < MIN_PIP_VERSION:
        fail_installation(
            f"DG CLI requires at least Pip {MIN_PIP_VERSION[0]}.{MIN_PIP_VERSION[1]}." +
            f"Currently installed version {major}.{minor}"
        )
# endregion


# region CLI initialization
def initialize_cli(authenticated_user: AuthenticatedUser, azure_devops_pat: str):
    dg_cli_core_path = find_dg_cli_core_path()

    if dg_cli_core_path is None:
        action_info(
            "Could not find the DG CLI executable for initialization. " +
            "You will be prompted again in order to authenticate with your Azure DevOps account."
        )
        return

    with action_start("Initializing the CLI"):
        run_process([
            dg_cli_core_path,
            "init",
            "auth",
            "--access-token",
            authenticated_user.access_token,
            "--refresh-token",
            authenticated_user.refresh_token,
            "--packing-pat-token",
            azure_devops_pat
        ])
        action_success("Initialization complete")


def find_dg_cli_core_path() -> Path | None:
    shell_bin_path = shutil.which('dg')
    if shell_bin_path is not None:
        return Path(shell_bin_path)

    pipx_bin_dir: Path
    pipx_env_bin_dir = os.getenv("PIPX_BIN_DIR", None)
    if pipx_env_bin_dir is not None:
        pipx_bin_dir = Path(pipx_env_bin_dir)
    else:
        pipx_bin_dir = Path.home().joinpath(".local").joinpath("bin")

    pipx_bin_path = pipx_bin_dir.joinpath("dg")
    if IS_WINDOWS:
        pipx_bin_path = pipx_bin_path.with_suffix(".exe")

    if pipx_bin_path.exists():
        return pipx_bin_path

    return None
# endregion


# region Python executable utils
def run_python(args: List[str], **kwargs):
    python = find_python_executable()
    return run_process([python, *args], **kwargs)


__python_executable: str | None = None


def find_python_executable() -> str:
    global __python_executable

    if __python_executable is not None:
        return __python_executable

    for executable in PYTHON_CANDIDATES:
        try:
            # Do not use run_process here, since this would fail the installation
            output = subprocess.run(
                [executable, '--version'],
                text=True,
                encoding='UTF-8',
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
            )

            if output.returncode == 0:
                __python_executable = executable
                action_info(f'Found python executable: {executable}')
                return executable
        except Exception:
            continue

    fail_installation("Could not find a python version matching our requirements :(")


# endregion

# region PipX utils

def run_pipx(args: List[str], action_message: str, **kwargs):
    pipx_path = ensure_pipx_installed()

    with action_start(action_message):
        return run_process([pipx_path,*args], **kwargs)


__pipx_path: str | None = None


def ensure_pipx_installed() -> str:
    global __pipx_path
    if __pipx_path is not None:
        return __pipx_path

    pipx_path: str | None = None
    with action_start('Checking for existing "pipx" installation'):
        pipx_path = shutil.which('pipx')
        if pipx_path is not None:
            __pipx_path = pipx_path
            action_success(f"PipX is available at {pipx_path}")
            return pipx_path

        action_warning("PipX is not installed, installing it now")

    pipx_path = install_pipx()
    __pipx_path = pipx_path
    return pipx_path


def get_pipx_path() -> str:
    """
    Searches and returns the path of the pipx binary.
    This is a workaround to get access to the pipx binary before
    it has been made available through the PATH, which would require
    a shell restart when not installed by brew.
    """

    pipx_bin_path = None
    userbase_path = Path(site.getuserbase()).resolve()

    paths_to_test = (
        userbase_path.joinpath("bin").joinpath("pipx"),
        Path(site.getusersitepackages()).resolve().parent.joinpath("Scripts").joinpath("pipx.exe"),
    )
    for path_to_test in paths_to_test:
        if path_to_test.exists():
            pipx_bin_path = path_to_test
            break

    if pipx_bin_path is None:
        fail_installation(
            f"Could not find pipx binary in {paths_to_test[0].parent} or {paths_to_test[1].parent}",
            "We successfully installed pipx but failed to find the binary. Please restart your shell/terminal and try again."
        )

    return pipx_bin_path.as_posix()


def install_pipx() -> str:
    with action_start('Installing PipX (-> https://pypa.github.io/pipx/)'):
        is_brew_installed = shutil.which('brew') is not None
        if IS_MACOS and is_brew_installed:
            # Getting the pipx path reliably after installing it with brew is a lot easier
            # than using the get_pipx_user_bin_path() workaround, so we use that instead 
            install_pipx_with_brew()
        else:
            install_pipx_with_pip()
        action_success("PipX installed and made available on PATH (might require terminal/shell restart!)")

    with action_start("Making PipX available on PATH"):
        pipx_path = shutil.which('pipx')
        if IS_MACOS and is_brew_installed:
            pipx_path = str(shutil.which('pipx'))
        else:
            pipx_path = get_pipx_path()

        action_success(f"Found PipX at {pipx_path}")

        run_process(
            [pipx_path, 'ensurepath'],
            require_success=True,
            failure_message="Could not make 'pipx' available on PATH. Check the output of the executed 'brew' command and try running"
                            "'pipx ensurepath' manually. Run the install script again to continue."
        )
        action_success("PipX was made available in PATH (might require terminal/shell restart!)")

    return pipx_path


def install_pipx_with_brew() -> None:
    brew_install_env = os.environ.copy()
    brew_install_env['HOMEBREW_NO_AUTO_UPDATE'] = '1'

    run_process(
        ['brew', 'install', f'pipx@{PIPX_VERSION}'],
        env=brew_install_env,
        require_success=True,
        failure_message="Failed installing 'pipx'. Follow the installation instructions " +
                        "at 'https://pypa.github.io/pipx/installation/'. Rerun this script once 'pipx' is installed and " +
                        "available in the PATH"
    )


def install_pipx_with_pip() -> None:
    run_python(
        ['-m', 'pip', 'install', '--user', f'pipx=={PIPX_VERSION}'],
        require_success=True,
        failure_message="Failed installing 'pipx'. Follow the installation instructions " +
                        "at 'https://pypa.github.io/pipx/installation/'. Rerun this script once 'pipx' is installed and " +
                        "available in the PATH"
    )
# endregion


# region Install DG CLI
def install_dg_cli_core(pat_token: str | None):
    with action_start("Setting up DG CLI"):
        pipx_args = [
            "install",
            "dg-cli-core",
        ]

        index_url_param: List[str] = []
        if not IS_DG_CLI_INTEGRATION_TEST:
            index_url_param = [
                "--index-url",
                f"https://token:{pat_token}@pkgs.dev.azure.com/{PACKAGE_FEED_PATH}"
            ]

            pipx_args.extend(index_url_param)
        else:
            action_info("Using local index for DG CLI installation")

        with action_start("Installing dg-cli-core"):
            python_executable = find_python_executable()
            resolved_python_executable = shutil.which(python_executable)
            if resolved_python_executable is None:
                fail_installation(f"Could not find python executable {python_executable}")

            action_info(f"Using python executable at: \"{resolved_python_executable}\"")

            python_path = Path(resolved_python_executable)
            run_pipx(
                pipx_args,
                action_message="Running pipx install dg-cli-core",
                require_success=True,
                failure_message="Installation failed, something seems to be off :(",
                env={
                    **os.environ,
                    "PIPX_DEFAULT_PYTHON": python_path.as_posix(),
                }
            )
            action_success(f'DG CLI was successfully installed')

        if IS_WINDOWS:
            install_windows_dependencies(index_url_param)

        if IS_DEVCONTAINER:
            install_remote_dependencies(index_url_param)

def install_remote_dependencies(index_url_param: List[str]):
    with action_start(f"Adding a file keyring for devcontainer"):
        run_pipx(
            ["inject", "dg-cli-core", "dg-cli-plugin-keyringdevcontainer", *index_url_param],
            action_message="Installing dg-cli-plugin-keyringdevcontainer into dg-cli-core",
            failure_message="dg-cli-plugin-keyringdevcontainer install failed, check out the following logs"
        )
        action_success(f"dg-cli-plugin-keyringdevcontainer installed")

def install_windows_dependencies(index_url_param: List[str]):
    with action_start(f"Adding a required (virtual) dependency for accessing Docker on Windows"):
        run_pipx(
            ["inject", "dg-cli-core", "pypiwin32", *index_url_param],
            action_message="Installing pypiwin32 into dg-cli-core",
            failure_message="pypiwin32 install failed, check out the following logs"
        )
        action_success(f"pypiwin32 installed")


# endregion

if __name__ == "__main__":
    main()
