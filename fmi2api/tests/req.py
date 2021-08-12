import zmq
import os

if __name__ == "__main__":

    context = zmq.Context()
    socket = context.socket(zmq.REQ)
    dispatcher_endpoint = os.environ["UNIFMU_DISPATCHER_ENDPOINT"]
    socket.connect(dispatcher_endpoint)

    socket.send_string("Hello World!")
