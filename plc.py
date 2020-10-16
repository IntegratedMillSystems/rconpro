from struct import *

from . import CommError
from .consumer import Consumer

class PLC:
    def __init__(self, connection, ip, slot):
        self.connection = connection

        self.ip = ip
        self.slot = slot

        self.Consumers = []

        # To be set by register
        self.SessionHandle = 0
    
    def register(self):
        # Register the connection
        self.connection.setupSocket.connect(self.ip)
        self.connection.setupSocket.send(self._buildRegisterSession())
        reg_response = self.connection.setupSocket.receiveAll(4096)

        # Handle response
        if reg_response:
            self.SessionHandle = unpack_from('<I', reg_response, 4)[0]
            print("Session Handle: ", self.SessionHandle)
        else:
            raise CommError('Register session failed')
    
    def _buildRegisterSession(self):
        """
        Build the packet to register the CIP connection
        """
        EIPCommand = 0x0065
        EIPLength = 0x0004
        EIPSessionHandle = 0x00
        EIPStatus = 0x0000
        EIPContext = 0x00
        EIPOptions = 0x0000

        EIPProtocolVersion = 0x01
        EIPOptionFlag = 0x00

        return pack('<HHIIQIHH',
                    EIPCommand,
                    EIPLength,
                    EIPSessionHandle,
                    EIPStatus,
                    EIPContext,
                    EIPOptions,
                    EIPProtocolVersion,
                    EIPOptionFlag)

    def addConsumer(self, hint, handler):
        con = Consumer(self, hint, handler)
        con.forwardOpen()

        self.Consumers.append(con)
        return con