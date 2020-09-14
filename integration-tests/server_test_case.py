import os
import logging
import time
import unittest
import docker
import socket
from datetime import datetime, timedelta
from docker.models.containers import Container
from docker.models.images import Image

DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


class ServerTestCase(unittest.TestCase):
    server_implementation: str
    container_port: int = 9876

    _client: docker.DockerClient = None
    _container: Container = None
    _image: Image = None

    @classmethod
    def setUpClass(cls) -> None:
        super().setUpClass()

        logging.info(f"Building image for {cls.server_implementation}")
        cls._client = docker.from_env()
        cls.image, log_generator = cls._client.images.build(path=os.path.join(DIR, cls.server_implementation), rm=True)
        for line in log_generator:
            if "stream" in line.keys():
                logging.debug(line["stream"], end="")

    def setUp(self) -> None:
        super().setUp()

        logging.info(f"Running {self.server_implementation} server")
        self._container = self._client.containers.run(self.image.id, detach=True, auto_remove=True)
        t = datetime.utcnow()
        while not self._container.attrs["State"]["Running"]:
            self._container.reload()
            if t + timedelta(seconds=10) < datetime.utcnow():
                self._container.stop()
                raise TimeoutError(f"Container {self._container.id} was not started in time")

    @classmethod
    def tearDownClass(cls) -> None:
        super().tearDownClass()
        cls._client.close()

    def tearDown(self) -> None:
        logging.info(f"Stopping {self.server_implementation} server")
        self._container.stop()

    def _connectClient(self) -> socket.socket:
        host = self._container.attrs["NetworkSettings"]["IPAddress"]
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)

        t = datetime.utcnow()
        while True:
            try:
                sock.connect((host, self.container_port))
                sock.settimeout(1)
                return sock
            except ConnectionRefusedError as e:
                if t + timedelta(seconds=10) < datetime.utcnow():
                    raise e

    @staticmethod
    def _send(client: socket.socket, content: str):
        if content[-1] != "\n":
            content += "\n"
        client.send(content.encode("ASCII"))

    @staticmethod
    def _recv(client: socket.socket) -> str or None:
        try:
            return client.recv(2048).decode("ASCII")
        except socket.timeout as e:
            return None

    def test_set_pixel(self):
        with self._connectClient() as client:
            self._send(client, "PX 0 0 AABBCC")
            response = self._recv(client)

            self.assertIsNone(response, "Expected empty response")

    def test_get_pixel(self):
        with self._connectClient() as client:
            self._send(client, "PX 0 0")
            response = self._recv(client)

            self.assertIsNotNone(response)
            self.assertRegex(response, r"^PX 0 0 .+$", "Expected a pixel color response")

    def test_set_and_gotten_pixel_are_same(self):
        with self._connectClient() as client:
            self._send(client, "PX 0 0 AABBCC")
            time.sleep(1)
            self._send(client, "PX 0 0")
            response = self._recv(client)

            self.assertIsNotNone(response)
            self.assertEqual(response, "PX 0 0 AABBCC\n")

    def test_get_size(self):
        with self._connectClient() as client:
            self._send(client, "SIZE")
            response = self._recv(client)

            self.assertIsNotNone(response)
            self.assertRegex(response, "^SIZE .+ .+$")
