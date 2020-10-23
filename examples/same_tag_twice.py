from pyconpro import Connection, PLC, Consumer, ConsumerHint
from time import sleep

con = Connection()
myplc = con.addPLC("172.16.13.200")

def handler1(data):
    print("My first consumer: ", data)

def handler2(data):
    print("My second consumer: ", data)

con.Start()

myhint = ConsumerHint(tag="test", datasize=6, rpi=1000, otrpi=1100)
myconsumer = myplc.addConsumer(myhint, handler1)
myconsumer.Start()

sleep(0.3)

myhint2 = ConsumerHint(tag="test", datasize=6, rpi=1000, otrpi=1100)
myconsumer2 = myplc.addConsumer(myhint2, handler2)
myconsumer2.Start()

con.Join()
con.Close()