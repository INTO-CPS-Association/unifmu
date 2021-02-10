import json
import logging
import xml.etree.ElementTree as ET
from argparse import ArgumentParser
from pathlib import Path

# from command_server import CommandServicer
from adder import Adder

from concurrent import futures
import logging
import grpc

from unifmu_fmi2_pb2_grpc import SendCommandServicer, add_SendCommandServicer_to_server
from unifmu_fmi2_pb2 import StatusReturn, GetRealReturn, GetIntegerReturn, GetBooleanReturn, GetStringReturn
from fmi2 import Fmi2FMU

class CommandServicer(SendCommandServicer):

    def __init__(self, fmu):
        super().__init__()
        self.fmu = fmu

    ##### REAL #####
    def Fmi2SetReal(self, request, context):
        status = self.fmu.set_xxx(request.references, request.values)
        return StatusReturn(status=status)

    def Fmi2GetReal(self, request, context):
        status, values = self.fmu.get_xxx(request.references)
        return GetRealReturn(status=status, values=values)


    ##### INTEGER #####
    def Fmi2SetInteger(self, request, context):
        status = self.fmu.set_xxx(request.references, request.values)
        return StatusReturn(status=status)

    def Fmi2GetInteger(self, request, context):
        status, values = self.fmu.get_xxx(request.references)
        return GetIntegerReturn(status=status, values=values)


    ##### BOOLEAN #####
    def Fmi2SetBoolean(self, request, context):
        status = self.fmu.set_xxx(request.references, request.values)
        return StatusReturn(status=status)

    def Fmi2GetBoolean(self, request, context):
        status, values = self.fmu.get_xxx(request.references)
        return GetBooleanReturn(status=status, values=values)

    ##### STRING #####
    def Fmi2SetString(self, request, context):
        status = self.fmu.set_xxx(request.references, request.values)
        return StatusReturn(status=status)

    def Fmi2GetString(self, request, context):
        status, values = self.fmu.get_xxx(request.references)
        return GetStringReturn(status=status, values=values)

    #### Do step ####
    def Fmi2DoStep(self, request, context):
        status = self.fmu.do_step(request.current_time, request.step_size, request.no_step_prior)
        return StatusReturn(status=status)

    ##### Set Debug Logging ####
    def Fmi2SetDebugLogging(self, request, context):
        status = self.fmu.set_debug_logging(request.categories, request.logging_on)
        return StatusReturn(status=status)



if __name__ == "__main__":

    reference_to_attr = {}
    path = Path.cwd().parent / "grpc_python/src/modelDescription.xml"
    with open(path) as f:
        for v in ET.parse(f).find("ModelVariables"):
            reference_to_attr[int(v.attrib["valueReference"])] = v.attrib["name"]

    slave = Adder(reference_to_attr)

    server = grpc.server(futures.ThreadPoolExecutor(max_workers=10))
    add_SendCommandServicer_to_server(CommandServicer(slave), server)
    server.add_insecure_port('[::]:50051') # TODO must connect to correct port (right now this is hardcoded)
    server.start()
    print("Started server")
    print("Waiting!")
    server.wait_for_termination()


