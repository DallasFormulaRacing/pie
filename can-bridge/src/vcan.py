# CAN BUS Traffic simulator
import can
import time
import random
import fixedint

# needs to be extended
CMD_ID_FIRST_24_CELLS = 0x00A0
CMD_ID_SECOND_24_CELLS = 0x00A1
CMD_ID_THIRD_24_CELLS = 0x00A2
CMD_ID_FOURTH_24_CELLS = 0x00A3
CMD_ID_FIFTH_24_CELLS = 0x00A4
CMD_ID_SIXTH_24_CELLS = 0x00A5
CMD_ID_FIRST_60_TEMPS = 0x00B0
CMD_ID_LAST_60_TEMPS = 0x00B1
CMD_ID_PACK_METADATA = 0x00C0
CMD_ID_IMD_DATA = 0x00D0

bus = can.interface.Bus(channel='vcan0', interface='socketcan', fd=True)

def build_arbitration_id(command, source=0x02, target=0x1E, priority=1):
    return ((priority & 0x07) << 26) | ((target & 0x1F) << 21) | ((command & 0xFFFF) << 5) | (source & 0x1F)

def send(data_in, command, source=0x02, target=0x1E, priority=1):
    arbitration_id = build_arbitration_id(command, source, target, priority)
    msg = can.Message(
        arbitration_id=arbitration_id,
        data=data_in,
        is_extended_id=True,
        is_fd=True
    )
    bus.send(msg)
    print(f"Sent: {data_in} cmd={hex(command)} src={hex(source)} id={hex(arbitration_id)}")
    time.sleep(0.12)

def generate_fake_cell_frame():
    data_array = []
    for _ in range(24):  
        val = int(generate_fake_cell_value())
        data_array.append((val >> 8) & 0xFF)
        data_array.append(val & 0xFF)
    return bytes(data_array)  

# 64 bytes, 4 byte padding, 1 byte per temp
def generate_fake_temp_frame():
    return bytes([0]*8)

# 24 bytes, 2, 2, 2, 18
def generate_fake_pack_frame():
    return bytes([0]*8)

def f2i(n: float):
    return fixedint.Int16(n/0.00015 - 1.5)

# return float (actual voltage) as int16_t with lsb = 150microVolt +/- 1.5
def generate_fake_cell_value():
    val = f2i(random.randint(2900, 4100)/1000.0)
    return val

#return uint8_t
def generate_fake_temp_value():
    random.randint(2900, 4100)  
    return bytes([0]*8)

try:
    while True:
        send(generate_fake_cell_frame(), CMD_ID_FIRST_24_CELLS)
        send(generate_fake_cell_frame(), CMD_ID_SECOND_24_CELLS)
        send(generate_fake_cell_frame(), CMD_ID_THIRD_24_CELLS)
        send(generate_fake_cell_frame(), CMD_ID_FOURTH_24_CELLS)
        send(generate_fake_cell_frame(), CMD_ID_FIFTH_24_CELLS)
        send(generate_fake_cell_frame(), CMD_ID_SIXTH_24_CELLS)
        send(generate_fake_temp_frame(), CMD_ID_FIRST_60_TEMPS)
        send(generate_fake_temp_frame(), CMD_ID_LAST_60_TEMPS)
        send(generate_fake_pack_frame(), CMD_ID_PACK_METADATA)
except KeyboardInterrupt:
    print("Shutting down...")
finally:
    bus.shutdown()