import socket
from struct import *
import threading

class PyPLCException(Exception):
    ...

class CommError(PyPLCException):
    ...

from .socket_ import SetupSocket, CPSocket
from .plc import PLC
from .consumer import Consumer, ConsumerHint
# from .producer import Producer

class Connection(object):
    def __init__(self):
        self.setupSocket = SetupSocket()
        self.CPSocket = CPSocket()
        self.PLCs = []

        self.SequenceCount = 0
    
    def addPLC(self, ip, shot=0):
        plc = PLC(self, ip, shot)
        plc.register()

        self.PLCs.append(plc)
        return plc
    
    def Start(self, join=False):
        self.CPSocket.bind()

        # Start thread
        self.thread = threading.Thread(target=self.connection_thread)
        self.thread.start()

        if join:
            self.thread.join()
    
    def Join(self):
        self.thread.join()

    def connection_thread(self):
        while True:
            try:
                data = self.CPSocket.receive(1024)
                print(data)
                # TODO: Send to consumer instances

            except KeyboardInterrupt:
                break
            except Exception as e:
                print(e)

    def Close(self):
        self.setupSocket.close()
        self.CPSocket.close()