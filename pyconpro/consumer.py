from struct import pack, unpack_from
from random import randrange
from typing import NamedTuple
from threading import Timer

from . import CommError # pylint: disable=import-error

class ConsumerHint(NamedTuple):
    """
    Hint for the Consumer
    """

    tag: str
    datasize: int
    rpi: int
    otrpi: int

class Consumer:
    """
    Registers a forward open with a PLC, keeps the connection
    open, and receives data from the Connection instance.
    """

    def __init__(self, plc, hint, handler, args=(), kwargs={}):
        """
        Initiates Consumer by initiating variables and checking
        types.
        """

        self.plc = plc

        self._handler = handler
        self._handler_args = args
        self._handler_kwargs = kwargs

        if not isinstance(hint, ConsumerHint):
            raise TypeError("hint must be of type ConsumerHint")
        self.hint = hint

    def getTagSize(self):
        """
        Gets the size of the tag. May not be implemented.
        """

        ...
    
    def forwardOpen(self):
        """
        Registers a forward open with the given plc.
        """

        # Send Forward Open to producer
        self.plc.connection.setupSocket.send(self._buildForwardOpenPacket())
        
        forward_open_response = self.plc.connection.setupSocket.receiveAll(4096)

        # Handle response
        sts = unpack_from('<b', forward_open_response, 42)[0]
        if not sts:
            self.OTConnectionID = unpack_from('<I', forward_open_response, 44)[0]
            self.TOConnectionID = unpack_from('<I', forward_open_response, 48)[0] # Get ID for Connection instance to check
        else:
            raise CommError('Forward open failed')
    
    def _buildForwardOpenPacket(self):
        """
        Assemble the forward open packet
        """

        forwardOpen = self._buildCIPForwardOpen()
        rrDataHeader = self._buildEIPSendRRDataHeader(len(forwardOpen))
        return rrDataHeader+forwardOpen

    def _buildEIPSendRRDataHeader(self, frameLen):
        """
        Build the EIP Send RR Data Header
        """

        EIPCommand = 0x6F
        EIPLength = 16+frameLen
        EIPSessionHandle = self.plc.SessionHandle
        EIPStatus = 0x00
        EIPContext = 0x8000004a00000000
        EIPOptions = 0x00

        EIPInterfaceHandle = 0x00
        EIPTimeout = 0x00
        EIPItemCount = 0x02
        EIPItem1Type = 0x00
        EIPItem1Length = 0x00
        EIPItem2Type = 0xB2
        EIPItem2Length = frameLen

        return pack('<HHIIQIIHHHHHH',
                    EIPCommand,
                    EIPLength,
                    EIPSessionHandle,
                    EIPStatus,
                    EIPContext,
                    EIPOptions,
                    EIPInterfaceHandle,
                    EIPTimeout,
                    EIPItemCount,
                    EIPItem1Type,
                    EIPItem1Length,
                    EIPItem2Type,
                    EIPItem2Length)

    def _buildCIPForwardOpen(self):
        """
        Forward Open happens after a connection is made,
        this will sequp the CIP connection parameters
        """

        CIPPathSize = 0x02
        CIPClassType = 0x20

        CIPClass = 0x06
        CIPInstanceType = 0x24

        CIPInstance = 0x01
        CIPPriority = 0x0A
        CIPTimeoutTicks = 0x0e
        CIPOTConnectionID = 0x00
        CIPTOConnectionID = randrange(65000)
        self.SerialNumber = randrange(65000)
        CIPConnectionSerialNumber = self.SerialNumber
        CIPVendorID = 0x01 # VendorID
        CIPOriginatorSerialNumber = 42 # OriginatorSerialNumber
        CIPMultiplier = 0x00
        CIPOTRPI = int(self.hint.otrpi)*1000
        CIPConnectionParameters = 0x4802
        CIPTORPI = int(self.hint.rpi)*1000
        CIPTransportTrigger = 0x81

        # decide whether to use the standard ForwardOpen
        # or the large format
        CIPService = 0x54
        #CIPConnectionParameters += 508
        pack_format = '<BBBBBBBBIIHHIIIHIHB'

        CIPOTNetworkConnectionParameters = CIPConnectionParameters
        CIPTONetworkConnectionParameters = 0x4800+self.hint.datasize

        ForwardOpen = pack(pack_format,
                           CIPService,
                           CIPPathSize,
                           CIPClassType,
                           CIPClass,
                           CIPInstanceType,
                           CIPInstance,
                           CIPPriority,
                           CIPTimeoutTicks,
                           CIPOTConnectionID,
                           CIPTOConnectionID,
                           CIPConnectionSerialNumber,
                           CIPVendorID,
                           CIPOriginatorSerialNumber,
                           CIPMultiplier,
                           CIPOTRPI,
                           CIPOTNetworkConnectionParameters,
                           CIPTORPI,
                           CIPTONetworkConnectionParameters,
                           CIPTransportTrigger)

        # add the connection path
        path_size, path = self._connection_path()
        connection_path = pack('<B', int(path_size/2))

        connection_path += path
        return ForwardOpen + connection_path

    def _connection_path(self):
        PortSegment = 0x01 #b
        LinkAddress = 0x00 #b
        KeySegment = 0x34 #b
        KeyFormat = 0x04 #b
        VendorID = 0x00 #h
        DeviceType = 0x00 #h
        ProductCode = 0x00 #h
        MajorRevision = 0x00 #b
        MinorRevision = 0x00 #b

        path = pack('<BBBBHHHBB',
                    PortSegment,
                    LinkAddress,
                    KeySegment,
                    KeyFormat,
                    VendorID,
                    DeviceType,
                    ProductCode,
                    MajorRevision,
                    MinorRevision)

        ANSISymbol = self._buildTagIOI(self.hint.tag, 160)
        path += ANSISymbol

        return len(path), path

    def _buildTagIOI(self, tagName, data_type):
        """
        The tag IOI is basically the tag name assembled into
        an array of bytes structured in a way that the PLC will
        understand.  It's a little crazy, but we have to consider the
        many variations that a tag can be:

        TagName (DINT)
        TagName.1 (Bit of DINT)
        TagName.Thing (UDT)
        TagName[4].Thing[2].Length (more complex UDT)

        We also might be reading arrays, a bool from arrays (atomic), strings.
            Oh and multi-dim arrays, program scope tags...
        """

        ioi = b""
        tagArray = tagName.split(".")

        # this loop figures out the packet length and builds our packet
        for i in range(len(tagArray)):
            if tagArray[i].endswith("]"):
                tag, base_tag, index = _parseTagName(tagArray[i], 0)
                del tag

                BaseTagLenBytes = len(base_tag)
                if data_type == 211 and i == len(tagArray)-1:
                    index = int(index/32)

                # Assemble the packet
                ioi += pack('<BB', 0x91, BaseTagLenBytes)
                ioi += base_tag.encode('utf-8')
                if BaseTagLenBytes % 2:
                    BaseTagLenBytes += 1
                    ioi += pack('<B', 0x00)

                # BaseTagLenWords = BaseTagLenBytes/2
                if i < len(tagArray):
                    if not isinstance(index, list):
                        if index < 256:
                            ioi += pack('<BB', 0x28, index)
                        if 65536 > index > 255:
                            ioi += pack('<HH', 0x29, index)
                        if index > 65535:
                            ioi += pack('<HI', 0x2A, index)
                    else:
                        for i in range(len(index)):
                            if index[i] < 256:
                                ioi += pack('<BB', 0x28, index[i])
                            if 65536 > index[i] > 255:
                                ioi += pack('<HH', 0x29, index[i])
                            if index[i] > 65535:
                                ioi += pack('<HI', 0x2A, index[i])
            else:
                """
                for non-array segment of tag
                the try might be a stupid way of doing this.  If the portion of the tag
                    can be converted to an integer successfully then we must be just looking
                    for a bit from a word rather than a UDT.  So we then don't want to assemble
                    the read request as a UDT, just read the value of the DINT.  We'll figure out
                    the individual bit in the read function.
                """

                try:
                    if int(tagArray[i]) <= 31:
                        pass
                except Exception:
                    BaseTagLenBytes = int(len(tagArray[i]))
                    ioi += pack('<BB', 0x91, BaseTagLenBytes)
                    ioi += tagArray[i].encode('utf-8')
                    if BaseTagLenBytes % 2:
                        BaseTagLenBytes += 1
                        ioi += pack('<B', 0x00)

        return ioi

    def Start(self):
        """
        Start the thread keeping the connection alive. Everything
        else after this should be passive.
        """

        self.keepAlive = LoopThread(self.hint.otrpi, self._askForData, daemon=True)
        self.keepAlive.start()

    def Stop(self):
        self.keepAlive.stop()

    def _askForData(self):
        """
        Send keep alive packets to producer
        """

        packet = self._response_packet()
        self.plc.connection.SequenceCount += 1
        self.plc.connection.CPSocket.sendto(packet, (self.plc.ip, 2222))

    def _response_packet(self):
        """
        Build the response packet
        """

        item_count = 0x02
        type_id = 0x8002
        length = 0x08
        connection_id = self.OTConnectionID
        sequence_number =  self.plc.connection.SequenceCount
        conn_data = 0x00b1
        data_length = 0x02
        sequence_count = 1

        payload = pack('<HHHIIHHH',
                        item_count,
                        type_id,
                        length,
                        connection_id,
                        sequence_number,
                        conn_data,
                        data_length,
                        sequence_count)

        return payload

    def handle(self, data):
        """
        Send the data to the handler function
        """

        self._handler(data, *self._handler_args, **self._handler_kwargs)

def _parseTagName(tag, offset):
    """
    Parse the packet to get the base tag name
    the offset is so that we can increment the array pointer if need be
    """

    bt = tag
    ind = 0
    try:
        if tag.endswith(']'):
            pos = (len(tag)-tag.rindex("["))  # find position of [
            bt = tag[:-pos]		    # remove [x]: result=SuperDuper
            temp = tag[-pos:]		    # remove tag: result=[x]
            ind = temp[1:-1]		    # strip the []: result=x
            s = ind.split(',')		    # split so we can check for multi dimensin array
            if len(s) == 1:
                ind = int(ind)
                # newTagName = bt+'['+str(ind+offset)+']'
            else:
                # if we have a multi dim array, return the index
                ind = []
                for i in range(len(s)):
                    s[i] = int(s[i])
                    ind.append(s[i])
        else:
            pass
        return tag, bt, ind
    except Exception:
        return tag, bt, 0

class LoopThread(object):
    """
    Runs a function every RPI milliseconds using threading.Timer
    """

    def __init__ (self, RPI, function, daemon=False):
        """
        Initiates variables
        """

        self._timer = None
        self.RPI = RPI * 0.001
        self.function = function
        self.daemon = daemon

        self.is_running = False

    def _run(self):
        """
        Run the function and start the timer again.
        """
        self.is_running = False
        self.start()
        self.function()

    def start(self):
        """
        Start the timer
        """

        if not self.is_running:
            self._timer = Timer(self.RPI, self._run)
            self._timer.daemon = self.daemon
            self._timer.start()

            self.is_running = True
        
    def stop(self):
        """
        Stop the current timer and create no more.
        """

        self._timer.cancel()
        self.is_running = False