#!/usr/bin/env python3
"""Test OSC communication with Ableton"""

import socket
import struct
import time
import threading

def encode_osc_message(address: str, args: list = None) -> bytes:
    """Encode an OSC message"""
    if args is None:
        args = []

    # Encode address with null padding
    address_bytes = address.encode('utf-8') + b'\x00'
    while len(address_bytes) % 4 != 0:
        address_bytes += b'\x00'

    # Build type tags and argument data
    type_tags = ','
    arg_bytes = b''

    for arg in args:
        if isinstance(arg, int):
            type_tags += 'i'
            arg_bytes += struct.pack('>i', arg)
        elif isinstance(arg, float):
            type_tags += 'f'
            arg_bytes += struct.pack('>f', arg)
        elif isinstance(arg, str):
            type_tags += 's'
            s_bytes = arg.encode('utf-8') + b'\x00'
            while len(s_bytes) % 4 != 0:
                s_bytes += b'\x00'
            arg_bytes += s_bytes

    # Encode type tags with null padding
    type_tag_bytes = type_tags.encode('utf-8') + b'\x00'
    while len(type_tag_bytes) % 4 != 0:
        type_tag_bytes += b'\x00'

    return address_bytes + type_tag_bytes + arg_bytes

def parse_osc_message(data: bytes) -> tuple:
    """Parse an OSC message"""
    try:
        idx = data.index(b',')
        address = data[:idx].decode('utf-8').rstrip('\x00')

        type_tag_start = idx
        idx = data.index(b'\x00', type_tag_start) + 1
        if idx % 4 != 0:
            idx += 4 - (idx % 4)

        type_tags_str = data[type_tag_start:idx].decode('utf-8').rstrip('\x00')

        values = []
        for tag in type_tags_str[1:]:
            if tag == 'i':
                value = struct.unpack('>i', data[idx:idx+4])[0]
                values.append(value)
                idx += 4
            elif tag == 'f':
                value = struct.unpack('>f', data[idx:idx+4])[0]
                values.append(value)
                idx += 4
            elif tag == 's':
                end = data.index(b'\x00', idx)
                value = data[idx:end].decode('utf-8')
                values.append(value)
                idx = end + 1
                if idx % 4 != 0:
                    idx += 4 - (idx % 4)

        return address, values
    except Exception as e:
        print(f"Parse error: {e}")
        return "", []

# Create sender socket
sender = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)

# Create listener socket for responses
listener = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
listener.bind(('127.0.0.1', 11001))
listener.settimeout(3.0)

print("Testing OSC communication with Ableton Live...")
print("Sender: 127.0.0.1 -> 127.0.0.1:11000")
print("Listener: 127.0.0.1:11001")
print("-" * 50)

# Test 1: Send /live/get
print("\n1. Sending /live/get to Ableton...")
message = encode_osc_message("/live/get")
sender.sendto(message, ('127.0.0.1', 11000))
print(f"   Sent: /live/get")

# Listen for response
try:
    data, addr = listener.recvfrom(4096)
    address, args = parse_osc_message(data)
    print(f"   Response from {addr}: {address} {args}")
except socket.timeout:
    print("   No response received (timeout after 3 seconds)")

# Test 2: Send /live/tempo with current tempo request
print("\n2. Testing /live/tempo...")
message = encode_osc_message("/live/tempo", [120.0])
sender.sendto(message, ('127.0.0.1', 11000))
print(f"   Sent: /live/tempo [120.0]")

try:
    data, addr = listener.recvfrom(4096)
    address, args = parse_osc_message(data)
    print(f"   Response from {addr}: {address} {args}")
except socket.timeout:
    print("   No response received")

# Test 3: Check what Ableton sends to port 11001
print("\n3. Listening for any messages from Ableton on port 11001...")
print("   (Waiting 5 seconds for any messages...)")

listener.settimeout(1.0)
start_time = time.time()
message_count = 0

while time.time() - start_time < 5:
    try:
        data, addr = listener.recvfrom(4096)
        address, args = parse_osc_message(data)
        message_count += 1
        print(f"   Message {message_count} from {addr}: {address} {args}")
    except socket.timeout:
        continue

if message_count == 0:
    print("   No messages received from Ableton")

sender.close()
listener.close()

print("\n" + "=" * 50)
print("Test complete.")
print("\nIf no responses were received, check that:")
print("1. Ableton Live is running")
print("2. OrbitRemote is selected in Preferences > Link/Tempo/MIDI > Control Surface")
print("3. The OrbitRemote script is the updated version with response handling")