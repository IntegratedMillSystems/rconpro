import socket
from struct import *
import threading

# Custom Exception
class CommError(Exception):
    ...

class PleaseResetError(Exception):
    ...

from .socket import SetupSocket, CPSocket # pylint: disable=import-error
from .plc import PLC # pylint: disable=import-error
from .consumer import Consumer, ConsumerHint # pylint: disable=import-error
# from .producer import Producer

SHUT_RDWR = 2

class Connection:
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
    
    def addPLC(self, ip, slot=0):
        """
        Initiates a PLC instance, registers our socket with
        the actual PLC over the network, and saves it to a list.
        """

        plc = PLC(self, ip, slot)
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
        self.thread = threading.Thread(target=self.connection_thread, daemon=True)
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
        the appropriate Consumer instance.
        """

        while True:
            try:
                # Receive data
                data = self.CPSocket.receive(1024)
                
                # Get ID and find the correct consumer
                connectionID = unpack_from('<I', data, 6)[0]

                for plc in self.PLCs:
                    for con in plc.Consumers:
                        if con.TOConnectionID == connectionID:

                            # Send the data to the handler
                            con.handle( data[20:] )

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