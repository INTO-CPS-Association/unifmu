FROM python:3.8

RUN pip install zmq grpc

COPY modelDescription.xml modelDescription.xml

WORKDIR resources
COPY src .