FROM python:3.9

RUN pip install --upgrade pip
RUN pip install zmq protobuf

WORKDIR /resources
COPY backend.py .
COPY model.py .
COPY schemas schemas
