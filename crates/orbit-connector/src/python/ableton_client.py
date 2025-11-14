"""
Python OSC client for controlling Ableton Live through the OrbitRemote script.
"""

import socket
import struct
from typing import List, Union, Optional, Dict, Any
import asyncio
import threading
import time
import json


class AbletonOSCClient:
    """OSC client for sending commands to Ableton Live via OrbitRemote"""

    def __init__(self, host: str = "127.0.0.1", port: int = 11000, response_port: int = 11001):
        self.host = host
        self.port = port
        self.response_port = response_port
        self.socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)

        # Response listener setup
        self.response_socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self.response_socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        self.response_socket.bind(('127.0.0.1', self.response_port))
        self.response_socket.settimeout(0.1)

        self.responses: Dict[str, Any] = {}
        self.response_events: Dict[str, threading.Event] = {}
        self.request_locks: Dict[str, threading.Lock] = {}
        self.running = True

        # Start response listener thread
        self.listener_thread = threading.Thread(target=self._response_listener)
        self.listener_thread.daemon = True
        self.listener_thread.start()

    def __del__(self):
        self.running = False
        if hasattr(self, 'listener_thread'):
            self.listener_thread.join(timeout=0.5)
        if hasattr(self, 'socket'):
            self.socket.close()
        if hasattr(self, 'response_socket'):
            self.response_socket.close()

    def _response_listener(self):
        """Listen for OSC responses from Ableton"""
        while self.running:
            try:
                data, addr = self.response_socket.recvfrom(4096)
                address, args = self._parse_osc_message(data)

                # Store response and signal any waiting threads
                self.responses[address] = args
                if address in self.response_events:
                    self.response_events[address].set()
                    print(f"DEBUG: Signaled event for {address}")
                else:
                    print(f"DEBUG: No event waiting for {address}")

                # Log for debugging
                print(f"Received OSC response: {address} {args}")

            except socket.timeout:
                continue
            except Exception as e:
                if self.running:
                    print(f"OSC response listener error: {e}")

    def _parse_osc_message(self, data: bytes) -> tuple:
        """Parse an OSC message from bytes"""
        try:
            # Find the comma that separates address from type tags
            idx = data.index(b',')
            address = data[:idx].decode('utf-8').rstrip('\x00')

            # Parse type tags
            type_tag_start = idx
            idx = data.index(b'\x00', type_tag_start) + 1
            if idx % 4 != 0:
                idx += 4 - (idx % 4)

            type_tags_str = data[type_tag_start:idx].decode('utf-8').rstrip('\x00')

            values = []
            for tag in type_tags_str[1:]:  # Skip the comma
                if tag == 'i':
                    value = struct.unpack('>i', data[idx:idx + 4])[0]
                    values.append(value)
                    idx += 4
                elif tag == 'f':
                    value = struct.unpack('>f', data[idx:idx + 4])[0]
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
            print(f"OSC parse error: {e}")
            return "", []

    def _encode_osc_message(self, address: str, args: List[Union[int, float, str]]) -> bytes:
        """Encode an OSC message into bytes"""
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

    def send_message(self, address: str, args: Optional[List[Union[int, float, str]]] = None) -> bool:
        """Send an OSC message to Ableton Live"""
        if args is None:
            args = []

        try:
            message = self._encode_osc_message(address, args)
            self.socket.sendto(message, (self.host, self.port))
            return True
        except Exception as e:
            print(f"Failed to send OSC message {address}: {e}")
            return False

    def send_and_wait_for_response(self, address: str, args: Optional[List[Union[int, float, str]]] = None,
                                   response_address: str = None, timeout: float = 5.0) -> Optional[Any]:
        """Send an OSC message and wait for a response"""
        if response_address is None:
            response_address = address + "/response"

        # Get or create a lock for this address to prevent concurrent requests
        if address not in self.request_locks:
            self.request_locks[address] = threading.Lock()
        request_lock = self.request_locks[address]

        with request_lock:
            # Clear any old response
            if response_address in self.responses:
                del self.responses[response_address]

            # Create event for this response
            event = threading.Event()
            self.response_events[response_address] = event

            try:
                # Send the message
                if not self.send_message(address, args):
                    return None

                # Wait for response
                if event.wait(timeout):
                    response = self.responses.get(response_address)
                    return response
                else:
                    print(f"Timeout waiting for response to {address}")
                    return None

            finally:
                # Clean up
                if response_address in self.response_events:
                    del self.response_events[response_address]

    # Transport controls
    def play(self) -> bool:
        """Start playback in Ableton Live"""
        return self.send_message("/live/play")

    def stop(self) -> bool:
        """Stop playback in Ableton Live"""
        return self.send_message("/live/stop")

    def set_tempo(self, bpm: float) -> bool:
        """Set the tempo in BPM"""
        return self.send_message("/live/tempo", [bpm])

    # Track controls
    def set_track_volume(self, track_id: int, volume: float) -> bool:
        """Set track volume (0.0 to 1.0)"""
        return self.send_message("/live/track/volume", [track_id, volume])

    def mute_track(self, track_id: int, mute: bool = True) -> bool:
        """Mute or unmute a track"""
        return self.send_message("/live/track/mute", [track_id, int(mute)])

    def solo_track(self, track_id: int, solo: bool = True) -> bool:
        """Solo or unsolo a track"""
        return self.send_message("/live/track/solo", [track_id, int(solo)])

    def arm_track(self, track_id: int, arm: bool = True) -> bool:
        """Arm or disarm a track for recording"""
        return self.send_message("/live/track/arm", [track_id, int(arm)])

    # Clip and scene controls
    def launch_clip(self, track_id: int, clip_slot: int) -> bool:
        """Launch a specific clip"""
        return self.send_message("/live/clip/launch", [track_id, clip_slot])

    def launch_scene(self, scene_id: int) -> bool:
        """Launch a scene"""
        return self.send_message("/live/scene/launch", [scene_id])

    # Info retrieval
    def get_live_set_info(self) -> Optional[Dict[str, Any]]:
        """Get current Live set information"""
        response = self.send_and_wait_for_response("/live/get")
        if response and len(response) > 0:
            # Parse the response - it comes as a string representation of a dict
            try:
                info_str = response[0]
                # Use ast.literal_eval for safe parsing of the dict string
                import ast
                info_dict = ast.literal_eval(info_str)
                return info_dict
            except Exception as e:
                print(f"Failed to parse Live set info: {e}")
                return None
        return None

    def get_track_names(self) -> Optional[List[Dict[str, Any]]]:
        """Get list of all tracks with their names and properties"""
        print("DEBUG: Sending /live/tracks message")
        response = self.send_and_wait_for_response("/live/tracks")
        print(f"DEBUG: Got response: {response}")
        if response and len(response) > 0:
            try:
                tracks_str = response[0]
                print(f"DEBUG: Parsing: {tracks_str[:100]}...")
                # Try JSON first since server now uses json.dumps
                import json
                tracks_list = json.loads(tracks_str)
                return tracks_list
            except Exception as e:
                print(f"DEBUG: JSON parse failed, trying ast: {e}")
                try:
                    import ast
                    tracks_list = ast.literal_eval(tracks_str)
                    return tracks_list
                except Exception as e2:
                    print(f"Failed to parse track info: {e2}")
                    return None
        return None


class AsyncAbletonOSCClient(AbletonOSCClient):
    """Async version of the Ableton OSC client"""

    async def send_message_async(self, address: str, args: Optional[List[Union[int, float, str]]] = None) -> bool:
        """Send an OSC message asynchronously"""
        loop = asyncio.get_event_loop()
        return await loop.run_in_executor(None, self.send_message, address, args)

    async def play_async(self) -> bool:
        """Start playback in Ableton Live (async)"""
        return await self.send_message_async("/live/play")

    async def stop_async(self) -> bool:
        """Stop playback in Ableton Live (async)"""
        return await self.send_message_async("/live/stop")

    async def set_tempo_async(self, bpm: float) -> bool:
        """Set the tempo in BPM (async)"""
        return await self.send_message_async("/live/tempo", [bpm])

    async def set_track_volume_async(self, track_id: int, volume: float) -> bool:
        """Set track volume (async)"""
        return await self.send_message_async("/live/track/volume", [track_id, volume])

    async def mute_track_async(self, track_id: int, mute: bool = True) -> bool:
        """Mute or unmute a track (async)"""
        return await self.send_message_async("/live/track/mute", [track_id, int(mute)])

    async def solo_track_async(self, track_id: int, solo: bool = True) -> bool:
        """Solo or unsolo a track (async)"""
        return await self.send_message_async("/live/track/solo", [track_id, int(solo)])

    async def arm_track_async(self, track_id: int, arm: bool = True) -> bool:
        """Arm or disarm a track for recording (async)"""
        return await self.send_message_async("/live/track/arm", [track_id, int(arm)])

    async def launch_clip_async(self, track_id: int, clip_slot: int) -> bool:
        """Launch a specific clip (async)"""
        return await self.send_message_async("/live/clip/launch", [track_id, clip_slot])

    async def launch_scene_async(self, scene_id: int) -> bool:
        """Launch a scene (async)"""
        return await self.send_message_async("/live/scene/launch", [scene_id])


# Global client instance for easy access from tools
_global_client: Optional[AbletonOSCClient] = None
_client_lock = threading.Lock()


def get_ableton_client() -> AbletonOSCClient:
    """Get or create the global Ableton OSC client"""
    global _global_client
    with _client_lock:
        if _global_client is None:
            try:
                _global_client = AbletonOSCClient()
            except OSError as e:
                if "Address already in use" in str(e):
                    # Try with a different port if 11001 is taken
                    _global_client = AbletonOSCClient(response_port=11002)
        elif isinstance(_global_client, AsyncAbletonOSCClient):
            # Return the async client since it inherits from sync client
            return _global_client
        return _global_client


def get_async_ableton_client() -> AsyncAbletonOSCClient:
    """Get or create the global async Ableton OSC client"""
    global _global_client
    with _client_lock:
        if _global_client is None:
            try:
                _global_client = AsyncAbletonOSCClient()
            except OSError as e:
                if "Address already in use" in str(e):
                    # Try with a different port if 11001 is taken
                    _global_client = AsyncAbletonOSCClient(response_port=11002)
        elif not isinstance(_global_client, AsyncAbletonOSCClient):
            # Replace sync client with async client
            old_client = _global_client
            try:
                _global_client = AsyncAbletonOSCClient()
            except OSError as e:
                if "Address already in use" in str(e):
                    _global_client = AsyncAbletonOSCClient(response_port=11002)
            # Clean up old client
            if hasattr(old_client, '__del__'):
                old_client.__del__()
        return _global_client
