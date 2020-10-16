import socket
from . import CommError
import struct

HEADER_SIZE = 24

class Socket(object):
    def __init__(self, timeout=5.0):
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.sock.settimeout(timeout)
        self.sock.setsockopt(socket.SOL_SOCKET, socket.SO_KEEPALIVE, 1)

    def send(self, msg, timeout=0):
        if timeout != 0:
            self.sock.settimeout(timeout)
        total_sent = 0
        while total_sent < len(msg):
            try:
                sent = self.sock.send(msg[total_sent:])
                if sent == 0:
                    raise CommError("Socket connection broken.")
                total_sent += sent
            except socket.error:
                raise CommError("Socket connection broken.")
        return total_sent

    def receiveAll(self, bytelen, timeout=0):
        try:
            if timeout != 0:
                self.sock.settimeout(timeout)
            data = self.sock.recv(bytelen)
            data_len = struct.unpack_from('<H', data, 2)[0]
            while len(data) - HEADER_SIZE < data_len:
                data += self.sock.recv(bytelen)

            return data
        except socket.error as err:
            raise CommError(err)
    
    def receive(self, bytelen, timeout=0):
        try:
            if timeout != 0:
                self.sock.settimeout(timeout)
            data = self.sock.recv(bytelen)
            return data
        except socket.error as err:
            raise CommError(err)

    def close(self):
        self.sock.close()

class SetupSocket(Socket):
    def connect(self, host):
        try:
            self.sock.connect((host, 44818))
        except socket.timeout:
            raise CommError("Socket timeout during connection.")

class CPSocket(Socket):
    def __init__(self, timeout=5.0):
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self.sock.settimeout(timeout)
        self.sock.setsockopt(socket.SOL_SOCKET, socket.SO_KEEPALIVE, 1)
    
    def bind(self):
        self.sock.bind(("0.0.0.0", 2222))
    
    def sendto(self, packet, address):
        self.sock.sendto(packet, address)