import coloredlogs,logging
import colorama
import os
import sys
import toml

from blackbox_backend import BlackboxBackend

logging.basicConfig(level=logging.DEBUG)
logger = logging.getLogger(__file__)
coloredlogs.install(level='DEBUG')
colorama.init()
__location__ = os.path.realpath(os.path.join(os.getcwd(), os.path.dirname(__file__)))
BOLD = '\033[1m'

if __name__ == "__main__":
    
    input_ok = False

    if len(sys.argv) == 2:
        try:
            proxy_port = int(sys.argv[1])
            input_ok = True
        except:
            logger.error(f'Only one argument for the port in integer format is accepted.')
            sys.exit(-1)

    while not input_ok:
        port_str = input(f'{colorama.Back.GREEN}Input the port for remote proxy FMU:{colorama.Style.RESET_ALL}\n')
        try:
            proxy_port = int(port_str)
            input_ok = True
        except:
            logger.error(f'Only integers accepted.')

    with open(os.path.join(__location__,'endpoint.toml'), 'r') as f:
        endpoint_config = toml.load(f)
        proxy_ip_address = endpoint_config["ip"]
    dispatcher_endpoint =  str(proxy_ip_address) + ":" + str(proxy_port)
    logger.info(f"dispatcher endpoint received: {BOLD} {colorama.Back.GREEN} {dispatcher_endpoint} {colorama.Style.RESET_ALL}")

    backend = BlackboxBackend()

    backend.connect_to_endpoint("tcp://" + dispatcher_endpoint)
    logger.info(f"Socket connected successfully.")

    backend.handshake()
    backend.command_reply_loop()
