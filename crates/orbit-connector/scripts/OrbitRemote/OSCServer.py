import socket
import struct
import threading
import time
from typing import Optional, Tuple, Any

class OSCServer:
    def __init__(self, parent, port=11000, host='127.0.0.1'):
        self.parent = parent
        self.port = port
        self.host = host
        self.socket: Optional[socket.socket] = None
        self.running = False
        self.thread: Optional[threading.Thread] = None
        self._start_server()

    def _start_server(self):
        try:
            self.socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
            self.socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            self.socket.bind((self.host, self.port))
            self.socket.settimeout(0.1)
            self.running = True
            self.thread = threading.Thread(target=self._receive_loop)
            self.thread.daemon = True
            self.thread.start()
            self.parent.log_message(f"OSC Server started on {self.host}:{self.port}")
        except Exception as e:
            self.parent.log_message(f"Failed to start OSC server: {e}")
            self.socket = None

    def _receive_loop(self):
        while self.running and self.socket:
            try:
                data, addr = self.socket.recvfrom(4096)
                self._handle_message(data, addr)
            except socket.timeout:
                continue
            except Exception as e:
                if self.running:
                    self.parent.log_message(f"OSC receive error: {e}")

    def _handle_message(self, data: bytes, addr: Tuple[str, int]):
        if not self.socket:
            return

        try:
            address, args = self._parse_osc_message(data)
            self.parent.log_message(f"OSC: {address} {args}")

            song = self.parent._c_instance.song()

            if address == "/live/play":
                song.start_playing()
                self._send_response(addr, "/live/play/response", ["success"])

            elif address == "/live/stop":
                song.stop_playing()
                self._send_response(addr, "/live/stop/response", ["success"])

            elif address == "/live/tempo" and args:
                tempo = float(args[0])
                song.tempo = max(20.0, min(999.0, tempo))
                self._send_response(addr, "/live/tempo/response", ["success", song.tempo])

            elif address == "/live/track/volume" and len(args) >= 2:
                track_id = int(args[0])
                volume = float(args[1])
                if 0 <= track_id < len(song.tracks):
                    track = song.tracks[track_id]
                    if hasattr(track, 'mixer_device') and hasattr(track.mixer_device, 'volume'):
                        track.mixer_device.volume.value = max(0.0, min(1.0, volume))
                        self._send_response(addr, "/live/track/volume/response", ["success", track_id, track.mixer_device.volume.value])

            elif address == "/live/track/mute" and len(args) >= 2:
                track_id = int(args[0])
                mute = bool(int(args[1]))
                if 0 <= track_id < len(song.tracks):
                    track = song.tracks[track_id]
                    track.mute = mute
                    self._send_response(addr, "/live/track/mute/response", ["success", track_id, track.mute])

            elif address == "/live/track/solo" and len(args) >= 2:
                track_id = int(args[0])
                solo = bool(int(args[1]))
                if 0 <= track_id < len(song.tracks):
                    track = song.tracks[track_id]
                    track.solo = solo
                    self._send_response(addr, "/live/track/solo/response", ["success", track_id, track.solo])

            elif address == "/live/track/arm" and len(args) >= 2:
                track_id = int(args[0])
                arm = bool(int(args[1]))
                if 0 <= track_id < len(song.tracks):
                    track = song.tracks[track_id]
                    if hasattr(track, 'can_be_armed') and track.can_be_armed:
                        track.arm = arm
                        self._send_response(addr, "/live/track/arm/response", ["success", track_id, track.arm])

            elif address == "/live/clip/launch" and len(args) >= 2:
                track_id = int(args[0])
                clip_slot = int(args[1])
                if 0 <= track_id < len(song.tracks) and 0 <= clip_slot < len(song.scenes):
                    clip_slots = song.tracks[track_id].clip_slots
                    if clip_slot < len(clip_slots) and clip_slots[clip_slot].clip:
                        clip_slots[clip_slot].clip.fire()
                        self._send_response(addr, "/live/clip/launch/response", ["success", track_id, clip_slot])

            elif address == "/live/scene/launch" and args:
                scene_id = int(args[0])
                if 0 <= scene_id < len(song.scenes):
                    song.scenes[scene_id].fire()
                    self._send_response(addr, "/live/scene/launch/response", ["success", scene_id])

            elif address == "/live/get":
                response = self._get_live_set_info()
                self._send_response(addr, "/live/get/response", response)

        except Exception as e:
            self.parent.log_message(f"Error handling OSC message: {e}")
            self._send_response(addr, "/error", [str(e)])

    def _get_live_set_info(self):
        song = self.parent._c_instance.song()
        info = {
            "tempo": song.tempo,
            "is_playing": song.is_playing,
            "track_count": len(song.tracks),
            "scene_count": len(song.scenes)
        }
        return [str(info)]

    def _parse_osc_message(self, data: bytes) -> Tuple[str, list]:
        try:
            idx = data.index(b',')
            address = data[:idx].decode('utf-8').rstrip('\x00')

            type_tags = []
            values = []

            type_tag_start = idx
            idx = data.index(b'\x00', type_tag_start) + 1
            if idx % 4 != 0:
                idx += 4 - (idx % 4)

            type_tags_str = data[type_tag_start:idx].decode('utf-8').rstrip('\x00')

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
            self.parent.log_message(f"OSC parse error: {e}")
            return "", []

    def _send_response(self, addr: Tuple[str, int], osc_addr: str, args: list):
        if not self.socket:
            return

        try:
            message = self._encode_osc_message(osc_addr, args)
            self.socket.sendto(message, addr)
        except Exception as e:
            self.parent.log_message(f"Failed to send OSC response: {e}")

    def _encode_osc_message(self, address: str, args: list) -> bytes:
        address_bytes = address.encode('utf-8')
        if len(address_bytes) % 4 != 0:
            address_bytes += b'\x00' * (4 - len(address_bytes) % 4)

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
                if len(s_bytes) % 4 != 0:
                    s_bytes += b'\x00' * (4 - len(s_bytes) % 4)
                arg_bytes += s_bytes

        type_tag_bytes = type_tags.encode('utf-8') + b'\x00'
        if len(type_tag_bytes) % 4 != 0:
            type_tag_bytes += b'\x00' * (4 - len(type_tag_bytes) % 4)

        return address_bytes + type_tag_bytes + arg_bytes

    def process_messages(self):
        pass

    def shutdown(self):
        self.running = False
        if self.thread:
            self.thread.join(timeout=1.0)
        if self.socket:
            self.socket.close()
            self.socket = None
        self.parent.log_message("OSC Server shut down")