import coloredlogs,logging
import colorama
from threading import Thread
import os

logging.basicConfig(level=logging.DEBUG)
logger = logging.getLogger(__file__)
coloredlogs.install(level='DEBUG')
colorama.init()

BOLD = '\033[1m'

if __name__ == "__main__":
    logger.info(f"Setting up proxy FMU backend.")
    dispatcher_endpoint = os.environ["UNIFMU_DISPATCHER_ENDPOINT"]
    dispatcher_endpoint_port = os.environ["UNIFMU_DISPATCHER_ENDPOINT_PORT"]
    logger.info(f"Proxy dispatcher endpoint: {dispatcher_endpoint}.")
    logger.info(f"Proxy dispatcher endpoint port: {dispatcher_endpoint_port}.")
    logger.info(f"{colorama.Fore.YELLOW}Use this port to connect the remote (private) FMU model: {BOLD}{colorama.Back.GREEN}'{dispatcher_endpoint_port}'{colorama.Style.RESET_ALL}")
    print(f"{colorama.Fore.YELLOW}Use this port to connect the remote (private) FMU model: {BOLD}{colorama.Back.GREEN}'{dispatcher_endpoint_port}'{colorama.Style.RESET_ALL}")
