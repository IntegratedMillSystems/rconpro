import socket
from struct import *
import threading

# Custom Exceptions
class PyConProException(Exception):
    ...

class CommError(PyConProException):
    ...

from .socket_ import SetupSocket, CPSocket
from .plc import PLC
from .consumer import Consumer, ConsumerHint
# from .producer import Producer

class Connection(object):
    """
    Manages sockets and serves Consumer instances data.
    Acts as a parent to PLC class instances (a connection
    can get data from multiple PLCs).
    """

    def __init__(self):
        """
        Initiates Connection class by setting up sockets
        and initiating other variables.
        """

        self.setupSocket = SetupSocket()
        self.CPSocket = CPSocket()
        self.PLCs = []

        self.SequenceCount = 0
    
    def addPLC(self, ip, shot=0):
        """
        Initiates a PLC instance, registers our socket with
        the actual PLC over the network, and saves it to a list.
        """

        plc = PLC(self, ip, shot)
        plc.register()

        self.PLCs.append(plc)
        return plc
    
    def Start(self, join=False):
        """
        Start the connection by binding the receiving socket
        and starting the receiving thread.
        """

        self.CPSocket.bind()

        # Start thread
        self.thread = threading.Thread(target=self.connection_thread)
        self.thread.start()

        if join:
            self.thread.join()
    
    def Join(self):
        """
        Join the receiving thread.
        """

        self.thread.join()

    def connection_thread(self):
        """
        The receiving thread, to be run by self.Start.
        Listens for incomming packets and then sends them to
        the appropriate Consumer instance (TODO).
        """

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
        """
        Closes the sockets
        """

        self.setupSocket.close()
        self.CPSocket.close()