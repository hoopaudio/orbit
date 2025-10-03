import Live
from .OSCServer import OSCServer

class OrbitRemote:
    def __init__(self, c_instance):
        self._c_instance = c_instance
        self.log_message = c_instance.log_message
        self._osc_server = OSCServer(self, port=11000)
        self.log_message("OrbitRemote: Initialized with OSC server")

    def disconnect(self):
        self.log_message("OrbitRemote: Disconnecting")
        if self._osc_server:
            self._osc_server.shutdown()

    def refresh_state(self):
        pass

    def update_display(self):
        if self._osc_server:
            self._osc_server.process_messages()