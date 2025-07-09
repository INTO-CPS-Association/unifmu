import logging
import os

from backend import Backend

logging.basicConfig(level=logging.DEBUG)
logger = logging.getLogger(__file__)

if __name__ == "__main__":

    dispatcher_endpoint = os.environ["UNIFMU_DISPATCHER_ENDPOINT"]
    logger.info(f"dispatcher endpoint received: {dispatcher_endpoint}")

    backend = Backend()
    backend.connect_to_endpoint(dispatcher_endpoint)
    backend.handshake()
    backend.command_reply_loop()