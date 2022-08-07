#!/usr/bin/env python3

import argparse
import time
import threading
import os
import sys

import gi

gi.require_version("Gtk", "3.0")
gi.require_version("GdkPixbuf", "2.0")
from gi.repository import Gtk, GdkPixbuf, GLib, GObject

SRC_DIR = os.path.abspath(os.path.join(os.path.dirname(os.path.dirname(__file__)), "src"))
sys.path.append(SRC_DIR)

from pixelflut_client import Client, BinaryAlgorithms


def parse_args():
    parser = argparse.ArgumentParser()

    parser.add_argument("server_hostname")
    parser.add_argument("server_port")

    return parser.parse_args()


def get_new_pixbuf():
    global client

    receive_start = time.time()
    pixels = client.receive_binary(BinaryAlgorithms.RgbBase64)
    receive_end = time.time()

    render_start = time.time()
    pixbuf = GdkPixbuf.Pixbuf.new_from_bytes(
        GLib.Bytes.new(pixels),
        GdkPixbuf.Colorspace.RGB,
        False,
        8,
        client.size[0],
        client.size[1],
        client.size[0] * 3,
    )
    render_end = time.time()

    line = f"receiving: {receive_end - receive_start}s, rendering: {render_end - render_start}s, fps: {1 / ((receive_end - receive_start) + (render_end - render_start))}"
    print(f"\033[K{line}\033[{len(line)}D", end="", flush=True)

    return pixbuf


def display_pixbuf(pixbuf):
    global image
    image.set_from_pixbuf(pixbuf)


def update():
    while True:
        pixbuf = get_new_pixbuf()
        GLib.idle_add(display_pixbuf, pixbuf)


if __name__ == "__main__":
    args = parse_args()

    client = Client()
    client.connect(args.server_hostname, int(args.server_port))

    window = Gtk.Window(title=f"Pixelflut remote canvas ({args.server_hostname}:{args.server_port})")
    window.set_default_size(client.size[0], client.size[1])
    window.connect("destroy", Gtk.main_quit)

    image = Gtk.Image.new()
    window.add(image)
    window.show_all()

    worker = threading.Thread(target=update)
    worker.daemon = True
    worker.start()

    Gtk.main()
