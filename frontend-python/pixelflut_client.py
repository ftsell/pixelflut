import socket
import base64


class BinaryAlgorithms:
    RgbBase64 = "rgb64"
    RgbaBase64 = "rgba64"


class Client():
    sock = None  # type: socket.socket
    x_size = 0  # type: int
    y_size = 0  # type: int

    def __init__(self):
        self.sock = socket.socket()

    def connect(self, hostname, port):
        self.sock.connect((hostname, int(port)))
        self.x_size, self.y_size = self.get_size()

    def get_size(self) -> (int, int):
        self.sock.send(b"SIZE\n")
        response = self.sock.recv(256).decode("ASCII")
        # SIZE $X $Y
        x = response.split(" ")[1]
        y = response.split(" ")[2]

        return (int(x), int(y))

    def set_pixel(self, x: int, y: int, color: str):
        self.sock.send(f"PX {x} {y} {color}\n".encode("ASCII"))
        response = self.sock.recv(256).decode("ASCII")

        if response != f"PX {x} {y} {color}\n":
            escaped = response.replace("\n", "\\n")
            print(f"Error while setting pixel. Received Response: {escaped}")

    def get_pixel(self, x: int, y: int) -> str:
        self.sock.send(f"PX {x} {y}\n".encode("ASCII"))
        response = self.sock.recv(256).decode("ASCII")
        # PX $X $X $COLOR

        return response.split(" ")[3]

    def receive_binary(self, algorithm: str) -> list:
        """
        Returns a list of 8-bit integer values.
        Each value being one color channel.
        3 values representing one pixel
        """
        self.sock.send(f"STATE {algorithm}\n".encode("ASCII"))

        response = b''
        while len(response) == 0 or response[-1] != 10:     # 10 is \n
            response += self.sock.recv(1024)
        response = response[:-1]        # remove \n
        response = response[len(f"STATE {algorithm}".encode("ASCII")):]

        return base64.b64decode(response)
