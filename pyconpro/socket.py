import socket
import struct

HEADER_SIZE = 24

class SocketError(Exception):
    ...

class Socket(object):
    """
    Wrapper for socket.socket
    """

    def __init__(self, timeout=5.0):
        """
        Initiate socket
        """

        self.timeout = timeout
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.sock.settimeout(self.timeout)
        self.sock.setsockopt(socket.SOL_SOCKET, socket.SO_KEEPALIVE, 1)

        # Allows the program to quickly restart the socket
        self.sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)

    def send(self, msg, timeout=0):
        """
        Send a message (in multiple packets if necissary)
        """

        if timeout != 0:
            self.sock.settimeout(timeout)

        try:
            sent = self.sock.sendall(msg)
        except socket.error:
            raise SocketError("Socket connection broken.")
        
        return sent

    def receiveAll(self, bytelen, timeout=0):
        """
        Get message size and then try to receive the entire
        message (even if there are multiple packets).
        """

        try:
            if timeout != 0:
                self.sock.settimeout(timeout)
            data = self.sock.recv(bytelen)
            data_len = struct.unpack_from('<H', data, 2)[0]
            while len(data) - HEADER_SIZE < data_len:
                data += self.sock.recv(bytelen)

            return data
        except socket.error as err:
            raise SocketError(err)
    
    def receive(self, bytelen, timeout=0):
        """
        Receive a single packet.
        """

        try:
            if timeout != 0:
                self.sock.settimeout(timeout)
            data = self.sock.recv(bytelen)
            return data
        except socket.error as err:
            raise SocketError(err)

    def close(self):
        self.sock.close()

class SetupSocket(Socket):
    """
    A socket for setting up sessions with PLCs.
    """
    
    def connect(self, host):
        try:
            self.sock.connect((host, 44818))
        except:
            try:
                self.sock.shutdown(socket.SHUT_RD)
                self.sock.close()
            except:
                pass
            
            self.__init__(timeout=self.timeout)
            self.sock.connect((host, 44818))


class CPSocket(Socket):
    """
    A socket for receiving data from PLCs.
    """

    def __init__(self, timeout=5.0):
        """
        Initiate a UDP socket.
        """

        self.sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self.sock.settimeout(timeout)
        self.sock.setsockopt(socket.SOL_SOCKET, socket.SO_KEEPALIVE, 1)
    
    def bind(self):
        """
        Bind to any avalible ip at port 2222
        """

        self.sock.bind(("0.0.0.0", 2222))
    
    def sendto(self, packet, address):
        """
        Send a message to a specific address.
        """
        
        self.sock.sendto(packet, address)